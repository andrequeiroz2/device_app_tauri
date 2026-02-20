import { useMemo, useState, useEffect } from "react";
import { useInfiniteQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "react-router-dom";
import { mqttBrokerApi } from "@/services/mqttBrokerApi";
import { useAuth } from "@/context/AuthContext";
import { useCollectorConnection } from "@/hooks/useCollectorConnection";
import { ConnectDisconnectBrokerButton } from "@/components/ConnectDisconnectBrokerButton";
import type { MqttBrokerPublic, MqttBrokerListResponse, MqttBrokerFilter } from "@/types/mqttBroker";
import { Button } from "@/components/ui/button";
import { Loader2, Server, Plus, Wifi, WifiOff } from "lucide-react";
import { toast } from "sonner";
import { MqttBrokerFilter as MqttBrokerFilterPanel } from "@/components/MqttBrokerFilter";
import { storage } from "@/lib/storage";
import { cn } from "@/lib/utils";
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

const PAGE_SIZE = 10;

const MqttBrokersList = () => {
  const { token, logout } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const {
    connectedBrokerUuid,
    connectBroker,
    disconnectBroker,
  } = useCollectorConnection();
  const [filter, setFilter] = useState<MqttBrokerFilter>({ status: "active" });
  const [inactiveModalOpen, setInactiveModalOpen] = useState(false);
  const [selectedInactiveBroker, setSelectedInactiveBroker] = useState<MqttBrokerPublic | null>(null);
  const [isActivating, setIsActivating] = useState(false);

  // Load filter from localStorage on mount
  useEffect(() => {
    const savedFilter = storage.getMqttBrokerFilter();
    setFilter(savedFilter);
  }, []);

  const {
    data,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery({
    queryKey: ["mqtt-brokers-list", filter],
    initialPageParam: 1,
    queryFn: async ({ pageParam }) => {
      if (!token) {
        logout();
        return null;
      }
      const resp = await mqttBrokerApi.listMqttBrokers(token, pageParam, PAGE_SIZE, filter);
      if (!resp.success) {
        if (resp.unauthorized) {
          logout();
        } else {
          toast.error(resp.message ?? "Failed to load brokers.");
        }
        throw new Error(resp.message ?? "Failed to load brokers.");
      }
      return resp.data as MqttBrokerListResponse;
    },
    getNextPageParam: (lastPage) => {
      if (!lastPage) return undefined;
      const { page, page_size, total } = lastPage;
      const loaded = page * page_size;
      return loaded < total ? page + 1 : undefined;
    },
    retry: false,
  });

  const items: MqttBrokerPublic[] = useMemo(() => {
    if (!data?.pages) return [];
    return data.pages.flatMap((p) => p?.items ?? []);
  }, [data]);

  const handleCardClick = (broker: MqttBrokerPublic, e: React.MouseEvent) => {
    if (!broker.is_active) {
      e.preventDefault();
      e.stopPropagation();
      setSelectedInactiveBroker(broker);
      setInactiveModalOpen(true);
    }
  };

  const handleActivate = async () => {
    if (!token || !selectedInactiveBroker) return;
    setIsActivating(true);

    const result = await mqttBrokerApi.updateMqttBroker(token, {
      uuid: selectedInactiveBroker.uuid,
      is_active: true,
    });

    setIsActivating(false);
    setInactiveModalOpen(false);
    setSelectedInactiveBroker(null);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to activate broker.");
      return;
    }

    toast.success("Broker activated successfully.");
    queryClient.invalidateQueries({ queryKey: ["mqtt-brokers-list"] });
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">Broker</h1>
          <p className="text-muted-foreground text-sm">
            List of your MQTT brokers.
          </p>
        </div>
        <MqttBrokerFilterPanel value={filter} onChange={setFilter} />
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      ) : (
        <>
          {items.length === 0 ? (
            <div className="bg-background border border-border rounded-xl p-12 text-center">
              <Server className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
              <p className="text-muted-foreground mb-4">No brokers found.</p>
              <Button asChild>
                <Link to="/mqtt-brokers/create">
                  <Plus className="w-4 h-4 mr-2" />
                  Create your first broker
                </Link>
              </Button>
            </div>
          ) : (
            <>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {items.map((broker) => (
                  <div
                    key={broker.uuid}
                    onClick={(e) => {
                      if (broker.is_active) {
                        navigate(`/mqtt-brokers/${broker.uuid}`);
                      } else {
                        handleCardClick(broker, e);
                      }
                    }}
                    className={cn(
                      "bg-background border border-border rounded-xl p-6 hover:border-primary transition-colors cursor-pointer",
                      !broker.is_active && "opacity-60 grayscale"
                    )}
                  >
                    <div className="flex items-start justify-between mb-4">
                      <div className="flex-1">
                        <h3 className="text-lg font-semibold">{broker.name}</h3>
                      </div>
                    </div>

                    <div className="space-y-2 text-sm">
                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground">Host:</span>
                        <span className="font-mono">{broker.host}:{broker.port}</span>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground">TLS:</span>
                        <span>{broker.use_tls ? "Yes" : "No"}</span>
                      </div>
                      {broker.is_default && (
                        <div className="flex items-center justify-between">
                          <span className="text-muted-foreground">Default:</span>
                          <span className="text-primary">Yes</span>
                        </div>
                      )}
                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground">Status:</span>
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
                      {broker.is_active && (
                        <div className="pt-2">
                          <ConnectDisconnectBrokerButton
                            brokerUuid={broker.uuid}
                            isActive={broker.is_active}
                            isConnected={connectedBrokerUuid === broker.uuid}
                            onConnect={connectBroker}
                            onDisconnect={disconnectBroker}
                            variant="outline"
                            size="sm"
                            className="w-full"
                          />
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              {hasNextPage && (
                <div className="flex justify-center pt-4">
                  <Button
                    variant="outline"
                    onClick={() => fetchNextPage()}
                    disabled={isFetchingNextPage}
                  >
                    {isFetchingNextPage ? (
                      <>
                        <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                        Loading...
                      </>
                    ) : (
                      "Load More"
                    )}
                  </Button>
                </div>
              )}
            </>
          )}
        </>
      )}

      <AlertDialog open={inactiveModalOpen} onOpenChange={setInactiveModalOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Broker Inactive</AlertDialogTitle>
            <AlertDialogDescription>
              The broker "{selectedInactiveBroker?.name}" is currently inactive. Only activation is allowed for inactive brokers.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isActivating}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleActivate}
              disabled={isActivating}
            >
              {isActivating ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Activating...
                </>
              ) : (
                "Activate"
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

export default MqttBrokersList;
