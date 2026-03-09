import type { IconPublic } from "./icon";

export type DeviceType = "actuator" | "sensor";
export type OperationStatus = "online" | "offline";

export type DevicePublic = {
  uuid: string;
  user_uuid: string;
  location_uuid: string;
  name: string;
  description?: string | null;
  device_type: DeviceType;
  model: string;
  firmware_version?: string | null;
  mac_address: string;
  sensor_type?: string | null;
  actuator_type?: string | null;
  device_scale?: [string, string][] | null;
  adopted_at?: string | null;
  operation_status?: OperationStatus | null;
  last_seen_at?: string | null;
  ip_address?: string | null;
  publish_qos: number;
  subscribe_qos: number;
  status_retain: boolean;
  data_retain: boolean;
  lwt_enabled: boolean;
  lwt_message?: string | null;
  lwt_qos: number;
  lwt_retain: boolean;
  heartbeat_interval: number;
  offline_threshold: number;
  last_command?: string | null;
  last_command_at?: string | null;
  is_active: boolean;
  icon?: IconPublic | null;
  position_x?: number | null;
  position_y?: number | null;
  created_at: string;
  updated_at: string;
};

export type DeviceCommandChartPoint = {
  command: string;
  sent_at: string;
  source?: string | null;
};

export type DeviceCommandsChartFilter = {
  device_uuid: string;
  start_date: string;
  end_date: string;
  limit?: number;
};

export type DeviceFilter = {
  is_active?: "all" | "active" | "inactive";
  operation_status?: "all" | "online" | "offline";
  device_type?: "all" | "actuator" | "sensor";
  location_uuid?: string;
};

export type DeviceListParams = {
  page?: number;
  page_size?: number;
  filter?: DeviceFilter;
};

export type DeviceListResponse = {
  items: DevicePublic[];
  total: number;
  page: number;
  page_size: number;
};

export type DeviceUpdateInput = {
  uuid: string;
  name?: string | null;
  description?: string | null;
  location_uuid?: string | null;
  position_x?: number | null;
  position_y?: number | null;
  is_active?: boolean | null;
};
