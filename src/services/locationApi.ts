import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";
import type { LocationCreateInput, LocationImageInput, LocationCreateCommandInput, LocationListResponse, LocationPublic } from "@/types/location";

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

type CreateLocationResult = {
  success: boolean;
  message?: string;
  data?: LocationPublic;
  unauthorized?: boolean;
};

type ListLocationsResult = {
  success: boolean;
  message?: string;
  unauthorized?: boolean;
  data?: LocationListResponse;
};

type DeleteLocationResult = {
  success: boolean;
  message?: string;
  unauthorized?: boolean;
};

export const locationApi = {
  async createLocation(
    token: string,
    payload: LocationCreateInput,
    image?: File | null,
  ): Promise<CreateLocationResult> {
    try {
      let imageInput: LocationImageInput | undefined = undefined;
      if (image) {
        imageInput = {
          data_base64: await toBase64(image),
          original_name: image.name,
          mime: image.type,
          size_bytes: image.size,
        };
      }

      const cmdPayload: LocationCreateCommandInput = {
        location: payload,
        image: imageInput,
      };

      const resp = await invoke<ApiResponse<LocationPublic>>("create_location", {
        token,
        payload: cmdPayload,
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

  async listLocations(
    token: string,
    page: number,
    pageSize: number,
  ): Promise<ListLocationsResult> {
    try {
      const resp = await invoke<ApiResponse<LocationListResponse>>("list_locations", {
        token,
        params: { page, page_size: pageSize },
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

  async deleteLocation(
    token: string,
    locationUuid: string,
  ): Promise<DeleteLocationResult> {
    try {
      const resp = await invoke<ApiResponse<null>>("delete_location", {
        token,
        payload: { uuid: locationUuid },
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

async function toBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      const base64 = result.includes(",") ? result.split(",")[1] : result;
      resolve(base64);
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

