import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { deviceApi } from "@/services/deviceApi";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
import { Loader2, BarChart3, Cpu, ArrowLeft } from "lucide-react";
import { toast } from "sonner";

export default function DevicesList() {
  const { token, logout } = useAuth();
  const navigate = useNavigate();

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

  const devices = data?.items ?? [];

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] text-muted-foreground gap-2">
        <p className="font-semibold text-destructive">
          {error instanceof Error ? error.message : "Failed to load devices"}
        </p>
        <Button onClick={() => navigate("/")} variant="outline" size="sm">
          Back to Home
        </Button>
      </div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
      className="flex flex-col h-[calc(100vh-120px)]"
    >
      <div className="flex items-center justify-between shrink-0 pb-4">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate("/")}
            aria-label="Back"
          >
            <ArrowLeft className="w-5 h-5" />
          </Button>
          <div>
            <h1 className="text-2xl font-semibold">Devices</h1>
            <p className="text-muted-foreground text-sm">
              Select a device to view its dashboard.
            </p>
          </div>
        </div>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center flex-1">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      ) : devices.length === 0 ? (
        <div className="bg-background border border-border rounded-xl p-12 text-center flex-1 flex flex-col items-center justify-center">
          <Cpu className="w-12 h-12 mb-4 text-muted-foreground" />
          <p className="text-muted-foreground mb-4">No devices found.</p>
          <Button asChild variant="outline" size="sm">
            <Link to="/locations/list">Adopt a device to get started</Link>
          </Button>
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto border border-border rounded-xl">
          <div className="space-y-2 p-4">
            {devices.map((device) => (
              <Link
                key={device.uuid}
                to={`/devices/${device.uuid}/dashboard`}
                className="block rounded-lg border p-4 transition-colors hover:bg-muted/50"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      {device.device_type === "sensor" ? (
                        <BarChart3 className="w-5 h-5 text-muted-foreground shrink-0" />
                      ) : (
                        <Cpu className="w-5 h-5 text-muted-foreground shrink-0" />
                      )}
                      <span className="font-medium">{device.name}</span>
                    </div>
                    <p className="mt-1 text-sm text-muted-foreground">
                      {device.model} • {device.device_type}
                    </p>
                    <p className="mt-0.5 text-xs text-muted-foreground">
                      Dashboard →
                    </p>
                  </div>
                </div>
              </Link>
            ))}
          </div>
        </div>
      )}
    </motion.div>
  );
}
