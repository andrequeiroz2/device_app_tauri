import { useQuery, useQueryClient } from "@tanstack/react-query";
import { collectorApi } from "@/services/collectorApi";
import { useAuth } from "@/context/AuthContext";
import { toast } from "sonner";

const CONNECTED_BROKER_QUERY_KEY = ["connected-broker-uuid"];

export function useCollectorConnection() {
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();

  const {
    data: connectedBrokerUuid,
    isLoading: isLoadingConnected,
    refetch: refetchConnected,
  } = useQuery({
    queryKey: CONNECTED_BROKER_QUERY_KEY,
    queryFn: async () => {
      if (!token) return null;
      const result = await collectorApi.getConnectedBrokerUuid(token);
      if (!result.success) {
        if (result.unauthorized) {
          logout();
          return null;
        }
        throw new Error(result.message ?? "Failed to get connected broker");
      }
      return result.data ?? null;
    },
    enabled: !!token,
  });

  const invalidateConnected = () => {
    queryClient.invalidateQueries({ queryKey: CONNECTED_BROKER_QUERY_KEY });
    queryClient.invalidateQueries({ queryKey: ["mqtt-brokers-list"] });
    queryClient.invalidateQueries({ queryKey: ["mqtt-broker"] });
  };

  const connectBroker = async (brokerUuid: string) => {
    if (!token) return { success: false };
    const result = await collectorApi.connectBroker(token, brokerUuid);
    if (result.success) {
      invalidateConnected();
      toast.success("Broker connected");
    } else {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
      } else {
        toast.error(result.message ?? "Failed to connect broker");
      }
    }
    return result;
  };

  const disconnectBroker = async () => {
    if (!token) return { success: false };
    const result = await collectorApi.disconnectBroker(token);
    if (result.success) {
      invalidateConnected();
      toast.success("Broker disconnected");
    } else {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
      } else {
        toast.error(result.message ?? "Failed to disconnect broker");
      }
    }
    return result;
  };

  return {
    connectedBrokerUuid: connectedBrokerUuid ?? null,
    isLoadingConnected,
    refetchConnected,
    invalidateConnected,
    connectBroker,
    disconnectBroker,
    token,
    logout,
  };
}
