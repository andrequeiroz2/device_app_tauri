export type IconCategory = "sensor" | "actuator";

export type IconPublic = {
  uuid: string;
  code: string;
  name: string;
  iconify_id: string;
  category: string;
  color?: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
};

export type IconCreateInput = {
  name: string;
  iconify_id: string;
  category: IconCategory;
  color?: string | null;
};

export type IconStatusFilter = "active" | "all";

export type IconFilter = {
  status?: IconStatusFilter;
};

export type IconUpdateInput = {
  uuid: string;
  name?: string | null;
  iconify_id?: string | null;
  category?: IconCategory | null;
  color?: string | null;
  is_active?: boolean | null;
};

export type IconListParams = {
  category?: IconCategory | null;
  status?: IconStatusFilter | null;
  page?: number;
  page_size?: number;
};

export type IconListResponse = {
  items: IconPublic[];
  total: number;
  page: number;
  page_size: number;
};

/** Cores permitidas para ícones (hex). */
export const ICON_COLORS = [
  { hex: "#E53935", name: "Red" },
  { hex: "#1E88E5", name: "Blue" },
  { hex: "#43A047", name: "Green" },
  { hex: "#FB8C00", name: "Orange" },
  { hex: "#8E24AA", name: "Purple" },
  { hex: "#FDD835", name: "Yellow" },
  { hex: "#00ACC1", name: "Cyan" },
  { hex: "#5C6BC0", name: "Indigo" },
  { hex: "#7B1FA2", name: "Deep Purple" },
  { hex: "#26A69A", name: "Teal" },
  { hex: "#78909C", name: "Grey" },
] as const;
