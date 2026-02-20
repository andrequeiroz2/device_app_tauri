import { useQuery, useInfiniteQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/context/AuthContext";
import { collectorNotificationsApi } from "@/services/collectorNotificationsApi";
import type { CollectorNotificationFilter } from "@/types/collectorNotifications";

export const COLLECTOR_NOTIFICATIONS_KEYS = {
  all: ["collector-notifications"] as const,
  list: (filter?: CollectorNotificationFilter) =>
    [...COLLECTOR_NOTIFICATIONS_KEYS.all, "list", filter ?? {}] as const,
  preview: () => [...COLLECTOR_NOTIFICATIONS_KEYS.all, "preview"] as const,
  detail: (uuid: string) => [...COLLECTOR_NOTIFICATIONS_KEYS.all, "detail", uuid] as const,
  count: () => [...COLLECTOR_NOTIFICATIONS_KEYS.all, "count"] as const,
};

const PREVIEW_PAGE_SIZE = 5;
const LIST_PAGE_SIZE = 20;

export function useCollectorNotificationsPreview() {
  const { token } = useAuth();

  return useQuery({
    queryKey: COLLECTOR_NOTIFICATIONS_KEYS.preview(),
    queryFn: async () => {
      if (!token) return [];
      const result = await collectorNotificationsApi.list(
        token,
        1,
        PREVIEW_PAGE_SIZE,
        { is_read: "no_read", severity: "All" },
      );
      if (!result.success) throw new Error(result.message ?? "Failed to list notifications");
      return result.data?.items ?? [];
    },
    enabled: !!token,
  });
}

export function useCollectorNotificationsList(filter: CollectorNotificationFilter) {
  const { token } = useAuth();

  return useInfiniteQuery({
    queryKey: COLLECTOR_NOTIFICATIONS_KEYS.list(filter),
    initialPageParam: 1,
    queryFn: async ({ pageParam }) => {
      if (!token) throw new Error("Not authenticated");
      const result = await collectorNotificationsApi.list(
        token,
        pageParam,
        LIST_PAGE_SIZE,
        filter,
      );
      if (!result.success) throw new Error(result.message ?? "Failed to list notifications");
      return (
        result.data ?? { items: [], total: 0, page: 1, page_size: LIST_PAGE_SIZE }
      );
    },
    getNextPageParam: (lastPage) => {
      const loaded = lastPage.page * lastPage.page_size;
      return loaded < lastPage.total ? lastPage.page + 1 : undefined;
    },
    enabled: !!token,
  });
}

export function useCollectorNotificationsCount() {
  const { token } = useAuth();

  return useQuery({
    queryKey: COLLECTOR_NOTIFICATIONS_KEYS.count(),
    queryFn: async () => {
      if (!token) return 0;
      const result = await collectorNotificationsApi.count(token);
      if (!result.success) throw new Error(result.message ?? "Failed to count notifications");
      return result.data ?? 0;
    },
    enabled: !!token,
  });
}

export function useCollectorNotification(uuid: string | null) {
  const { token } = useAuth();

  return useQuery({
    queryKey: COLLECTOR_NOTIFICATIONS_KEYS.detail(uuid ?? ""),
    queryFn: async () => {
      if (!token || !uuid) return null;
      const result = await collectorNotificationsApi.get(token, uuid);
      if (!result.success) throw new Error(result.message ?? "Failed to get notification");
      return result.data ?? null;
    },
    enabled: !!token && !!uuid,
  });
}

export function useMarkCollectorNotificationRead() {
  const queryClient = useQueryClient();
  const { token } = useAuth();

  return useMutation({
    mutationFn: async (uuid: string) => {
      if (!token) throw new Error("Not authenticated");
      const result = await collectorNotificationsApi.markRead(token, uuid);
      if (!result.success) throw new Error(result.message ?? "Failed to mark as read");
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: COLLECTOR_NOTIFICATIONS_KEYS.all });
    },
  });
}

export function useMarkAllCollectorNotificationsRead() {
  const queryClient = useQueryClient();
  const { token } = useAuth();

  return useMutation({
    mutationFn: async () => {
      if (!token) throw new Error("Not authenticated");
      const result = await collectorNotificationsApi.markAllRead(token);
      if (!result.success) throw new Error(result.message ?? "Failed to mark all as read");
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: COLLECTOR_NOTIFICATIONS_KEYS.all });
    },
  });
}
