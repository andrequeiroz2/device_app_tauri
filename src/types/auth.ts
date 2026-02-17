import type { UserResponse } from "@/types/user";

export interface LoginPayload {
  email: string;
  password: string;
}

export interface LoginResponseData {
  token: string;
  user: UserResponse;
}

export interface LoginResult {
  success: boolean;
  message?: string;
  token?: string;
  user?: UserResponse;
}

export interface ForgotPasswordPayload {
  email: string;
}

export interface ValidateResetTokenResponse {
  user_uuid: string;
  email: string;
}

export interface ResetPasswordPayload {
  token: string;
  password: string;
  confirm_password: string;
}

export interface ChangePasswordPayload {
  current_password: string;
  new_password: string;
  confirm_password: string;
}

export interface ApiResult<T = void> {
  success: boolean;
  message?: string;
  data?: T;
}

