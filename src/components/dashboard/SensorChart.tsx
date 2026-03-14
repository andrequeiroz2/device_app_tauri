import { useQuery } from "@tanstack/react-query";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { sensorReadingApi } from "@/services/sensorReadingApi";
import { useAuth } from "@/context/AuthContext";

export type ChartDataPoint = {
  time: string;
  value: number;
};

export const SENSOR_READINGS_QUERY_KEY = ["sensor-readings"] as const;

type SensorChartProps = {
  deviceUuid: string;
  measurement: string;
  scale: string;
  period?: "today" | "7d" | "30d";
};

function getPeriodDates(period: "today" | "7d" | "30d"): { start: string; end: string } {
  const now = new Date();
  const end = now.toISOString();
  let start: Date;

  switch (period) {
    case "today":
      start = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      break;
    case "7d":
      start = new Date(now);
      start.setDate(start.getDate() - 7);
      break;
    case "30d":
      start = new Date(now);
      start.setDate(start.getDate() - 30);
      break;
    default:
      start = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  }

  return {
    start: start.toISOString(),
    end,
  };
}

export function SensorChart({
  deviceUuid,
  measurement,
  scale,
  period = "today",
}: SensorChartProps) {
  const { token, logout } = useAuth();

  const { data = [], isLoading } = useQuery({
    queryKey: [...SENSOR_READINGS_QUERY_KEY, deviceUuid, measurement, period],
    queryFn: async () => {
      if (!token) {
        logout();
        return [];
      }
      const { start, end } = getPeriodDates(period);
      const result = await sensorReadingApi.listSensorReadings(token, {
        device_uuid: deviceUuid,
        measurement,
        start_date: start,
        end_date: end,
        limit: 500,
      });
      if (result.unauthorized) logout();
      if (!result.success || !result.data) return [];
      return result.data.map((r): ChartDataPoint => ({
        time: new Date(r.recorded_at).toLocaleTimeString(undefined, {
          hour: "2-digit",
          minute: "2-digit",
        }),
        value: r.value,
      }));
    },
    enabled: !!token && !!deviceUuid && !!measurement,
  });

  if (isLoading) {
    return (
      <div className="h-64 flex items-center justify-center text-muted-foreground">
        Loading chart...
      </div>
    );
  }

  if (data.length === 0) {
    return (
      <div className="h-64 flex items-center justify-center text-muted-foreground border rounded-lg border-dashed">
        No data for this period
      </div>
    );
  }

  return (
    <div className="w-full h-64">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
          <XAxis
            dataKey="time"
            className="text-xs"
            tick={{ fill: "hsl(var(--muted-foreground))" }}
          />
          <YAxis
            unit={` ${scale}`}
            className="text-xs"
            tick={{ fill: "hsl(var(--muted-foreground))" }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: "hsl(var(--background))",
              border: "1px solid hsl(var(--border))",
              borderRadius: "var(--radius)",
            }}
            formatter={(value: number) => [`${value} ${scale}`, measurement]}
          />
          <Line
            type="monotone"
            dataKey="value"
            stroke="hsl(var(--primary))"
            strokeWidth={2}
            dot={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
