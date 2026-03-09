import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";
import type {
  IconPublic,
  IconCreateInput,
  IconUpdateInput,
  IconListParams,
  IconListResponse,
} from "@/types/icon";

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

export type ListIconsResult = {
  success: boolean;
  message?: string;
  data?: IconListResponse;
  unauthorized?: boolean;
};

export type GetIconResult = {
  success: boolean;
  message?: string;
  data?: IconPublic;
  unauthorized?: boolean;
};

export type CreateIconResult = {
  success: boolean;
  message?: string;
  data?: IconPublic;
  unauthorized?: boolean;
};

export type UpdateIconResult = {
  success: boolean;
  message?: string;
  data?: IconPublic;
  unauthorized?: boolean;
};

export type DeleteIconResult = {
  success: boolean;
  message?: string;
  unauthorized?: boolean;
};

export const iconApi = {
  async listIcons(
    token: string,
    page: number,
    pageSize: number,
    params?: Omit<IconListParams, "page" | "page_size">
  ): Promise<ListIconsResult> {
    try {
      const resp = await invoke<ApiResponse<IconListResponse>>("list_icons", {
        token,
        params: {
          ...(params ?? {}),
          page,
          page_size: pageSize,
          status: params?.status ?? "active",
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

      return {
        success: true,
        data: resp.data,
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

  async getIcon(token: string, iconUuid: string): Promise<GetIconResult> {
    try {
      const resp = await invoke<ApiResponse<IconPublic>>("get_icon", {
        token,
        iconUuid,
      });

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
        data: resp.data,
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

  async createIcon(
    token: string,
    payload: IconCreateInput
  ): Promise<CreateIconResult> {
    try {
      const resp = await invoke<ApiResponse<IconPublic>>("create_icon", {
        token,
        payload: {
          name: payload.name.trim(),
          iconify_id: payload.iconify_id.trim(),
          category: payload.category,
          color: payload.color?.trim() || undefined,
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

      return {
        success: true,
        data: resp.data,
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

  async updateIcon(
    token: string,
    payload: IconUpdateInput
  ): Promise<UpdateIconResult> {
    try {
      const resp = await invoke<ApiResponse<IconPublic>>("update_icon", {
        token,
        payload: {
          uuid: payload.uuid,
          name: payload.name?.trim() || undefined,
          iconify_id: payload.iconify_id?.trim() || undefined,
          category: payload.category ?? undefined,
          color: payload.color?.trim() || undefined,
          is_active: payload.is_active ?? undefined,
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

      return {
        success: true,
        data: resp.data,
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

  async deleteIcon(
    token: string,
    uuid: string
  ): Promise<DeleteIconResult> {
    try {
      const resp = await invoke<ApiResponse<null>>("delete_icon", {
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

      return {
        success: true,
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
