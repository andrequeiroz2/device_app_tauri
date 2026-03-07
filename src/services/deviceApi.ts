import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";
import type {
  DevicePublic,
  DeviceCommandChartPoint,
  DeviceCommandsChartFilter,
  DeviceListParams,
  DeviceListResponse,
} from "@/types/device";

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

export type GetDeviceResult = {
  success: boolean;
  message?: string;
  data?: DevicePublic;
  unauthorized?: boolean;
};

export type GetDeviceCommandsForChartResult = {
  success: boolean;
  message?: string;
  data?: DeviceCommandChartPoint[];
  unauthorized?: boolean;
};

export type ListDevicesResult = {
  success: boolean;
  message?: string;
  data?: DeviceListResponse;
  unauthorized?: boolean;
};

export const deviceApi = {
  async listDevices(
    token: string,
    params: DeviceListParams
  ): Promise<ListDevicesResult> {
    try {
      const resp = await invoke<ApiResponse<DeviceListResponse>>("list_devices", {
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

  async getDeviceByMac(
    token: string,
    macAddress: string
  ): Promise<GetDeviceResult> {
    try {
      const resp = await invoke<ApiResponse<DevicePublic | null>>("get_device_by_mac", {
        token,
        macAddress,
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
        data: resp.data ?? undefined,
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

  async getDevice(token: string, uuid: string): Promise<GetDeviceResult> {
    try {
      const resp = await invoke<ApiResponse<DevicePublic>>("get_device", {
        token,
        deviceUuid: uuid,
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

  async getDeviceCommandsForChart(
    token: string,
    filter: DeviceCommandsChartFilter
  ): Promise<GetDeviceCommandsForChartResult> {
    try {
      const resp = await invoke<ApiResponse<DeviceCommandChartPoint[]>>(
        "get_device_commands_for_chart",
        { token, filter }
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
};
