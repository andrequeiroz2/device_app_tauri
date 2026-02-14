import { invoke } from "@tauri-apps/api/core";
import { ApiResponse } from "@/types/api";
import { CreateUserResponse } from "@/types/user";

export interface RegisterCredentials {
  name: string;
  email: string;
  password: string;
  confirm_password: string;
}

export interface UserCreateResult {
  success: boolean;
  message?: string;
  user?: {
    uuid?: string;
    username?: string;
    email: string;
  };
}

export const userApi = {
  async create(credentials: RegisterCredentials): Promise<UserCreateResult> {
    try {
      const response = await invoke<ApiResponse<CreateUserResponse>>("create_user", {
        payload: {
          username: credentials.name,
          email: credentials.email,
          password: credentials.password,
          confirm_password: credentials.confirm_password ?? credentials.password,
        },
      });
      console.log("create_user response", response);

      if (!response.success) {
        console.error("create_user error response", response);
        return { success: false, message: normalizeMessage(response.message) };
      }

      return {
        success: true,
        user: {
          uuid: response.data?.uuid,
          username: response.data?.username,
          email: response.data?.email ?? credentials.email,
        },
        message: response.message,
      };
    } catch (err) {
      console.error("create_user invoke error", err);
      return { success: false, message: normalizeMessage(err) };
    }
  },
};

function normalizeMessage(msg: unknown): string {
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
}

