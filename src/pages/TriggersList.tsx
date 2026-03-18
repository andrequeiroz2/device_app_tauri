import { useEffect, useMemo, useRef, useState } from "react";
import { useInfiniteQuery, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useLocation, useNavigate, useParams } from "react-router-dom";
import { triggerApi } from "@/services/triggerApi";
import { deviceApi } from "@/services/deviceApi";
import { locationApi } from "@/services/locationApi";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
import { DeviceInformationDialog } from "@/components/DeviceInformationDialog";
import {
  Loader2,
  ArrowLeft,
  Zap,
  Pencil,
  Trash2,
  Send,
  PanelRight,
  X,
  Filter,
  ChevronDown,
  Info,
} from "lucide-react";
import { toast } from "sonner";
import type { TriggerPublic, TriggerFilter } from "@/types/trigger";
import type { DevicePublic } from "@/types/device";
import type { LocationPublic } from "@/types/location";
import { cn } from "@/lib/utils";
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
const PAGE_SIZE = 20;

export default function TriggersList() {
  const { token, logout } = useAuth();
  const navigate = useNavigate();
  const { deviceUuid } = useParams<{ deviceUuid?: string }>();
  const isDeviceRoute = !!deviceUuid;
  const routerLocation = useLocation();
  const fromLocationUuid = (routerLocation.state as { fromLocationUuid?: string } | null)
    ?.fromLocationUuid;
  const handleBack = () => {
    if (fromLocationUuid) {
      navigate(`/locations/${fromLocationUuid}`);
      return;
    }
    navigate("/");
  };
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<TriggerFilter>({});
  const [locationUuid, setLocationUuid] = useState<string>("all");
  const [isPanelOpen, setIsPanelOpen] = useState(false);
  const [isFilterExpanded, setIsFilterExpanded] = useState(true);
  const [deleteTrigger, setDeleteTrigger] = useState<TriggerPublic | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [testingUuid, setTestingUuid] = useState<string | null>(null);
  const [deviceInfoPopupDevice, setDeviceInfoPopupDevice] = useState<DevicePublic | null>(null);

  const { data: devicesData } = useQuery({
    queryKey: ["devices", locationUuid],
    queryFn: async () => {
      if (!token) return { items: [] as DevicePublic[], total: 0 };
      const r = await deviceApi.listDevices(token, {
        page: 1,
        page_size: 200,
        filter: locationUuid === "all" ? {} : { location_uuid: locationUuid },
      });
      return r.success && r.data ? r.data : { items: [], total: 0 };
    },
    enabled: !!token,
  });
  const devices = devicesData?.items ?? [];

  const { data: locationsData } = useQuery({
    queryKey: ["locations", "triggers-filter"],
    queryFn: async () => {
      if (!token) return { items: [] as LocationPublic[], total: 0, page: 1, page_size: 200 };
      const r = await locationApi.listLocations(token, 1, 200, { status: "all" });
      return r.success && r.data ? r.data : { items: [], total: 0, page: 1, page_size: 200 };
    },
    enabled: !!token,
  });
  const locations = locationsData?.items ?? [];

  const { data: deviceContext } = useQuery({
    queryKey: ["device", deviceUuid],
    queryFn: async () => {
      if (!token || !deviceUuid) return null;
      const r = await deviceApi.getDevice(token, deviceUuid);
      return r.success && r.data ? r.data : null;
    },
    enabled: !!token && !!deviceUuid,
  });

  const loadMoreRef = useRef<HTMLDivElement | null>(null);

  const {
    data,
    isLoading,
    error,
    hasNextPage,
    fetchNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ["triggers-list", filter],
    initialPageParam: 1,
    queryFn: async ({ pageParam }) => {
      if (!token) {
        logout();
        return null;
      }
      const result = await triggerApi.listTriggers(token, {
        page: Number(pageParam) || 1,
        page_size: PAGE_SIZE,
        filter,
      });
      if (!result.success) {
        if (result.unauthorized) {
          toast.error("Session expired. Please login again.");
          logout();
          return null;
        }
        throw new Error(result.message ?? "Failed to load triggers");
      }
      return result.data ?? { items: [], total: 0, page: 1, page_size: PAGE_SIZE };
    },
    getNextPageParam: (lastPage) => {
      if (!lastPage) return undefined;
      const { page, page_size, total } = lastPage;
      const loaded = page * page_size;
      return loaded < total ? page + 1 : undefined;
    },
    enabled: !!token,
    retry: false,
  });

  const triggers = useMemo<TriggerPublic[]>(() => {
    if (!data?.pages) return [];
    return data.pages.flatMap((p) => p?.items ?? []);
  }, [data]);
  useEffect(() => {
    if (!deviceUuid) return;
    setFilter((prev) => ({
      ...prev,
      device_uuid: deviceUuid,
    }));
    setLocationUuid("all");
  }, [deviceUuid]);

  const visibleTriggers = useMemo(() => {
    if (isDeviceRoute) return triggers;
    if (locationUuid === "all") return triggers;
    const allowedDevices = new Set(devices.map((d) => d.uuid));
    return triggers.filter((t) => t.device_uuid && allowedDevices.has(t.device_uuid));
  }, [triggers, devices, locationUuid, isDeviceRoute]);
  const hasActiveFilters =
    filter.is_active === true || !!filter.device_uuid || locationUuid !== "all" || isDeviceRoute;

  useEffect(() => {
    if (!loadMoreRef.current) return;
    if (!hasNextPage) return;
    const el = loadMoreRef.current;

    const observer = new IntersectionObserver(
      (entries) => {
        const first = entries[0];
        if (first?.isIntersecting && hasNextPage && !isFetchingNextPage) {
          fetchNextPage();
        }
      },
      { root: null, rootMargin: "200px", threshold: 0.01 }
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [fetchNextPage, hasNextPage, isFetchingNextPage, visibleTriggers.length]);

  const handleDelete = async () => {
    if (!token || !deleteTrigger) return;
    setIsDeleting(true);
    const result = await triggerApi.deleteTrigger(token, deleteTrigger.uuid);
    setIsDeleting(false);
    setDeleteTrigger(null);
    if (result.unauthorized) {
      logout();
      return;
    }
    if (result.success) {
      toast.success("Trigger deleted.");
      queryClient.invalidateQueries({ queryKey: ["triggers-list"] });
    } else {
      toast.error(result.message ?? "Failed to delete trigger");
    }
  };

  const handleTest = async (t: TriggerPublic) => {
    if (!token) return;
    if (t.action_type !== "discord" && t.action_type !== "telegram") {
      toast.info("Test is only available for Discord and Telegram triggers.");
      return;
    }
    setTestingUuid(t.uuid);
    const result = await triggerApi.sendTest(token, t.uuid);
    setTestingUuid(null);
    if (result.unauthorized) {
      logout();
      return;
    }
    if (result.success) {
      toast.success("Test notification sent.");
    } else {
      toast.error(result.message ?? "Failed to send test");
    }
  };

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] text-muted-foreground gap-2">
        <p className="font-semibold text-destructive">
          {error instanceof Error ? error.message : "Failed to load triggers"}
        </p>
        <Button onClick={handleBack} variant="outline" size="sm">
          Back to Home
        </Button>
      </div>
    );
  }

  return (
    <>
      <div className="flex items-center justify-between shrink-0 pb-4">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={handleBack}
            aria-label="Back"
            title={fromLocationUuid ? "Back to location" : "Back to Home"}
          >
            <ArrowLeft className="w-5 h-5" />
          </Button>
          <div>
            <h1 className="text-2xl font-semibold">Triggers</h1>
            <p className="text-muted-foreground text-sm">
              {isDeviceRoute && deviceContext ? (
                <span className="flex items-center gap-2">
                  <span>{`Device: ${deviceContext.name}`}</span>
                  <button
                    type="button"
                    className="p-1 rounded hover:bg-accent"
                    aria-label="Device info"
                    title="Device info"
                    onClick={() => setDeviceInfoPopupDevice(deviceContext)}
                  >
                    <Info className="w-4 h-4 text-muted-foreground" />
                  </button>
                </span>
              ) : (
                "Automate notifications and device commands."
              )}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setIsPanelOpen(true)}
            className="flex items-center gap-2"
          >
            <PanelRight className="w-4 h-4" />
            Panel
          </Button>
        </div>
      </div>

      {!isDeviceRoute && (
        <div className="pb-4">
          <div className="grid gap-3 max-w-[260px]">
            <div className="flex flex-col gap-1">
              <span className="text-xs font-medium text-muted-foreground">Location</span>
              <div className="relative">
                <select
                  className="h-10 w-full appearance-none rounded-lg border border-border bg-background pr-9 pl-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                  value={locationUuid}
                  onChange={(e) => {
                    setLocationUuid(e.target.value);
                    setFilter((f) => ({ ...f, device_uuid: undefined }));
                  }}
                >
                  <option value="all">All locations</option>
                  {locations.map((loc) => (
                    <option key={loc.uuid} value={loc.uuid}>
                      {loc.name}
                    </option>
                  ))}
                </select>
                <ChevronDown className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              </div>
            </div>

            <div className="flex flex-col gap-1">
              <span className="text-xs font-medium text-muted-foreground">Device</span>
              <div className="relative">
                <select
                  className="h-10 w-full appearance-none rounded-lg border border-border bg-background pr-9 pl-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                  value={filter.device_uuid ?? "all"}
                  onChange={(e) =>
                    setFilter((f) => ({
                      ...f,
                      device_uuid: e.target.value === "all" ? undefined : e.target.value,
                    }))
                  }
                >
                  <option value="all">All devices</option>
                  {devices.map((d) => (
                    <option key={d.uuid} value={d.uuid}>
                      {d.name}
                    </option>
                  ))}
                </select>
                <ChevronDown className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              </div>
            </div>
          </div>
        </div>
      )}

      {isPanelOpen && (
        <div
          className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm"
          onClick={() => setIsPanelOpen(false)}
        />
      )}

      <div
        className={cn(
          "fixed top-0 right-0 z-[60] h-full w-80 bg-background border-l border-border shadow-lg transition-transform duration-300 ease-in-out",
          isPanelOpen ? "translate-x-0" : "translate-x-full"
        )}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex flex-col h-full">
          <div className="flex items-center justify-between p-4 border-b border-border">
            <div className="flex items-center gap-2">
              <PanelRight className="w-5 h-5" />
              <h2 className="text-lg font-semibold">Panel</h2>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setIsPanelOpen(false)}
              className="h-8 w-8"
            >
              <X className="w-4 h-4" />
            </Button>
          </div>

          <div className="flex-1 p-4 space-y-4 overflow-y-auto">
            <Button asChild variant="outline" size="sm" className="w-full justify-center">
              <Link to="/triggers/create">Create trigger</Link>
            </Button>

            <div className="space-y-2">
              <button
                type="button"
                onClick={() => setIsFilterExpanded(!isFilterExpanded)}
                className="w-full flex items-center justify-between p-3 rounded-lg border border-border hover:bg-accent transition-colors"
              >
                <div className="flex items-center gap-2">
                  <Filter className="w-4 h-4" />
                  <span className="text-sm font-medium text-foreground">Filter</span>
                  {hasActiveFilters && <span className="w-2 h-2 bg-primary rounded-full" />}
                </div>
                <div
                  className={cn(
                    "transition-transform duration-200",
                    isFilterExpanded && "rotate-180"
                  )}
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M19 9l-7 7-7-7"
                    />
                  </svg>
                </div>
              </button>

              {isFilterExpanded && (
                <div className="mt-2 space-y-4 pl-2" role="radiogroup" aria-label="Filter by status">
                  <button
                    type="button"
                    role="radio"
                    aria-checked={filter.is_active === true}
                    onClick={() =>
                      setFilter((f) => ({
                        ...f,
                        is_active: true,
                      }))
                    }
                    className={cn(
                      "w-full text-left px-4 py-3 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2",
                      filter.is_active === true
                        ? "bg-primary text-primary-foreground border-primary"
                        : "bg-card hover:bg-accent border-border"
                    )}
                  >
                    <div className="flex items-center gap-2">
                      <div
                        className={cn(
                          "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                          filter.is_active === true
                            ? "border-primary-foreground bg-primary-foreground"
                            : "border-muted-foreground"
                        )}
                      >
                        {filter.is_active === true && (
                          <div className="w-2 h-2 rounded-full bg-primary" />
                        )}
                      </div>
                      <div>
                        <div className="font-medium">Active</div>
                        <div className="text-xs opacity-80 mt-1">Show only active triggers</div>
                      </div>
                    </div>
                  </button>
                  <button
                    type="button"
                    role="radio"
                    aria-checked={filter.is_active === undefined}
                    onClick={() =>
                      setFilter((f) => ({
                        ...f,
                        is_active: undefined,
                      }))
                    }
                    className={cn(
                      "w-full text-left px-4 py-3 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2",
                      filter.is_active === undefined
                        ? "bg-primary text-primary-foreground border-primary"
                        : "bg-card hover:bg-accent border-border"
                    )}
                  >
                    <div className="flex items-center gap-2">
                      <div
                        className={cn(
                          "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                          filter.is_active === undefined
                            ? "border-primary-foreground bg-primary-foreground"
                            : "border-muted-foreground"
                        )}
                      >
                        {filter.is_active === undefined && (
                          <div className="w-2 h-2 rounded-full bg-primary" />
                        )}
                      </div>
                      <div>
                        <div className="font-medium">All</div>
                        <div className="text-xs opacity-80 mt-1">Show all triggers</div>
                      </div>
                    </div>
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      ) : visibleTriggers.length === 0 ? (
        <div className="bg-background border border-border rounded-xl p-12 text-center">
          <Zap className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
          <p className="text-muted-foreground mb-4">
            {isDeviceRoute
              ? `No triggers for ${deviceContext?.name ?? "this device"}.`
              : "No triggers found."}
          </p>
          <Button asChild variant="outline" size="sm">
            <Link to="/triggers/create">
              {isDeviceRoute ? "Create your first trigger for this device" : "Create your first trigger"}
            </Link>
          </Button>
        </div>
      ) : (
        <div className="rounded-xl border border-border overflow-hidden bg-background max-h-[70vh] overflow-y-auto">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="sticky top-0 z-10 bg-muted/60 border-b border-border">
                <tr>
                  <th className="text-left p-3 font-medium">Name</th>
                  <th className="text-left p-3 font-medium">Event</th>
                  <th className="text-left p-3 font-medium">Action</th>
                  {!isDeviceRoute && <th className="text-left p-3 font-medium">Device</th>}
                  <th className="text-left p-3 font-medium">Status</th>
                  <th className="text-right p-3 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {visibleTriggers.map((t) => (
                  <tr key={t.uuid} className="border-t border-border">
                    <td className="p-3 font-medium">{t.name}</td>
                    <td className="p-3 text-muted-foreground">{t.source_event}</td>
                    <td className="p-3 text-muted-foreground">{t.action_type}</td>
                    {!isDeviceRoute && (
                      <td className="p-3 text-muted-foreground">
                        {t.device_uuid
                          ? devices.find((d) => d.uuid === t.device_uuid)?.name ?? t.device_uuid
                          : "—"}
                      </td>
                    )}
                    <td className="p-3">
                      <span
                        className={
                          t.is_active
                            ? "text-green-600 dark:text-green-400"
                            : "text-muted-foreground"
                        }
                      >
                        {t.is_active ? "Active" : "Inactive"}
                      </span>
                    </td>
                    <td className="p-3 text-right">
                      <div className="flex items-center justify-end gap-1">
                        {(t.action_type === "discord" || t.action_type === "telegram") && (
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => handleTest(t)}
                            disabled={testingUuid !== null}
                            title="Send test notification"
                          >
                            {testingUuid === t.uuid ? (
                              <Loader2 className="w-4 h-4 animate-spin" />
                            ) : (
                              <Send className="w-4 h-4" />
                            )}
                          </Button>
                        )}
                        <Button variant="ghost" size="icon" asChild title="Edit">
                          <Link to={`/triggers/${t.uuid}/edit`}>
                            <Pencil className="w-4 h-4" />
                          </Link>
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => setDeleteTrigger(t)}
                          title="Delete"
                        >
                          <Trash2 className="w-4 h-4 text-destructive" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div ref={loadMoreRef} className="h-10 flex items-center justify-center border-t border-border">
            {isFetchingNextPage ? (
              <div className="flex items-center gap-2 text-sm text-muted-foreground py-3">
                <Loader2 className="w-4 h-4 animate-spin" />
                Loading...
              </div>
            ) : hasNextPage ? (
              <div className="text-xs text-muted-foreground py-3">Scroll to load more</div>
            ) : (
              <div className="text-xs text-muted-foreground py-3">End of list</div>
            )}
          </div>
        </div>
      )}

      <DeviceInformationDialog
        open={!!deviceInfoPopupDevice}
        device={deviceInfoPopupDevice}
        onOpenChange={(nextOpen) => {
          if (!nextOpen) setDeviceInfoPopupDevice(null);
        }}
      />

      <AlertDialog open={!!deleteTrigger} onOpenChange={(open) => !open && setDeleteTrigger(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete trigger</AlertDialogTitle>
            <AlertDialogDescription>
              Delete &quot;{deleteTrigger?.name}&quot;? This cannot be undone.
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
                "Delete"
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
