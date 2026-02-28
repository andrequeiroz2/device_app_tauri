import { useEffect, useState } from "react";
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
  const { token } = useAuth();
  const [data, setData] = useState<ChartDataPoint[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!token) return;

    const load = async () => {
      setLoading(true);
      const { start, end } = getPeriodDates(period);
      const result = await sensorReadingApi.listSensorReadings(token, {
        device_uuid: deviceUuid,
        measurement,
        start_date: start,
        end_date: end,
        limit: 500,
      });
      setLoading(false);

      if (!result.success || !result.data) return;

      const chartData: ChartDataPoint[] = result.data.map((r) => ({
        time: new Date(r.recorded_at).toLocaleTimeString(undefined, {
          hour: "2-digit",
          minute: "2-digit",
        }),
        value: r.value,
      }));
      setData(chartData);
    };

    load();
  }, [token, deviceUuid, measurement, period]);

  if (loading) {
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
