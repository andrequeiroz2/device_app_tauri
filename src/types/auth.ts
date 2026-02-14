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

