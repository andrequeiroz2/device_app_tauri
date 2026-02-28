import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { deviceApi } from "@/services/deviceApi";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
import { Loader2, BarChart3, Cpu } from "lucide-react";
import { toast } from "sonner";

export default function DevicesList() {
  const { token, logout } = useAuth();

  const { data, isLoading, error } = useQuery({
    queryKey: ["devices"],
    queryFn: async () => {
      if (!token) {
        logout();
        return null;
      }
      const result = await deviceApi.listDevices(token, {
        page: 1,
        page_size: 50,
      });
      if (!result.success) {
        if (result.unauthorized) {
          toast.error("Session expired. Please login again.");
          logout();
          return null;
        }
        throw new Error(result.message ?? "Failed to load devices");
      }
      return result.data ?? { items: [], total: 0, page: 1, page_size: 50 };
    },
    enabled: !!token,
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[200px]">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-4">
        <p className="text-destructive">
          {error instanceof Error ? error.message : "Failed to load devices"}
        </p>
      </div>
    );
  }

  const devices = data?.items ?? [];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Devices</h1>
        <p className="text-muted-foreground">
          Select a device to view its dashboard
        </p>
      </div>

      {devices.length === 0 ? (
        <div className="rounded-lg border border-dashed p-8 text-center text-muted-foreground">
          No devices found. Adopt a device to get started.
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {devices.map((device) => (
            <Link
              key={device.uuid}
              to={`/devices/${device.uuid}/dashboard`}
              className="block"
            >
              <div className="rounded-lg border bg-card p-4 hover:bg-accent/50 transition-colors">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-2">
                    {device.device_type === "sensor" ? (
                      <BarChart3 className="w-5 h-5 text-muted-foreground" />
                    ) : (
                      <Cpu className="w-5 h-5 text-muted-foreground" />
                    )}
                    <h3 className="font-semibold">{device.name}</h3>
                  </div>
                </div>
                <p className="text-sm text-muted-foreground mt-1">
                  {device.model} • {device.device_type}
                </p>
                <p className="text-xs text-muted-foreground mt-2">
                  Dashboard →
                </p>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
