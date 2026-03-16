export type SerialPortInfo = {
  port_name: string;
  port_type: string;
  manufacturer?: string | null;
  product?: string | null;
  serial_number?: string | null;
};

/** Sensor: { [measurement]: { unit, min_reading, max_reading } }; Actuator: { type: "discrete"|"range", ... } */
export type ParameterRanges = Record<string, { unit: string; min_reading: number; max_reading: number }>;
export type CommandSpec =
  | { type: "discrete"; commands: string[] }
  | { type: "range"; min: number; max: number; unit: string };

export type DeviceInfoInput = {
  device_type: string;
  model: string;
  mac_address: string;
  sensor_type?: string | null;
  actuator_type?: string | null;
  device_scale?: unknown;
  parameter_ranges?: ParameterRanges | null;
  command_spec?: CommandSpec | null;
  firmware_version?: string | null;
};

/** Device info from probe (backend returns boarder_type) */
export type ProbeDeviceInfo = DeviceInfoInput & {
  boarder_type?: string;
  /** Present when adopted; used to check if device belongs to logged user */
  user_uuid?: string | null;
};

export type ProbeDeviceResult = {
  port: string;
  firmware_version?: string | null;
  device_info: ProbeDeviceInfo;
  can_adopt: boolean;
  message?: string | null;
};

export type AdoptDeviceInput = {
  port: string;
  baud_rate?: number;
  name: string;
  location_uuid: string;
  description?: string | null;
  icon_uuid?: string | null;
  broker_url: string;
  wifi_ssid: string;
  wifi_password: string;
  device_info: DeviceInfoInput;
};

export type DefaultBrokerInfo = {
  host: string;
  port: number;
  use_tls: boolean;
  broker_url: string;
};

export const BAUDRATES = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600] as const;
