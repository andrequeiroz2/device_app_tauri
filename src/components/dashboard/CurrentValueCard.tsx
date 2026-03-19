import { useQuery } from "@tanstack/react-query";
import { sensorReadingApi } from "@/services/sensorReadingApi";
import { useAuth } from "@/context/AuthContext";

export const SENSOR_READING_LATEST_QUERY_KEY = ["sensor-reading-latest"] as const;

type CurrentValueCardProps = {
  deviceUuid: string;
  measurement: string;
  scale: string;
  label?: string;
};

export function CurrentValueCard({
  deviceUuid,
  measurement,
  scale,
  label,
}: CurrentValueCardProps) {
  const { token, logout } = useAuth();

  const { data, isLoading } = useQuery({
    queryKey: [...SENSOR_READING_LATEST_QUERY_KEY, deviceUuid, measurement],
    queryFn: async () => {
      if (!token) {
        logout();
        return null;
      }
      const result = await sensorReadingApi.getSensorReadingLatest(
        token,
        deviceUuid,
        measurement
      );
      if (result.unauthorized) logout();
      return result.success ? result.data ?? null : null;
    },
    enabled: !!token && !!deviceUuid && !!measurement,
  });

  const value = data?.value ?? null;
  const recordedAt = data?.recorded_at ?? null;
  const defaultLabel = measurement.charAt(0).toUpperCase() + measurement.slice(1);
  const finalLabel = label ?? defaultLabel;

  return (
    <div className="rounded-lg border bg-card p-4">
      <p className="text-sm font-medium text-muted-foreground">{finalLabel}</p>
      {isLoading ? (
        <p className="text-2xl font-bold mt-1">...</p>
      ) : value !== null ? (
        <>
          <p className="text-2xl font-bold mt-1">
            {value} <span className="text-lg text-muted-foreground">{scale}</span>
          </p>
          {recordedAt && (
            <p className="text-xs text-muted-foreground mt-2">
              {new Date(recordedAt).toLocaleString()}
            </p>
          )}
        </>
      ) : (
        <p className="text-muted-foreground mt-1">No data</p>
      )}
    </div>
  );
}
