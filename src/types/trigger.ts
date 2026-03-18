/** Trigger list/detail from API */
export type TriggerPublic = {
  uuid: string;
  device_uuid: string | null;
  name: string;
  source_event: SourceEvent;
  condition_json: ConditionJson;
  action_type: ActionType;
  action_config_json: ActionConfigJson;
  cooldown_seconds?: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
};

export type SourceEvent = "sensor_reading" | "device_command" | "schedule";
export type ActionType = "discord" | "telegram" | "device_command";

export type TriggerSeverity = "inf" | "att" | "warn" | "critical";

/** Condition shapes by source_event */
export type ConditionSensorReading = {
  measurement: string;
  operator: ">=" | "<=" | "==" | "!=" | ">" | "<";
  value: number;
};
export type ConditionDeviceCommand =
  | { command: string }
  | { command_pattern: Record<string, unknown> };
export type ConditionSchedule = {
  days_of_week: number[];
  time: string;
  start_date: string;
  end_date: string;
};
export type ConditionJson =
  | ConditionSensorReading
  | ConditionDeviceCommand
  | ConditionSchedule;

/** Action config shapes by action_type */
export type ActionConfigDiscord = { webhook_url: string; severity?: TriggerSeverity };
export type ActionConfigTelegram = { bot_token: string; chat_id: string; severity?: TriggerSeverity };
export type ActionConfigDeviceCommand =
  | { target_device_uuid: string; command: string }
  | { target_device_uuid: string; command_payload: Record<string, unknown> };
export type ActionConfigJson =
  | ActionConfigDiscord
  | ActionConfigTelegram
  | ActionConfigDeviceCommand;

export type TriggerFilter = {
  device_uuid?: string;
  is_active?: boolean;
};

export type TriggerListParams = {
  page?: number;
  page_size?: number;
  filter?: TriggerFilter;
};

export type TriggerListResponse = {
  items: TriggerPublic[];
  total: number;
  page: number;
  page_size: number;
};

export type TriggerCreateInput = {
  device_uuid?: string | null;
  name: string;
  source_event: SourceEvent;
  condition_json: ConditionJson;
  action_type: ActionType;
  action_config_json: ActionConfigJson;
  cooldown_seconds?: number;
  is_active?: boolean;
};

export type TriggerUpdateInput = {
  uuid: string;
  device_uuid?: string | null;
  name?: string;
  source_event?: SourceEvent;
  condition_json?: ConditionJson;
  action_type?: ActionType;
  action_config_json?: ActionConfigJson;
  cooldown_seconds?: number;
  is_active?: boolean;
};
