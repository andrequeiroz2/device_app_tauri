import { useMemo, useState, useEffect } from "react";
import { useInfiniteQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { mqttBrokerApi } from "@/services/mqttBrokerApi";
import { useAuth } from "@/context/AuthContext";
import { useCollectorConnection } from "@/hooks/useCollectorConnection";
import { ConnectDisconnectBrokerButton } from "@/components/ConnectDisconnectBrokerButton";
import type { MqttBrokerPublic, MqttBrokerListResponse, MqttBrokerFilter } from "@/types/mqttBroker";
import { Button } from "@/components/ui/button";
import { Loader2, Server, ArrowLeft } from "lucide-react";
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
            <h1 className="text-2xl font-semibold">Broker</h1>
            <p className="text-muted-foreground text-sm">
              List of your MQTT brokers.
            </p>
          </div>
        </div>
        <MqttBrokerFilterPanel value={filter} onChange={setFilter} />
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center flex-1">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      ) : items.length === 0 ? (
        <div className="bg-background border border-border rounded-xl p-12 text-center flex-1 flex flex-col items-center justify-center">
          <Server className="w-12 h-12 mb-4 text-muted-foreground" />
          <p className="text-muted-foreground mb-4">No brokers found.</p>
          <Button asChild variant="outline" size="sm">
            <Link to="/mqtt-brokers/create">
              Create your first broker
            </Link>
          </Button>
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto border border-border rounded-xl">
          <div className="space-y-2 p-4">
            {items.map((broker) => {
              if (broker.is_active) {
                return (
                  <Link
                    key={broker.uuid}
                    to={`/mqtt-brokers/${broker.uuid}`}
                    className="block rounded-lg border p-4 transition-colors hover:bg-muted/50"
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 flex-wrap">
                          <span className="font-medium">{broker.name}</span>
                          {broker.is_default && (
                            <span className="text-xs px-2 py-0.5 rounded font-medium bg-primary/20 text-primary">
                              Default
                            </span>
                          )}
                        </div>
                        <p className="mt-1 text-sm text-muted-foreground font-mono">
                          {broker.host}:{broker.port}
                        </p>
                        <p className="mt-0.5 text-xs text-muted-foreground">
                          TLS: {broker.use_tls ? "Yes" : "No"}
                        </p>
                        <p className="mt-0.5 text-xs text-muted-foreground">
                          {connectedBrokerUuid === broker.uuid ? (
                            <span className="text-green-500">Connected</span>
                          ) : (
                            <span>Disconnected</span>
                          )}
                        </p>
                      </div>
                      <div
                        onClick={(e) => {
                          e.preventDefault();
                          e.stopPropagation();
                        }}
                      >
                        <ConnectDisconnectBrokerButton
                          brokerUuid={broker.uuid}
                          isActive={broker.is_active}
                          isConnected={connectedBrokerUuid === broker.uuid}
                          onConnect={connectBroker}
                          onDisconnect={disconnectBroker}
                          variant="outline"
                          size="sm"
                        />
                      </div>
                    </div>
                  </Link>
                );
              }
              return (
                <div
                  key={broker.uuid}
                  onClick={(e) => handleCardClick(broker, e)}
                  className={cn(
                    "block rounded-lg border p-4 transition-colors hover:bg-muted/50 cursor-pointer",
                    "bg-muted/30 border-l-4 border-l-amber-500/50"
                  )}
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="font-medium">{broker.name}</span>
                        <span className="text-xs px-2 py-0.5 rounded font-medium bg-muted text-muted-foreground">
                          Inactive
                        </span>
                      </div>
                      <p className="mt-1 text-sm text-muted-foreground font-mono">
                        {broker.host}:{broker.port}
                      </p>
                    </div>
                  </div>
                </div>
              );
            })}

            {hasNextPage && (
              <div className="flex justify-center pt-4">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => fetchNextPage()}
                  disabled={isFetchingNextPage}
                >
                  {isFetchingNextPage ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      Loading...
                    </>
                  ) : (
                    "Load more"
                  )}
                </Button>
              </div>
            )}
          </div>
        </div>
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
    </motion.div>
  );
};

export default MqttBrokersList;
