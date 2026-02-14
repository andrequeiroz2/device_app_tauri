import { createContext, useContext, useEffect, useMemo, useState } from "react";
import { authApi } from "@/services/authApi";
import type { LoginPayload, LoginResult } from "@/types/auth";
import { storage } from "@/lib/storage";

interface AuthState {
  token: string | null;
  user: LoginResult["user"] | null;
  isLoading: boolean;
  login: (credentials: LoginPayload) => Promise<LoginResult>;
  logout: () => void;
}

const AuthContext = createContext<AuthState | undefined>(undefined);

export const AuthProvider = ({ children }: { children: React.ReactNode }) => {
  const [token, setToken] = useState<string | null>(null);
  const [user, setUser] = useState<AuthResponse["user"] | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const storedToken = storage.getToken();
    if (storedToken) {
      setToken(storedToken);
    }
    setIsLoading(false);
  }, []);

  const login = async (credentials: LoginPayload) => {
    const result = await authApi.login(credentials);
    if (result.success && result.token) {
      storage.setToken(result.token);
      setToken(result.token);
      setUser(result.user ?? null);
    }
    return result;
  };

  const logout = () => {
    storage.clearToken();
    setToken(null);
    setUser(null);
  };

  const value = useMemo(
    () => ({
      token,
      user,
      isLoading,
      login,
      logout,
    }),
    [token, user, isLoading],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export const useAuth = () => {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used within AuthProvider");
  }
  return ctx;
};




