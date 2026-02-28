import { useEffect, useState } from "react";
import { sensorReadingApi } from "@/services/sensorReadingApi";
import { useAuth } from "@/context/AuthContext";

type CurrentValueCardProps = {
  deviceUuid: string;
  measurement: string;
  scale: string;
};

export function CurrentValueCard({
  deviceUuid,
  measurement,
  scale,
}: CurrentValueCardProps) {
  const { token } = useAuth();
  const [value, setValue] = useState<number | null>(null);
  const [recordedAt, setRecordedAt] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!token) return;

    const load = async () => {
      setLoading(true);
      const result = await sensorReadingApi.getSensorReadingLatest(
        token,
        deviceUuid,
        measurement
      );
      setLoading(false);

      if (result.success && result.data) {
        setValue(result.data.value);
        setRecordedAt(result.data.recorded_at);
      }
    };

    load();
  }, [token, deviceUuid, measurement]);

  const label = measurement.charAt(0).toUpperCase() + measurement.slice(1);

  return (
    <div className="rounded-lg border bg-card p-4">
      <p className="text-sm font-medium text-muted-foreground">{label}</p>
      {loading ? (
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
