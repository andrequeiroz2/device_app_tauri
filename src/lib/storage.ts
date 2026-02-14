const TOKEN_KEY = "device_app_token";

export const storage = {
  getToken(): string | null {
    if (typeof localStorage === "undefined") return null;
    return localStorage.getItem(TOKEN_KEY);
  },
  setToken(token: string) {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(TOKEN_KEY, token);
  },
  clearToken() {
    if (typeof localStorage === "undefined") return;
    localStorage.removeItem(TOKEN_KEY);
  },
};


