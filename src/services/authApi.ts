import { invoke } from "@tauri-apps/api/core";
import { ApiResponse } from "@/types/api";
import type {
  LoginPayload,
  LoginResponseData,
  LoginResult,
  ForgotPasswordPayload,
  ValidateResetTokenResponse,
  ResetPasswordPayload,
  ChangePasswordPayload,
  ApiResult,
} from "@/types/auth";

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

  async forgotPassword(payload: ForgotPasswordPayload): Promise<ApiResult> {
    try {
      const resp = await invoke<ApiResponse<void>>("forgot_password", {
        payload,
      });

      if (!resp.success) {
        return { success: false, message: normalizeMessage(resp.message) };
      }

      return {
        success: true,
        message: resp.message || "Recovery email sent successfully",
      };
    } catch (err) {
      console.error("forgotPassword invoke error", err);
      return { success: false, message: normalizeMessage(err) };
    }
  },

  async validateResetToken(token: string): Promise<ApiResult<ValidateResetTokenResponse>> {
    try {
      const resp = await invoke<ApiResponse<ValidateResetTokenResponse>>("validate_reset_token", {
        token,
      });

      if (!resp.success) {
        return { success: false, message: normalizeMessage(resp.message) };
      }

      return {
        success: true,
        message: resp.message,
        data: resp.data,
      };
    } catch (err) {
      console.error("validateResetToken invoke error", err);
      return { success: false, message: normalizeMessage(err) };
    }
  },

  async resetPassword(payload: ResetPasswordPayload): Promise<ApiResult> {
    try {
      const resp = await invoke<ApiResponse<void>>("reset_password", {
        payload,
      });

      if (!resp.success) {
        return { success: false, message: normalizeMessage(resp.message) };
      }

      return {
        success: true,
        message: resp.message || "Password reset successfully",
      };
    } catch (err) {
      console.error("resetPassword invoke error", err);
      return { success: false, message: normalizeMessage(err) };
    }
  },

  async changePassword(payload: ChangePasswordPayload, token: string): Promise<ApiResult> {
    try {
      const resp = await invoke<ApiResponse<void>>("change_password", {
        token,
        payload,
      });

      if (!resp.success) {
        return { success: false, message: normalizeMessage(resp.message) };
      }

      return {
        success: true,
        message: resp.message || "Password changed successfully",
      };
    } catch (err) {
      console.error("changePassword invoke error", err);
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
