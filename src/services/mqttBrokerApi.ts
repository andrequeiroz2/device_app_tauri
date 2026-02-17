import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";
import type { MqttBrokerCreateInput, MqttBrokerPublic, MqttBrokerListResponse, MqttBrokerFilter, MqttBrokerUpdateInput } from "@/types/mqttBroker";

const normalizeMessage = (msg: unknown): string => {
  if (typeof msg === "string") return msg;
  if (msg && typeof msg === "object") {
    const m = (msg as any).message;
    if (typeof m === "string") return m;
    try {
      return JSON.stringify(msg);
    } catch {
      return String(msg);
    }
  }
  return String(msg ?? "Unknown error");
};

type CreateMqttBrokerResult = {
  success: boolean;
  message?: string;
  data?: MqttBrokerPublic;
  unauthorized?: boolean;
};

type ListMqttBrokersResult = {
  success: boolean;
  message?: string;
  unauthorized?: boolean;
  data?: MqttBrokerListResponse;
};

type DeleteMqttBrokerResult = {
  success: boolean;
  message?: string;
  unauthorized?: boolean;
};

type GetMqttBrokerResult = {
  success: boolean;
  message?: string;
  data?: MqttBrokerPublic;
  unauthorized?: boolean;
};

type UpdateMqttBrokerResult = {
  success: boolean;
  message?: string;
  data?: MqttBrokerPublic;
  unauthorized?: boolean;
};

export const mqttBrokerApi = {
  async createMqttBroker(
    token: string,
    payload: MqttBrokerCreateInput,
  ): Promise<CreateMqttBrokerResult> {
    try {
      const resp = await invoke<ApiResponse<MqttBrokerPublic>>("create_mqtt_broker", {
        token,
        payload,
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

  async listMqttBrokers(
    token: string,
    page: number,
    pageSize: number,
    filter: MqttBrokerFilter,
  ): Promise<ListMqttBrokersResult> {
    try {
      const resp = await invoke<ApiResponse<MqttBrokerListResponse>>("list_mqtt_brokers", {
        token,
        params: { page, page_size: pageSize, filter },
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

  async deleteMqttBroker(
    token: string,
    brokerUuid: string,
  ): Promise<DeleteMqttBrokerResult> {
    try {
      const resp = await invoke<ApiResponse<null>>("delete_mqtt_broker", {
        token,
        payload: { uuid: brokerUuid },
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

  async getMqttBroker(
    token: string,
    brokerUuid: string,
  ): Promise<GetMqttBrokerResult> {
    try {
      const resp = await invoke<ApiResponse<MqttBrokerPublic>>("get_mqtt_broker", {
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

      if (!resp.data) {
        return {
          success: false,
          message: "Broker data not found in response",
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

  async updateMqttBroker(
    token: string,
    payload: MqttBrokerUpdateInput,
  ): Promise<UpdateMqttBrokerResult> {
    try {
      const resp = await invoke<ApiResponse<MqttBrokerPublic>>("update_mqtt_broker", {
        token,
        payload,
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
};

