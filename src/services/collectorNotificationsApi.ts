import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";
import type {
  CollectorNotificationPublic,
  CollectorNotificationFilter,
  CollectorNotificationListResponse,
} from "@/types/collectorNotifications";

const normalizeMessage = (msg: unknown): string => {
  if (typeof msg === "string") return msg;
  if (msg && typeof msg === "object") {
    const m = (msg as { message?: unknown })?.message;
    if (typeof m === "string") return m;
    try {
      return JSON.stringify(msg);
    } catch {
      return String(msg);
    }
  }
  return String(msg ?? "Unknown error");
};

type Result<T> = {
  success: boolean;
  message?: string;
  data?: T;
  unauthorized?: boolean;
};

type ListCollectorNotificationsResult = {
  success: boolean;
  message?: string;
  data?: CollectorNotificationListResponse;
  unauthorized?: boolean;
};

export const collectorNotificationsApi = {
  async list(
    token: string,
    page: number,
    pageSize: number,
    filter: CollectorNotificationFilter,
  ): Promise<ListCollectorNotificationsResult> {
    try {
      const resp = await invoke<ApiResponse<CollectorNotificationListResponse>>(
        "list_collector_notifications",
        {
          token,
          params: {
            page,
            page_size: pageSize,
            filter: {
              is_read: filter.is_read ?? "no_read",
              severity: filter.severity ?? "All",
            },
          },
        },
      );

      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }

      return {
        success: true,
        data: resp.data ?? { items: [], total: 0, page: 1, page_size: pageSize },
        message: resp.message,
      };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },

  async get(
    token: string,
    uuid: string,
  ): Promise<Result<CollectorNotificationPublic | null>> {
    try {
      const resp = await invoke<
        ApiResponse<CollectorNotificationPublic | null>
      >("get_collector_notification", { token, uuid });

      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }

      return {
        success: true,
        data: resp.data ?? null,
        message: resp.message,
      };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },

  async markRead(token: string, uuid: string): Promise<Result<void>> {
    try {
      const resp = await invoke<ApiResponse<void>>(
        "mark_collector_notification_read",
        { token, uuid },
      );

      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }

      return { success: true, message: resp.message };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },

  async markAllRead(token: string): Promise<Result<void>> {
    try {
      const resp = await invoke<ApiResponse<void>>(
        "mark_all_collector_notifications_read",
        { token },
      );

      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }

      return { success: true, message: resp.message };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },

  async count(token: string): Promise<Result<number>> {
    try {
      const resp = await invoke<ApiResponse<number>>(
        "count_collector_notifications",
        { token },
      );

      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }

      return {
        success: true,
        data: resp.data ?? 0,
        message: resp.message,
      };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },
};
