import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { deviceApi } from "@/services/deviceApi";
import { useAuth } from "@/context/AuthContext";
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

  return (
    <div className="space-y-6">
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
  return (
    <div className="space-y-6">
      <div className="rounded-lg border bg-card p-4">
        <h3 className="text-lg font-semibold mb-4">Command History</h3>
        <ActuatorChart deviceUuid={device.uuid} period={period} />
      </div>
    </div>
  );
}

export default function DeviceDashboard() {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { token, logout } = useAuth();
  const [period, setPeriod] = useState<PeriodFilter>("today");

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
