import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { mqttBrokerApi } from "@/services/mqttBrokerApi";
import { useAuth } from "@/context/AuthContext";
import { useCollectorConnection } from "@/hooks/useCollectorConnection";
import { ConnectDisconnectBrokerButton } from "@/components/ConnectDisconnectBrokerButton";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Loader2, Wifi, WifiOff, AlertCircle } from "lucide-react";
import { toast } from "sonner";
import { MqttBrokerActionsPanel } from "@/components/MqttBrokerActionsPanel";

const MqttBrokerDetail = () => {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();
  const { connectedBrokerUuid, connectBroker, disconnectBroker } =
    useCollectorConnection();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const { data: broker, isLoading, error: queryError } = useQuery({
    queryKey: ["mqtt-broker", uuid],
    queryFn: async () => {
      if (!token || !uuid) {
        logout();
        return null;
      }
      const result = await mqttBrokerApi.getMqttBroker(token, uuid);
      if (!result.success) {
        if (result.unauthorized) {
          toast.error("Session expired. Please login again.");
          logout();
          return null;
        }
        const errorMsg = result.message ?? "Failed to load broker.";
        console.error("getMqttBroker error:", errorMsg);
        throw new Error(errorMsg);
      }
      if (!result.data) {
        console.error("getMqttBroker: no data returned");
        return null;
      }
      return result.data;
    },
    enabled: !!uuid && !!token,
    retry: false,
  });

  const handleDelete = async () => {
    if (!token || !uuid) return;
    setIsDeleting(true);

    const result = await mqttBrokerApi.deleteMqttBroker(token, uuid);
    setIsDeleting(false);
    setDeleteDialogOpen(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to delete broker.");
      return;
    }

    toast.success("Broker deleted successfully.");
    queryClient.invalidateQueries({ queryKey: ["mqtt-brokers-list"] });
    navigate("/mqtt-brokers/list");
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (queryError) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] text-muted-foreground gap-2">
        <p className="font-semibold">Error loading broker</p>
        <p className="text-sm">{queryError.message}</p>
        <Button onClick={() => navigate("/mqtt-brokers/list")} variant="outline">
          Back to List
        </Button>
      </div>
    );
  }

  if (!broker) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] text-muted-foreground gap-2">
        <p>Broker not found.</p>
        <Button onClick={() => navigate("/mqtt-brokers/list")} variant="outline">
          Back to List
        </Button>
      </div>
    );
  }

  const isInactive = !broker.is_active;

  return (
    <>
      <div className="space-y-4">
        {isInactive && (
          <div className="border border-yellow-500/50 bg-yellow-500/10 rounded-lg p-4 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <AlertCircle className="w-5 h-5 text-yellow-600 dark:text-yellow-500" />
              <div>
                <p className="font-semibold text-yellow-900 dark:text-yellow-100">
                  Broker Inactive
                </p>
                <p className="text-sm text-yellow-700 dark:text-yellow-300">
                  This broker is currently inactive.
                </p>
              </div>
            </div>
          </div>
        )}

        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-semibold">{broker.name}</h1>
            {broker.is_active && (
              <ConnectDisconnectBrokerButton
                brokerUuid={broker.uuid}
                isActive={broker.is_active}
                isConnected={connectedBrokerUuid === broker.uuid}
                onConnect={connectBroker}
                onDisconnect={disconnectBroker}
              />
            )}
          </div>
          <MqttBrokerActionsPanel
            brokerUuid={broker.uuid}
            isActive={broker.is_active}
            isConnected={connectedBrokerUuid === broker.uuid}
            onConnect={connectBroker}
            onDisconnect={disconnectBroker}
            onDelete={() => setDeleteDialogOpen(true)}
          />
        </div>

        <div className="border border-border rounded-xl bg-card p-6 space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <p className="text-sm font-medium text-muted-foreground">Host</p>
              <p className="font-mono text-sm">{broker.host}:{broker.port}</p>
            </div>
            <div className="space-y-2">
              <p className="text-sm font-medium text-muted-foreground">TLS</p>
              <p>{broker.use_tls ? "Yes" : "No"}</p>
            </div>
            {broker.username && (
              <div className="space-y-2">
                <p className="text-sm font-medium text-muted-foreground">Username</p>
                <p>{broker.username}</p>
              </div>
            )}
            <div className="space-y-2">
              <p className="text-sm font-medium text-muted-foreground">Status</p>
              <div className="flex items-center gap-2">
                {connectedBrokerUuid === broker.uuid ? (
                  <>
                    <Wifi className="w-4 h-4 text-green-500" />
                    <span className="text-green-500">Connected</span>
                  </>
                ) : (
                  <>
                    <WifiOff className="w-4 h-4 text-muted-foreground" />
                    <span className="text-muted-foreground">Disconnected</span>
                  </>
                )}
              </div>
            </div>
            {broker.is_default && (
              <div className="space-y-2">
                <p className="text-sm font-medium text-muted-foreground">Default</p>
                <p className="text-primary">Yes</p>
              </div>
            )}
            {broker.description && (
              <div className="space-y-2 md:col-span-2">
                <p className="text-sm font-medium text-muted-foreground">Description</p>
                <p className="text-sm">{broker.description}</p>
              </div>
            )}
          </div>
        </div>
      </div>

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Broker</AlertDialogTitle>
            <AlertDialogDescription>
              Do you really want to delete the broker "{broker.name}"?
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              disabled={isDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isDeleting ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Deleting...
                </>
              ) : (
                "Continue"
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
};

export default MqttBrokerDetail;

