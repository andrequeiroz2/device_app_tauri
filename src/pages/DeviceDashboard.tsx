import { useState, useEffect, useRef } from "react";
import { useParams, useNavigate, useLocation } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { deviceApi } from "@/services/deviceApi";
import { useAuth } from "@/context/AuthContext";
import { SENSOR_READING_LATEST_QUERY_KEY } from "@/components/dashboard/CurrentValueCard";
import { SENSOR_READINGS_QUERY_KEY } from "@/components/dashboard/SensorChart";
import { DEVICE_COMMANDS_CHART_QUERY_KEY } from "@/components/dashboard/ActuatorChart";
import { SensorChart } from "@/components/dashboard/SensorChart";
import { CurrentValueCard } from "@/components/dashboard/CurrentValueCard";
import { ActuatorChart } from "@/components/dashboard/ActuatorChart";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Icon } from "@iconify/react";
import { ArrowLeft, Loader2, Calendar, BarChart3, Cpu } from "lucide-react";
import { toast } from "sonner";
import type { DevicePublic } from "@/types/device";

type PeriodFilter = "today" | "7d" | "30d";

function parseDeviceScale(scale: unknown): [string, string][] | null {
  if (!scale) return null;
  if (Array.isArray(scale)) {
    return scale.filter(
      (item): item is [string, string] =>
        Array.isArray(item) &&
        item.length === 2 &&
        typeof item[0] === "string" &&
        typeof item[1] === "string"
    ) as [string, string][];
  }
  return null;
}

function SensorDashboard({
  device,
  period,
}: {
  device: DevicePublic;
  period: PeriodFilter;
}) {
  const deviceScale = parseDeviceScale(device.device_scale);
  if (!deviceScale || deviceScale.length === 0) {
    return (
      <div className="rounded-lg border border-dashed p-8 text-center text-muted-foreground">
        No measurement scales configured for this sensor.
      </div>
    );
  }

  const parameterRanges = device.parameter_ranges && Object.keys(device.parameter_ranges).length > 0;

  return (
    <div className="space-y-6">
      {parameterRanges && (
        <div className="rounded-lg border bg-muted/30 p-3 text-sm">
          <p className="font-medium text-foreground mb-1">Reading ranges</p>
          <ul className="text-muted-foreground list-none space-y-0.5">
            {Object.entries(device.parameter_ranges!).map(([measurement, range]) => (
              <li key={measurement} className="font-mono">
                {measurement}: {range.min_reading}–{range.max_reading} {range.unit}
              </li>
            ))}
          </ul>
        </div>
      )}
      <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
        {deviceScale.map(([measurement, scale]) => (
          <CurrentValueCard
            key={measurement}
            deviceUuid={device.uuid}
            measurement={measurement}
            scale={scale}
          />
        ))}
      </div>

      <div className="space-y-6">
        {deviceScale.map(([measurement, scale]) => (
          <div
            key={measurement}
            className="rounded-lg border bg-card p-4"
          >
            <h3 className="text-lg font-semibold mb-4 capitalize">
              {measurement}
            </h3>
            <SensorChart
              deviceUuid={device.uuid}
              measurement={measurement}
              scale={scale}
              period={period}
            />
          </div>
        ))}
      </div>
    </div>
  );
}

function ActuatorDashboard({
  device,
  period,
}: {
  device: DevicePublic;
  period: PeriodFilter;
}) {
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();
  const commandSpec = device.command_spec;
  const rangeMin = commandSpec?.type === "range" ? commandSpec.min : 0;
  const rangeMax = commandSpec?.type === "range" ? commandSpec.max : 100;
  const rangeUnit = commandSpec?.type === "range" ? commandSpec.unit : "";
  const [rangeValue, setRangeValue] = useState<number>(rangeMin);
  const [sending, setSending] = useState(false);

  const handleSendDiscrete = async (command: string) => {
    if (!token) return;
    setSending(true);
    try {
      const result = await deviceApi.sendDeviceCommand(
        token,
        device.uuid,
        JSON.stringify({ command })
      );
      if (result.unauthorized) {
        logout();
        return;
      }
      if (result.success) {
        toast.success(`Command "${command}" sent`);
        queryClient.invalidateQueries({
          queryKey: [...DEVICE_COMMANDS_CHART_QUERY_KEY, device.uuid],
        });
        queryClient.invalidateQueries({ queryKey: ["device", device.uuid] });
      } else {
        toast.error(result.message ?? "Failed to send command");
      }
    } finally {
      setSending(false);
    }
  };

  const handleSendRange = async () => {
    if (!token || commandSpec?.type !== "range") return;
    setSending(true);
    try {
      const result = await deviceApi.sendDeviceCommand(
        token,
        device.uuid,
        JSON.stringify({ command_payload: { value: rangeValue } })
      );
      if (result.unauthorized) {
        logout();
        return;
      }
      if (result.success) {
        toast.success(`Value ${rangeValue} ${rangeUnit} sent`);
        queryClient.invalidateQueries({
          queryKey: [...DEVICE_COMMANDS_CHART_QUERY_KEY, device.uuid],
        });
        queryClient.invalidateQueries({ queryKey: ["device", device.uuid] });
      } else {
        toast.error(result.message ?? "Failed to send command");
      }
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="space-y-6">
      {commandSpec && (
        <div className="rounded-lg border bg-muted/30 p-3 text-sm">
          <p className="font-medium text-foreground mb-1">Command spec</p>
          <p className="text-muted-foreground">
            {commandSpec.type === "discrete"
              ? `Commands: ${commandSpec.commands.join(", ")}`
              : `Range: ${commandSpec.min}–${commandSpec.max} ${commandSpec.unit}`}
          </p>
        </div>
      )}
      {commandSpec && (
        <div className="rounded-lg border bg-card p-4">
          <h3 className="text-lg font-semibold mb-4">Send command</h3>
          {commandSpec.type === "discrete" ? (
            <div className="flex flex-wrap gap-2">
              {commandSpec.commands.map((cmd) => (
                <Button
                  key={cmd}
                  variant="outline"
                  disabled={sending}
                  onClick={() => handleSendDiscrete(cmd)}
                >
                  {sending ? (
                    <Loader2 className="w-4 h-4 animate-spin mr-2" />
                  ) : null}
                  {cmd}
                </Button>
              ))}
            </div>
          ) : (
            <div className="space-y-4 max-w-xs">
              <div className="flex items-center gap-4">
                <input
                  type="range"
                  min={rangeMin}
                  max={rangeMax}
                  step={1}
                  value={rangeValue}
                  onChange={(e) => setRangeValue(Number(e.target.value))}
                  className="flex-1 h-2 rounded-lg appearance-none cursor-pointer bg-muted accent-primary"
                />
                <span className="text-sm font-mono tabular-nums shrink-0">
                  {rangeValue} {rangeUnit}
                </span>
              </div>
              <Button
                variant="default"
                disabled={sending}
                onClick={handleSendRange}
              >
                {sending ? (
                  <Loader2 className="w-4 h-4 animate-spin mr-2" />
                ) : null}
                Send value
              </Button>
            </div>
          )}
        </div>
      )}
      <div className="rounded-lg border bg-card p-4">
        <h3 className="text-lg font-semibold mb-4">Command History</h3>
        <ActuatorChart deviceUuid={device.uuid} period={period} />
      </div>
    </div>
  );
}

const DEVICE_DASHBOARD_UPDATE_EVENT = "device-dashboard-update";

const DEBOUNCE_MS = 500;

export default function DeviceDashboard() {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const location = useLocation();
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();
  const [period, setPeriod] = useState<PeriodFilter>("today");
  const pathnameRef = useRef(location.pathname);
  pathnameRef.current = location.pathname;

  // Real-time: invalidate queries when backend emits device-dashboard-update
  useEffect(() => {
    if (!token) return;

    const debounceTimers = new Map<string, ReturnType<typeof setTimeout>>();

    const unlistenPromise = listen<{ device_uuid: string }>(
      DEVICE_DASHBOARD_UPDATE_EVENT,
      (event) => {
        const deviceUuid = event.payload?.device_uuid;
        if (!deviceUuid) return;

        // 4.3: só invalidar se o usuário estiver na tela do dashboard deste device
        if (pathnameRef.current !== `/devices/${deviceUuid}/dashboard`) return;

        // 4.2: debounce para evitar refetch excessivo
        const existing = debounceTimers.get(deviceUuid);
        if (existing) clearTimeout(existing);

        const timer = setTimeout(() => {
          debounceTimers.delete(deviceUuid);
          queryClient.invalidateQueries({ queryKey: ["device", deviceUuid] });
          queryClient.invalidateQueries({
            queryKey: [...SENSOR_READING_LATEST_QUERY_KEY, deviceUuid],
          });
          queryClient.invalidateQueries({
            queryKey: [...SENSOR_READINGS_QUERY_KEY, deviceUuid],
          });
          queryClient.invalidateQueries({
            queryKey: [...DEVICE_COMMANDS_CHART_QUERY_KEY, deviceUuid],
          });
        }, DEBOUNCE_MS);

        debounceTimers.set(deviceUuid, timer);
      }
    );

    return () => {
      debounceTimers.forEach((t) => clearTimeout(t));
      debounceTimers.clear();
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [token, queryClient]);

  const {
    data: device,
    isLoading,
    error: queryError,
  } = useQuery({
    queryKey: ["device", uuid],
    queryFn: async () => {
      if (!token || !uuid) {
        logout();
        return null;
      }
      const result = await deviceApi.getDevice(token, uuid);
      if (!result.success) {
        if (result.unauthorized) {
          toast.error("Session expired. Please login again.");
          logout();
          return null;
        }
        throw new Error(result.message ?? "Failed to load device");
      }
      return result.data ?? null;
    },
    enabled: !!uuid && !!token,
    retry: false,
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[200px]">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (queryError || !device) {
    return (
      <div className="space-y-4">
        <p className="text-destructive">
          {queryError instanceof Error
            ? queryError.message
            : "Device not found"}
        </p>
        <Button variant="outline" size="sm" onClick={() => navigate(-1)}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back
        </Button>
      </div>
    );
  }

  const isSensor = device.device_type === "sensor";

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate("/")}
            aria-label="Back"
          >
            <ArrowLeft className="w-5 h-5" />
          </Button>
          {device.icon?.iconify_id ? (
            <div
              className="w-10 h-10 flex items-center justify-center rounded-lg shrink-0"
              style={{
                backgroundColor: device.icon.color
                  ? `${device.icon.color}20`
                  : "var(--muted)",
              }}
            >
              <Icon
                icon={device.icon.iconify_id}
                className="w-6 h-6"
                style={{ color: device.icon.color ?? undefined }}
              />
            </div>
          ) : (
            <div className="w-10 h-10 flex items-center justify-center rounded-lg bg-muted shrink-0">
              {isSensor ? (
                <BarChart3 className="w-6 h-6 text-muted-foreground" />
              ) : (
                <Cpu className="w-6 h-6 text-muted-foreground" />
              )}
            </div>
          )}
          <div>
            <h1 className="text-2xl font-semibold">{device.name}</h1>
            <p className="text-sm text-muted-foreground">
              {device.model} • {device.device_type}
            </p>
          </div>
        </div>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm" className="gap-2">
              <Calendar className="w-4 h-4" />
              {period === "today"
                ? "Today"
                : period === "7d"
                  ? "Last 7 days"
                  : "Last 30 days"}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onSelect={() => setPeriod("today")}>
              Today
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => setPeriod("7d")}>
              Last 7 days
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => setPeriod("30d")}>
              Last 30 days
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {isSensor ? (
        <SensorDashboard device={device} period={period} />
      ) : (
        <ActuatorDashboard device={device} period={period} />
      )}
    </div>
  );
}
