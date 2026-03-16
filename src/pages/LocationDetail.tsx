import { useState, useMemo, useRef, useCallback, useEffect } from "react";
import { useParams, useNavigate, useLocation } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { convertFileSrc } from "@tauri-apps/api/core";
import { locationApi } from "@/services/locationApi";
import { deviceApi } from "@/services/deviceApi";
import type { DevicePublic } from "@/types/device";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Icon } from "@iconify/react";
import {
  Loader2,
  ImageOff,
  AlertCircle,
  ArrowLeft,
  MapPin,
  BarChart3,
  Cpu,
  Lock,
  LockOpen,
  Info,
  BarChart2,
} from "lucide-react";
import { toast } from "sonner";
import { listen } from "@tauri-apps/api/event";
import { LocationActionsPanel } from "@/components/LocationActionsPanel";
import { DeviceIconStatusBar } from "@/components/DeviceIconStatusBar";
import { SENSOR_READING_LATEST_ALL_QUERY_KEY } from "@/components/DeviceIconStatusBar";

const DEVICE_DASHBOARD_UPDATE_EVENT = "device-dashboard-update";
const DEBOUNCE_MS = 500;

const LocationDetail = () => {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [isActivating, setIsActivating] = useState(false);
  const [unallocatedDropdownOpen, setUnallocatedDropdownOpen] = useState(false);
  const [pendingDevices, setPendingDevices] = useState<
    Array<{ device: DevicePublic; position_x: number; position_y: number }>
  >([]);
  const [openBarDeviceUuid, setOpenBarDeviceUuid] = useState<string | null>(null);
  const [confirmingDeviceUuid, setConfirmingDeviceUuid] = useState<string | null>(null);
  const [editingAllocatedPositions, setEditingAllocatedPositions] = useState<
    Record<string, { position_x: number; position_y: number }>
  >({});
  const [deviceInfoPopupDevice, setDeviceInfoPopupDevice] = useState<DevicePublic | null>(null);
  const [dragState, setDragState] = useState<{
    deviceUuid: string;
    startClientX: number;
    startClientY: number;
    startPosX: number;
    startPosY: number;
  } | null>(null);
  const overlayContainerRef = useRef<HTMLDivElement>(null);
  const barAutoCloseTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pathnameRef = useRef(pathname);
  pathnameRef.current = pathname;

  const { data: location, isLoading, error: queryError } = useQuery({
    queryKey: ["location", uuid],
    queryFn: async () => {
      if (!token || !uuid) {
        logout();
        return null;
      }
      const result = await locationApi.getLocation(token, uuid);
      if (!result.success) {
        if (result.unauthorized) {
          toast.error("Session expired. Please login again.");
          logout();
          return null;
        }
        const errorMsg = result.message ?? "Failed to load location.";
        console.error("getLocation error:", errorMsg);
        throw new Error(errorMsg);
      }
      if (!result.data) {
        console.error("getLocation: no data returned");
        return null;
      }
      return result.data;
    },
    enabled: !!uuid && !!token,
    retry: false,
  });

  const { data: devicesData } = useQuery({
    queryKey: ["devices", "location", uuid],
    queryFn: async () => {
      if (!token || !uuid) {
        logout();
        return null;
      }
      const result = await deviceApi.listDevices(token, {
        page: 1,
        page_size: 100,
        filter: { location_uuid: uuid },
      });
      if (!result.success) {
        if (result.unauthorized) {
          toast.error("Session expired. Please login again.");
          logout();
          return null;
        }
        throw new Error(result.message ?? "Failed to load devices.");
      }
      return result.data ?? { items: [], total: 0, page: 1, page_size: 100 };
    },
    enabled: !!uuid && !!token,
  });

  const devices = devicesData?.items ?? [];
  const { allocated, unallocated } = useMemo(() => {
    const alloc: DevicePublic[] = [];
    const unalloc: DevicePublic[] = [];
    for (const d of devices) {
      if (
        d.position_x != null &&
        d.position_y != null &&
        typeof d.position_x === "number" &&
        typeof d.position_y === "number"
      ) {
        alloc.push(d);
      } else {
        unalloc.push(d);
      }
    }
    return { allocated: alloc, unallocated: unalloc };
  }, [devices]);

  const unallocatedToShow = useMemo(
    () => unallocated.filter((d) => !pendingDevices.some((p) => p.device.uuid === d.uuid)),
    [unallocated, pendingDevices]
  );

  const devicesOnImage = useMemo(() => {
    const items: Array<{
      device: DevicePublic;
      position_x: number;
      position_y: number;
      isPending: boolean;
      isEditingAllocated: boolean;
    }> = [];
    const clampPos = (v: number) => Math.min(95, Math.max(5, v));
    for (const p of pendingDevices) {
      items.push({
        device: p.device,
        position_x: clampPos(p.position_x),
        position_y: clampPos(p.position_y),
        isPending: true,
        isEditingAllocated: false,
      });
    }
    for (const d of allocated) {
      const editPos = editingAllocatedPositions[d.uuid];
      const px = clampPos(editPos?.position_x ?? d.position_x ?? 50);
      const py = clampPos(editPos?.position_y ?? d.position_y ?? 50);
      items.push({
        device: d,
        position_x: px,
        position_y: py,
        isPending: false,
        isEditingAllocated: !!editPos,
      });
    }
    return items;
  }, [pendingDevices, allocated, editingAllocatedPositions]);

  const handleConfirmAllocation = useCallback(
    async (deviceUuid: string, position_x: number, position_y: number) => {
      if (!token) return;
      if (barAutoCloseTimeoutRef.current) {
        clearTimeout(barAutoCloseTimeoutRef.current);
        barAutoCloseTimeoutRef.current = null;
      }
      setConfirmingDeviceUuid(deviceUuid);
      try {
        const result = await deviceApi.updateDevice(token, {
          uuid: deviceUuid,
          position_x,
          position_y,
        });
        if (!result.success) {
          if (result.unauthorized) {
            toast.error("Session expired. Please login again.");
            logout();
            return;
          }
          toast.error(result.message ?? "Failed to save position.");
          return;
        }
        setPendingDevices((prev) => prev.filter((p) => p.device.uuid !== deviceUuid));
        setEditingAllocatedPositions((prev) => {
          const next = { ...prev };
          delete next[deviceUuid];
          return next;
        });
        setOpenBarDeviceUuid(null);
        queryClient.invalidateQueries({ queryKey: ["devices", "location", uuid] });
        toast.success("Device position saved.");
      } finally {
        setConfirmingDeviceUuid(null);
      }
    },
    [token, logout, queryClient, uuid]
  );

  const handleDeviceSelectForAllocation = (device: DevicePublic) => {
    setPendingDevices((prev) =>
      prev.some((p) => p.device.uuid === device.uuid) ? prev : [...prev, { device, position_x: 50, position_y: 50 }]
    );
    setUnallocatedDropdownOpen(false);
  };

  const handleDragMove = useCallback(
    (e: MouseEvent) => {
      if (!dragState || !overlayContainerRef.current) return;
      const rect = overlayContainerRef.current.getBoundingClientRect();
      const deltaX = ((e.clientX - dragState.startClientX) / rect.width) * 100;
      const deltaY = ((e.clientY - dragState.startClientY) / rect.height) * 100;
      // Clamp para manter ícone inteiramente dentro da imagem (centro + metade do ícone)
      const newX = Math.min(95, Math.max(5, dragState.startPosX + deltaX));
      const newY = Math.min(95, Math.max(5, dragState.startPosY + deltaY));
      setPendingDevices((prev) =>
        prev.map((p) =>
          p.device.uuid === dragState.deviceUuid
            ? { ...p, position_x: newX, position_y: newY }
            : p
        )
      );
      setEditingAllocatedPositions((prev) => {
        if (!(dragState.deviceUuid in prev)) return prev;
        return { ...prev, [dragState.deviceUuid]: { position_x: newX, position_y: newY } };
      });
    },
    [dragState]
  );

  const handleDragEnd = useCallback(() => {
    setDragState(null);
  }, []);

  const handleDragStart = useCallback(
    (deviceUuid: string, position_x: number, position_y: number, e: React.MouseEvent) => {
      if (e.button !== 0) return;
      e.preventDefault();
      if (barAutoCloseTimeoutRef.current) {
        clearTimeout(barAutoCloseTimeoutRef.current);
        barAutoCloseTimeoutRef.current = null;
      }
      setDragState({
        deviceUuid,
        startClientX: e.clientX,
        startClientY: e.clientY,
        startPosX: position_x,
        startPosY: position_y,
      });
    },
    []
  );

  useEffect(() => {
    if (!dragState) return;
    const onMove = (e: MouseEvent) => handleDragMove(e);
    const onUp = () => handleDragEnd();
    document.body.style.cursor = "grabbing";
    document.body.style.userSelect = "none";
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
  }, [dragState, handleDragMove, handleDragEnd]);

  useEffect(() => {
    if (!openBarDeviceUuid) return;
    barAutoCloseTimeoutRef.current = setTimeout(() => {
      setEditingAllocatedPositions((prev) => {
        const next = { ...prev };
        delete next[openBarDeviceUuid!];
        return next;
      });
      setOpenBarDeviceUuid(null);
      barAutoCloseTimeoutRef.current = null;
    }, 5000);
    return () => {
      if (barAutoCloseTimeoutRef.current) {
        clearTimeout(barAutoCloseTimeoutRef.current);
        barAutoCloseTimeoutRef.current = null;
      }
    };
  }, [openBarDeviceUuid]);

  // Fase 3: atualização em tempo real da barra de status dos devices
  const deviceUuidsRef = useRef<Set<string>>(new Set());
  deviceUuidsRef.current = new Set(devicesData?.items?.map((d) => d.uuid) ?? []);

  useEffect(() => {
    if (!token || !uuid) return;

    const debounceTimers = new Map<string, ReturnType<typeof setTimeout>>();

    const unlistenPromise = listen<{ device_uuid: string }>(
      DEVICE_DASHBOARD_UPDATE_EVENT,
      (event) => {
        const deviceUuid = event.payload?.device_uuid;
        if (!deviceUuid) return;

        if (pathnameRef.current !== `/locations/${uuid}`) return;
        if (!deviceUuidsRef.current.has(deviceUuid)) return;

        const existing = debounceTimers.get(deviceUuid);
        if (existing) clearTimeout(existing);

        const timer = setTimeout(() => {
          debounceTimers.delete(deviceUuid);
          queryClient.invalidateQueries({
            queryKey: [...SENSOR_READING_LATEST_ALL_QUERY_KEY, deviceUuid],
          });
          queryClient.invalidateQueries({ queryKey: ["devices", "location", uuid] });
        }, DEBOUNCE_MS);

        debounceTimers.set(deviceUuid, timer);
      }
    );

    return () => {
      debounceTimers.forEach((t) => clearTimeout(t));
      debounceTimers.clear();
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [token, queryClient, uuid]);

  const handleDelete = async () => {
    if (!token || !uuid) return;
    setIsDeleting(true);

    const result = await locationApi.deleteLocation(token, uuid);
    setIsDeleting(false);
    setDeleteDialogOpen(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to delete location.");
      return;
    }

    toast.success("Location deleted successfully.");
    queryClient.invalidateQueries({ queryKey: ["locations-list"] });
    navigate("/locations/list");
  };

  const handleActivate = async () => {
    if (!token || !uuid) return;
    setIsActivating(true);

    const result = await locationApi.updateLocation(token, {
      uuid,
      is_active: true,
    });

    setIsActivating(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to activate location.");
      return;
    }

    toast.success("Location activated successfully.");
    queryClient.invalidateQueries({ queryKey: ["locations-list"] });
    queryClient.invalidateQueries({ queryKey: ["location", uuid] });
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (queryError) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] text-muted-foreground gap-2">
        <p className="font-semibold">Error loading location</p>
        <p className="text-sm">{queryError.message}</p>
        <Button onClick={() => navigate("/locations/list")} variant="outline">
          Back to List
        </Button>
      </div>
    );
  }

  if (!location) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] text-muted-foreground gap-2">
        <p>Location not found.</p>
        <Button onClick={() => navigate("/locations/list")} variant="outline">
          Back to List
        </Button>
      </div>
    );
  }

  // Add cache buster using updated_at timestamp to force reload after image update
  const cacheBuster = location.updated_at ? `?t=${new Date(location.updated_at).getTime()}` : '';
  const imageSrc = location.image_path
    ? `${convertFileSrc(location.image_path)}${cacheBuster}`
    : location.thumb_path
    ? `${convertFileSrc(location.thumb_path)}${cacheBuster}`
    : null;

  const fallback =
    "data:image/svg+xml;utf8," +
    encodeURIComponent(
      `<svg xmlns='http://www.w3.org/2000/svg' width='800' height='600' viewBox='0 0 800 600'><rect width='800' height='600' fill='%23f1f5f9'/><text x='50%' y='50%' dominant-baseline='middle' text-anchor='middle' fill='%2394a3b8' font-family='Arial' font-size='24'>No image saved</text></svg>`
    );

  const isInactive = !location.is_active;

  return (
    <>
      <div className="space-y-4">
        {isInactive && (
          <div className="border border-yellow-500/50 bg-yellow-500/10 rounded-lg p-4 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <AlertCircle className="w-5 h-5 text-yellow-600 dark:text-yellow-500" />
              <div>
                <p className="font-semibold text-yellow-900 dark:text-yellow-100">
                  Location Inactive
                </p>
                <p className="text-sm text-yellow-700 dark:text-yellow-300">
                  This location is currently inactive. Only activation is allowed.
                </p>
              </div>
            </div>
            <Button
              onClick={handleActivate}
              disabled={isActivating}
              className="bg-yellow-600 hover:bg-yellow-700 text-white"
            >
              {isActivating ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Activating...
                </>
              ) : (
                "Activate"
              )}
            </Button>
          </div>
        )}

        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => navigate("/locations/list")}
              aria-label="Back"
            >
              <ArrowLeft className="w-5 h-5" />
            </Button>
            <h1 className="text-2xl font-semibold">{location.name}</h1>
          </div>
          <div className="flex items-center gap-2">
            {unallocated.length > 0 && (
              <DropdownMenu open={unallocatedDropdownOpen} onOpenChange={setUnallocatedDropdownOpen}>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="outline"
                    size="icon"
                    aria-label={`${unallocatedToShow.length} device(s) to allocate`}
                    className="relative"
                  >
                    <MapPin className="w-5 h-5" />
                    <span className="absolute -top-1 -right-1 min-w-[18px] h-[18px] flex items-center justify-center rounded-full bg-primary text-primary-foreground text-xs font-medium px-1">
                      {unallocatedToShow.length > 99 ? "99+" : unallocatedToShow.length}
                    </span>
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="z-50 w-80 max-h-[400px] overflow-y-auto">
                  <div className="px-2 py-2 text-sm font-medium text-muted-foreground">
                    Devices to allocate
                  </div>
                  <DropdownMenuSeparator />
                  {unallocatedToShow.length === 0 ? (
                    <div className="px-4 py-6 text-center text-muted-foreground text-sm">
                      {pendingDevices.length > 0
                        ? "All selected devices are pending allocation"
                        : "No devices to allocate"}
                    </div>
                  ) : (
                    unallocatedToShow.map((device) => (
                      <DropdownMenuItem
                        key={device.uuid}
                        onClick={() => handleDeviceSelectForAllocation(device)}
                        className="flex flex-col items-start gap-1 py-2 cursor-pointer hover:bg-muted/70 focus:bg-muted/70"
                      >
                        <div className="flex items-center justify-between w-full gap-2">
                          <div className="flex items-center gap-2 min-w-0 flex-1">
                            {device.icon?.iconify_id ? (
                              <div
                                className="w-8 h-8 flex items-center justify-center rounded-lg shrink-0"
                                style={{
                                  backgroundColor: device.icon.color
                                    ? `${device.icon.color}20`
                                    : "var(--muted)",
                                }}
                              >
                                <Icon
                                  icon={device.icon.iconify_id}
                                  className="w-5 h-5"
                                  style={{ color: device.icon.color ?? undefined }}
                                />
                              </div>
                            ) : device.device_type === "sensor" ? (
                              <BarChart3 className="w-5 h-5 text-muted-foreground shrink-0" />
                            ) : (
                              <Cpu className="w-5 h-5 text-muted-foreground shrink-0" />
                            )}
                            <span className="font-medium truncate">{device.name}</span>
                          </div>
                          <span className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground shrink-0 capitalize">
                            {device.device_type}
                          </span>
                        </div>
                        {device.model && (
                          <span className="text-xs text-muted-foreground line-clamp-1">
                            {device.model}
                          </span>
                        )}
                      </DropdownMenuItem>
                    ))
                  )}
                </DropdownMenuContent>
              </DropdownMenu>
            )}
            {location.is_active && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => navigate(`/locations/${location.uuid}/devices/adopt`)}
              >
                Adopt Device
              </Button>
            )}
            <LocationActionsPanel
            locationUuid={location.uuid}
            isActive={location.is_active}
            name={location.name}
            address={location.address}
            description={location.description}
            onDelete={() => setDeleteDialogOpen(true)}
            />
          </div>
        </div>

      <div className="border border-border rounded-xl bg-card overflow-hidden">
        <div className="relative w-full bg-secondary/40 flex items-center justify-center min-h-[400px] max-h-[70vh] overflow-hidden">
          {imageSrc ? (
            <>
              <img
                src={imageSrc}
                alt={location.name}
                className="w-full h-auto object-contain"
                onError={(e) => {
                  e.currentTarget.src = fallback;
                }}
              />
              <div className="absolute inset-0 pointer-events-none">
                <div ref={overlayContainerRef} className="relative w-full h-full">
                  {devicesOnImage.map(
                    ({ device, position_x, position_y, isPending, isEditingAllocated }) => {
                    const isBarOpen = openBarDeviceUuid === device.uuid;
                    const canDrag = isBarOpen && (isPending || isEditingAllocated);
                    return (
                      <div
                        key={device.uuid}
                        className={`absolute flex flex-col items-center -translate-x-1/2 -translate-y-1/2 pointer-events-auto ${
                          canDrag
                            ? dragState?.deviceUuid === device.uuid
                              ? "cursor-grabbing"
                              : "cursor-grab"
                            : "cursor-pointer"
                        }`}
                        style={{
                          left: `${position_x}%`,
                          top: `${position_y}%`,
                        }}
                        title={
                          device.operation_status !== "online"
                            ? `${device.name} — Offline`
                            : device.name
                        }
                        onDoubleClick={(e) => {
                          e.stopPropagation();
                          setOpenBarDeviceUuid((prev) => {
                            if (prev === device.uuid) {
                              setEditingAllocatedPositions((p) => {
                                const next = { ...p };
                                delete next[device.uuid];
                                return next;
                              });
                              return null;
                            }
                            return device.uuid;
                          });
                        }}
                      >
                        {isBarOpen && (
                          <div
                            className="flex items-center gap-1 mb-1 px-2 py-1 rounded-md bg-popover border border-border shadow-md pointer-events-auto"
                            onDoubleClick={(e) => e.stopPropagation()}
                          >
                            {isPending || isEditingAllocated ? (
                              <button
                                type="button"
                                className="p-1 rounded hover:bg-accent disabled:opacity-50"
                                aria-label={isPending ? "Confirm allocation" : "Save position"}
                                disabled={confirmingDeviceUuid === device.uuid}
                                onClick={() =>
                                  handleConfirmAllocation(device.uuid, position_x, position_y)
                                }
                              >
                                {confirmingDeviceUuid === device.uuid ? (
                                  <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />
                                ) : (
                                  <LockOpen className="w-4 h-4 text-muted-foreground" />
                                )}
                              </button>
                            ) : (
                              <button
                                type="button"
                                className="p-1 rounded hover:bg-accent"
                                aria-label="Edit position"
                                onClick={() =>
                                  setEditingAllocatedPositions((prev) => ({
                                    ...prev,
                                    [device.uuid]: { position_x, position_y },
                                  }))
                                }
                              >
                                <Lock className="w-4 h-4 text-muted-foreground" />
                              </button>
                            )}
                            <button
                              type="button"
                              className="p-1 rounded hover:bg-accent"
                              aria-label="Device dashboard"
                              onClick={() => navigate(`/devices/${device.uuid}/dashboard`)}
                            >
                              <BarChart2 className="w-4 h-4 text-muted-foreground" />
                            </button>
                            <button
                              type="button"
                              className="p-1 rounded hover:bg-accent"
                              aria-label="Device info"
                              onClick={() => setDeviceInfoPopupDevice(device)}
                            >
                              <Info className="w-4 h-4 text-muted-foreground" />
                            </button>
                          </div>
                        )}
                        <div
                          className={`w-10 h-10 flex items-center justify-center rounded-lg select-none ${
                            !isPending && device.operation_status !== "online"
                              ? "ring-2 ring-destructive animate-pulse"
                              : ""
                          }`}
                          style={{
                            backgroundColor: device.icon?.color
                              ? `${device.icon.color}${isPending ? "40" : "20"}`
                              : "var(--muted)",
                            opacity: isPending ? 0.6 : 1,
                          }}
                          onMouseDown={
                            canDrag
                              ? (e) => {
                                  e.stopPropagation();
                                  handleDragStart(device.uuid, position_x, position_y, e);
                                }
                              : undefined
                          }
                        >
                          {device.icon?.iconify_id ? (
                            <Icon
                              icon={device.icon.iconify_id}
                              className="w-6 h-6"
                              style={{
                                color: device.icon.color ?? "var(--muted-foreground)",
                              }}
                            />
                          ) : device.device_type === "sensor" ? (
                            <BarChart3 className="w-6 h-6 text-muted-foreground" />
                          ) : (
                            <Cpu className="w-6 h-6 text-muted-foreground" />
                          )}
                        </div>
                        <DeviceIconStatusBar
                          deviceUuid={device.uuid}
                          deviceType={device.device_type}
                        />
                      </div>
                    );
                  })}
                </div>
              </div>
            </>
          ) : (
            <div className="flex flex-col items-center gap-2 text-muted-foreground py-16">
              <ImageOff className="w-12 h-12" />
              <span>No image saved</span>
            </div>
          )}
        </div>
      </div>

      <AlertDialog
        open={!!deviceInfoPopupDevice}
        onOpenChange={(open) => !open && setDeviceInfoPopupDevice(null)}
      >
        <AlertDialogContent className="max-w-md">
          <AlertDialogHeader>
            <AlertDialogTitle>Device Information</AlertDialogTitle>
            <AlertDialogDescription asChild>
              {deviceInfoPopupDevice && (
                <div className="space-y-3 pt-2 text-left">
                  <div>
                    <p className="font-semibold text-foreground mb-0.5">Name</p>
                    <p className="text-sm text-muted-foreground">{deviceInfoPopupDevice.name}</p>
                  </div>
                  {deviceInfoPopupDevice.description && (
                    <div>
                      <p className="font-semibold text-foreground mb-0.5">Description</p>
                      <p className="text-sm text-muted-foreground">
                        {deviceInfoPopupDevice.description}
                      </p>
                    </div>
                  )}
                  <div>
                    <p className="font-semibold text-foreground mb-0.5">Type</p>
                    <p className="text-sm text-muted-foreground capitalize">
                      {deviceInfoPopupDevice.device_type}
                    </p>
                  </div>
                  <div>
                    <p className="font-semibold text-foreground mb-0.5">Model</p>
                    <p className="text-sm text-muted-foreground">{deviceInfoPopupDevice.model}</p>
                  </div>
                  <div>
                    <p className="font-semibold text-foreground mb-0.5">MAC Address</p>
                    <p className="text-sm text-muted-foreground font-mono">
                      {deviceInfoPopupDevice.mac_address}
                    </p>
                  </div>
                  {deviceInfoPopupDevice.operation_status && (
                    <div>
                      <p className="font-semibold text-foreground mb-0.5">Status</p>
                      <p className="text-sm text-muted-foreground capitalize">
                        {deviceInfoPopupDevice.operation_status}
                      </p>
                    </div>
                  )}
                  {deviceInfoPopupDevice.sensor_type && (
                    <div>
                      <p className="font-semibold text-foreground mb-0.5">Sensor Type</p>
                      <p className="text-sm text-muted-foreground">
                        {deviceInfoPopupDevice.sensor_type}
                      </p>
                    </div>
                  )}
                  {deviceInfoPopupDevice.actuator_type && (
                    <div>
                      <p className="font-semibold text-foreground mb-0.5">Actuator Type</p>
                      <p className="text-sm text-muted-foreground">
                        {deviceInfoPopupDevice.actuator_type}
                      </p>
                    </div>
                  )}
                  {deviceInfoPopupDevice.device_type === "sensor" &&
                    deviceInfoPopupDevice.parameter_ranges &&
                    Object.keys(deviceInfoPopupDevice.parameter_ranges).length > 0 && (
                    <div>
                      <p className="font-semibold text-foreground mb-0.5">Reading ranges</p>
                      <ul className="text-sm text-muted-foreground list-none space-y-1">
                        {Object.entries(deviceInfoPopupDevice.parameter_ranges).map(
                          ([measurement, range]) => (
                            <li key={measurement} className="font-mono">
                              {measurement}: {range.min_reading}–{range.max_reading} {range.unit}
                            </li>
                          )
                        )}
                      </ul>
                    </div>
                  )}
                  {deviceInfoPopupDevice.device_type === "actuator" &&
                    deviceInfoPopupDevice.command_spec && (
                    <div>
                      <p className="font-semibold text-foreground mb-0.5">Command spec</p>
                      <p className="text-sm text-muted-foreground">
                        {deviceInfoPopupDevice.command_spec.type === "discrete"
                          ? `Commands: ${deviceInfoPopupDevice.command_spec.commands.join(", ")}`
                          : `Range: ${deviceInfoPopupDevice.command_spec.min}–${deviceInfoPopupDevice.command_spec.max} ${deviceInfoPopupDevice.command_spec.unit}`}
                      </p>
                    </div>
                  )}
                </div>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <Button onClick={() => setDeviceInfoPopupDevice(null)}>Close</Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Location</AlertDialogTitle>
            <AlertDialogDescription>
              Do you really want to delete the location "{location.name}"?
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              disabled={isDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isDeleting ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Deleting...
                </>
              ) : (
                "Continue"
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
    </>
  );
};

export default LocationDetail;

