import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";
import type {
  SerialPortInfo,
  ProbeDeviceResult,
  AdoptDeviceInput,
  DefaultBrokerInfo,
} from "@/types/provisioning";
import type { DevicePublic } from "@/types/device";

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

export type ListSerialPortsResult = {
  success: boolean;
  message?: string;
  data?: SerialPortInfo[];
  unauthorized?: boolean;
};

export type ProbeDeviceApiResult = {
  success: boolean;
  message?: string;
  data?: ProbeDeviceResult;
  unauthorized?: boolean;
};

export type AdoptDeviceApiResult = {
  success: boolean;
  message?: string;
  data?: DevicePublic;
  unauthorized?: boolean;
};

export type GetDefaultBrokerResult = {
  success: boolean;
  message?: string;
  data?: DefaultBrokerInfo | null;
  unauthorized?: boolean;
};

export type CheckDeviceByMacResult = {
  success: boolean;
  message?: string;
  data?: { exists: boolean };
  unauthorized?: boolean;
};

export type CheckDeviceByMacForAdoptionResult = {
  success: boolean;
  message?: string;
  data?: { exists: boolean; owner_user_uuid?: string };
  unauthorized?: boolean;
};

export const provisioningApi = {
  async listSerialPorts(): Promise<ListSerialPortsResult> {
    try {
      const resp = await invoke<ApiResponse<SerialPortInfo[]>>("list_serial_ports", {});

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

  async probeDevice(payload: { port: string; baud_rate?: number }): Promise<ProbeDeviceApiResult> {
    try {
      const resp = await invoke<ApiResponse<ProbeDeviceResult>>("probe_device", {
        payload: {
          port: payload.port,
          baud_rate: payload.baud_rate ?? 115200,
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

  async adoptDevice(
    token: string,
    payload: AdoptDeviceInput,
  ): Promise<AdoptDeviceApiResult> {
    try {
      const resp = await invoke<ApiResponse<DevicePublic>>("adopt_device", {
        token,
        payload: {
          ...payload,
          baud_rate: payload.baud_rate ?? 115200,
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

  async checkDeviceByMac(token: string, macAddress: string): Promise<CheckDeviceByMacResult> {
    try {
      const resp = await invoke<ApiResponse<{ exists: boolean }>>("check_device_by_mac", {
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
        data: resp.data ?? { exists: false },
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

  async checkDeviceByMacForAdoption(
    token: string,
    macAddress: string
  ): Promise<CheckDeviceByMacForAdoptionResult> {
    try {
      const resp = await invoke<ApiResponse<{
        exists: boolean;
        owner_user_uuid?: string;
      }>>("check_device_by_mac_for_adoption", {
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
        data: resp.data ?? { exists: false },
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

  async getDefaultBroker(token: string): Promise<GetDefaultBrokerResult> {
    try {
      const resp = await invoke<ApiResponse<DefaultBrokerInfo | null>>(
        "get_default_broker_for_adoption",
        { token }
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
