import type { LocationFilter } from "@/types/location";

const TOKEN_KEY = "device_app_token";
const LOCATION_FILTER_KEY = "device_app_location_filter";

const DEFAULT_FILTER: LocationFilter = {
  status: "active",
};

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
  getLocationFilter(): LocationFilter {
    if (typeof localStorage === "undefined") return DEFAULT_FILTER;
    try {
      const stored = localStorage.getItem(LOCATION_FILTER_KEY);
      if (!stored) return DEFAULT_FILTER;
      const parsed = JSON.parse(stored) as LocationFilter;
      // Validate structure
      if (parsed.status === "active" || parsed.status === "all") {
        return parsed;
      }
      return DEFAULT_FILTER;
    } catch {
      return DEFAULT_FILTER;
    }
  },
  setLocationFilter(filter: LocationFilter) {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(LOCATION_FILTER_KEY, JSON.stringify(filter));
  },
};


