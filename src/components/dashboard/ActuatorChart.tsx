import { useEffect, useState } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { deviceApi } from "@/services/deviceApi";
import { useAuth } from "@/context/AuthContext";

type ChartDataPoint = {
  time: string;
  timestamp: number;
  command: string;
  value: number;
  source: string;
};

type ActuatorChartProps = {
  deviceUuid: string;
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

export function ActuatorChart({ deviceUuid, period = "today" }: ActuatorChartProps) {
  const { token } = useAuth();
  const [data, setData] = useState<ChartDataPoint[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!token) return;

    const load = async () => {
      setLoading(true);
      const { start, end } = getPeriodDates(period);
      const result = await deviceApi.getDeviceCommandsForChart(token, {
        device_uuid: deviceUuid,
        start_date: start,
        end_date: end,
        limit: 500,
      });
      setLoading(false);

      if (!result.success || !result.data) return;

      const chartData: ChartDataPoint[] = result.data.map((cmd) => ({
        time: new Date(cmd.sent_at).toLocaleTimeString(undefined, {
          hour: "2-digit",
          minute: "2-digit",
        }),
        timestamp: new Date(cmd.sent_at).getTime(),
        command: cmd.command,
        value: cmd.command.toUpperCase() === "ON" ? 1 : 0,
        source: cmd.source ?? "manual",
      }));
      setData(chartData);
    };

    load();
  }, [token, deviceUuid, period]);

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
        No commands for this period
      </div>
    );
  }

  return (
    <div className="w-full h-64">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={data}>
          <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
          <XAxis
            dataKey="time"
            className="text-xs"
            tick={{ fill: "hsl(var(--muted-foreground))" }}
          />
          <YAxis
            domain={[0, 1]}
            ticks={[0, 1]}
            tickFormatter={(v) => (v === 1 ? "ON" : "OFF")}
            className="text-xs"
            tick={{ fill: "hsl(var(--muted-foreground))" }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: "hsl(var(--background))",
              border: "1px solid hsl(var(--border))",
              borderRadius: "var(--radius)",
            }}
            formatter={(value: number) => [value === 1 ? "ON" : "OFF", "Status"]}
            labelFormatter={(label) => `Time: ${label}`}
          />
          <Area
            type="stepAfter"
            dataKey="value"
            stroke="hsl(var(--primary))"
            fill="hsl(var(--primary) / 0.2)"
            strokeWidth={2}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
