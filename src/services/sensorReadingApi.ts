import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";
import type {
  SensorReadingPublic,
  SensorReadingLatest,
  SensorReadingAggregated,
  SensorReadingFilter,
  SensorReadingAggregatedFilter,
} from "@/types/sensorReading";

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

export type ListSensorReadingsResult = {
  success: boolean;
  message?: string;
  data?: SensorReadingPublic[];
  unauthorized?: boolean;
};

export type GetSensorReadingLatestResult = {
  success: boolean;
  message?: string;
  data?: SensorReadingLatest | null;
  unauthorized?: boolean;
};

export type GetSensorReadingLatestAllResult = {
  success: boolean;
  message?: string;
  data?: SensorReadingLatest[];
  unauthorized?: boolean;
};

export type GetSensorReadingAggregatedResult = {
  success: boolean;
  message?: string;
  data?: SensorReadingAggregated[];
  unauthorized?: boolean;
};

export const sensorReadingApi = {
  async listSensorReadings(
    token: string,
    filter: SensorReadingFilter
  ): Promise<ListSensorReadingsResult> {
    try {
      const resp = await invoke<ApiResponse<SensorReadingPublic[]>>(
        "list_sensor_readings",
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
        data: resp.data ?? [],
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

  async getSensorReadingLatest(
    token: string,
    deviceUuid: string,
    measurement: string
  ): Promise<GetSensorReadingLatestResult> {
    try {
      const resp = await invoke<ApiResponse<SensorReadingLatest | null>>(
        "get_sensor_reading_latest",
        { token, deviceUuid, measurement }
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

  async getSensorReadingLatestAll(
    token: string,
    deviceUuid: string
  ): Promise<GetSensorReadingLatestAllResult> {
    try {
      const resp = await invoke<ApiResponse<SensorReadingLatest[]>>(
        "get_sensor_reading_latest_all",
        { token, deviceUuid }
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
        data: resp.data ?? [],
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

  async getSensorReadingAggregated(
    token: string,
    filter: SensorReadingAggregatedFilter
  ): Promise<GetSensorReadingAggregatedResult> {
    try {
      const resp = await invoke<ApiResponse<SensorReadingAggregated[]>>(
        "get_sensor_reading_aggregated",
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
        data: resp.data ?? [],
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
