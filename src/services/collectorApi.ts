import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";

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

type CollectorResult<T = void> = {
  success: boolean;
  message?: string;
  data?: T;
  unauthorized?: boolean;
};

export const collectorApi = {
  async connectBroker(
    token: string,
    brokerUuid: string,
  ): Promise<CollectorResult<void>> {
    try {
      const resp = await invoke<ApiResponse<void>>("connect_broker", {
        token,
        brokerUuid,
      });

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

  async disconnectBroker(token: string): Promise<CollectorResult<void>> {
    try {
      const resp = await invoke<ApiResponse<void>>("disconnect_broker", {
        token,
      });

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

  async getConnectedBrokerUuid(
    token: string,
  ): Promise<CollectorResult<string | null>> {
    try {
      const resp = await invoke<ApiResponse<string | null>>(
        "get_connected_broker_uuid",
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
};
