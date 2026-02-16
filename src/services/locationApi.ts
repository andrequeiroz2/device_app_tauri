import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "@/types/api";
import type { LocationCreateInput, LocationImageInput, LocationCreateCommandInput, LocationListResponse, LocationPublic, LocationFilter, LocationUpdateInput } from "@/types/location";

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

type UpdateLocationResult = {
  success: boolean;
  message?: string;
  data?: LocationPublic;
  unauthorized?: boolean;
};

type GetLocationResult = {
  success: boolean;
  message?: string;
  data?: LocationPublic;
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
    filter: LocationFilter,
  ): Promise<ListLocationsResult> {
    try {
      const resp = await invoke<ApiResponse<LocationListResponse>>("list_locations", {
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

  async updateLocation(
    token: string,
    payload: LocationUpdateInput,
    image?: File | null,
  ): Promise<UpdateLocationResult> {
    try {
      let updatePayload: LocationUpdateInput = { ...payload };

      if (image) {
        updatePayload.image = {
          data_base64: await toBase64(image),
          original_name: image.name,
          mime: image.type,
          size_bytes: image.size,
        };
      }

      const resp = await invoke<ApiResponse<LocationPublic>>("update_location", {
        token,
        payload: updatePayload,
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

  async getLocation(
    token: string,
    locationUuid: string,
  ): Promise<GetLocationResult> {
    try {
      console.log("getLocation called with:", { locationUuid, hasToken: !!token });
      const resp = await invoke<ApiResponse<LocationPublic>>("get_location", {
        token,
        locationUuid: locationUuid, // Tauri converts camelCase to snake_case automatically
      });

      console.log("getLocation response:", resp);

      if (!resp.success) {
        const message = normalizeMessage(resp.message);
        console.error("getLocation failed:", message);
        return {
          success: false,
          message,
          unauthorized: message.toLowerCase().includes("unauthorized"),
        };
      }

      if (!resp.data) {
        console.error("getLocation: response.success but no data");
        return {
          success: false,
          message: "Location data not found in response",
        };
      }

      return {
        success: true,
        data: resp.data,
        message: resp.message,
      };
    } catch (err) {
      const message = normalizeMessage(err);
      console.error("getLocation invoke error:", err);
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

