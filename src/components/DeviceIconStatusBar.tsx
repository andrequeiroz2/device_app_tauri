import { useQuery } from "@tanstack/react-query";
import { sensorReadingApi } from "@/services/sensorReadingApi";
import { useAuth } from "@/context/AuthContext";
import type { DeviceType } from "@/types/device";
import type { SensorReadingLatest } from "@/types/sensorReading";

export const SENSOR_READING_LATEST_ALL_QUERY_KEY = [
  "sensor-reading-latest-all",
] as const;

const MAX_MEASUREMENTS = 3;
const STALE_TIME_MS = 10_000; // 4.3: reduz refetches em background quando há muitos devices

function formatReading(r: SensorReadingLatest): string {
  const value = Number.isInteger(r.value) ? String(r.value) : r.value.toFixed(1);
  return r.scale ? `${value}${r.scale}` : value;
}

type DeviceIconStatusBarProps = {
  deviceUuid: string;
  deviceType: DeviceType;
};

export function DeviceIconStatusBar({
  deviceUuid,
  deviceType,
}: DeviceIconStatusBarProps) {
  const { token, logout } = useAuth();

  const { data } = useQuery({
    queryKey: [...SENSOR_READING_LATEST_ALL_QUERY_KEY, deviceUuid],
    queryFn: async () => {
      if (!token) {
        logout();
        return [];
      }
      const result = await sensorReadingApi.getSensorReadingLatestAll(
        token,
        deviceUuid
      );
      if (result.unauthorized) logout();
      return result.success ? result.data ?? [] : [];
    },
    enabled: !!token && !!deviceUuid && deviceType === "sensor",
    staleTime: STALE_TIME_MS,
  });

  const readings = data ?? [];

  if (readings.length === 0) {
    return null;
  }

  const display = readings
    .slice(0, MAX_MEASUREMENTS)
    .map(formatReading)
    .join("  ");

  return (
    <div
      className="mt-0.5 px-1.5 py-0.5 rounded text-[10px] font-medium text-foreground/90 bg-background/70 backdrop-blur-sm border border-border/50 overflow-hidden text-ellipsis whitespace-nowrap max-w-[100px]"
      title={display}
    >
      {display}
    </div>
  );
}
