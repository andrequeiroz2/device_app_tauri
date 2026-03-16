import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";
import type {
  TriggerPublic,
  TriggerListParams,
  TriggerListResponse,
  TriggerCreateInput,
  TriggerUpdateInput,
} from "@/types/trigger";

const normalizeMessage = (msg: unknown): string => {
  if (typeof msg === "string") return msg;
  if (msg && typeof msg === "object") {
    const m = (msg as { message?: string }).message;
    if (typeof m === "string") return m;
    try {
      return JSON.stringify(msg);
    } catch {
      return String(msg);
    }
  }
  return String(msg ?? "Unknown error");
};

type Result<T = void> = {
  success: boolean;
  message?: string;
  data?: T;
  unauthorized?: boolean;
};

export const triggerApi = {
  async listTriggers(
    token: string,
    params: TriggerListParams
  ): Promise<Result<TriggerListResponse>> {
    try {
      const resp = await invoke<ApiResponse<TriggerListResponse>>("list_triggers", {
        token,
        params: {
          page: params.page ?? 1,
          page_size: params.page_size ?? 50,
          filter: params.filter ?? {},
        },
      });
      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }
      return { success: true, data: resp.data, message: resp.message };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },

  async getTrigger(
    token: string,
    triggerUuid: string
  ): Promise<Result<TriggerPublic>> {
    try {
      const resp = await invoke<ApiResponse<TriggerPublic>>("get_trigger", {
        token,
        triggerUuid,
      });
      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }
      return { success: true, data: resp.data, message: resp.message };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },

  async createTrigger(
    token: string,
    payload: TriggerCreateInput
  ): Promise<Result<TriggerPublic>> {
    try {
      const resp = await invoke<ApiResponse<TriggerPublic>>("create_trigger", {
        token,
        payload: {
          ...payload,
          condition_json: payload.condition_json as unknown as object,
          action_config_json: payload.action_config_json as unknown as object,
        },
      });
      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }
      return { success: true, data: resp.data, message: resp.message };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },

  async updateTrigger(
    token: string,
    payload: TriggerUpdateInput
  ): Promise<Result<TriggerPublic>> {
    try {
      const resp = await invoke<ApiResponse<TriggerPublic>>("update_trigger", {
        token,
        payload: {
          ...payload,
          condition_json: payload.condition_json as unknown as object | undefined,
          action_config_json: payload.action_config_json as unknown as object | undefined,
        },
      });
      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }
      return { success: true, data: resp.data, message: resp.message };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },

  async deleteTrigger(
    token: string,
    uuid: string
  ): Promise<Result<void>> {
    try {
      const resp = await invoke<ApiResponse<null>>("delete_trigger", {
        token,
        payload: { uuid },
      });
      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }
      return { success: true };
    } catch (err) {
      const message = normalizeMessage(err);
      return {
        success: false,
        message,
        unauthorized: message.toLowerCase().includes("unauthorized"),
      };
    }
  },

  /** Sends a test notification (Discord/Telegram only). */
  async sendTest(
    token: string,
    triggerUuid: string
  ): Promise<Result<void>> {
    try {
      const resp = await invoke<ApiResponse<null>>("trigger_send_test", {
        token,
        triggerUuid,
      });
      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }
      return { success: true };
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
