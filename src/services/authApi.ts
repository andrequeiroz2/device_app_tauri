import { invoke } from "@tauri-apps/api/core";
import { ApiResponse } from "@/types/api";
import type { LoginPayload, LoginResponseData, LoginResult } from "@/types/auth";

export const authApi = {
  async login(credentials: LoginPayload): Promise<LoginResult> {
    try {
      const resp = await invoke<ApiResponse<LoginResponseData>>("login", {
        payload: credentials,
      });

      if (!resp.success) {
        return { success: false, message: normalizeMessage(resp.message) };
      }

      return {
        success: true,
        message: resp.message,
        token: resp.data?.token,
        user: resp.data?.user,
      };
    } catch (err) {
      console.error("login invoke error", err);
      return { success: false, message: normalizeMessage(err) };
    }
  },

  async forgotPassword(_email: string): Promise<LoginResult> {
    return { success: false, message: "Password recovery not implemented on backend." };
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
