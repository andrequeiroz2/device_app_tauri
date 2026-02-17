export type MqttBrokerPublic = {
  uuid: string;
  name: string;
  description?: string | null;
  host: string;
  port: number;
  username?: string | null;
  use_tls: boolean;
  ca_certificate_path?: string | null;
  client_certificate_path?: string | null;
  client_key_path?: string | null;
  insecure_tls: boolean;
  client_id?: string | null;
  keep_alive_interval: number;
  clean_session: boolean;
  connection_timeout_secs: number;
  operation_timeout_secs: number;
  last_will_topic?: string | null;
  last_will_message?: string | null;
  last_will_qos: number;
  last_will_retain: boolean;
  is_active: boolean;
  is_connected: boolean;
  is_default: boolean;
  last_connected_at?: string | null;
  last_connection_error?: string | null;
  created_at: string;
  updated_at: string;
};

export type MqttBrokerCreateInput = {
  name: string;
  description?: string;
  host: string;
  port?: number;
  username?: string;
  password?: string;
  use_tls?: boolean;
  ca_certificate_path?: string;
  client_certificate_path?: string;
  client_key_path?: string;
  insecure_tls?: boolean;
  client_id?: string;
  keep_alive_interval?: number;
  clean_session?: boolean;
  connection_timeout_secs?: number;
  operation_timeout_secs?: number;
  last_will_topic?: string;
  last_will_message?: string;
  last_will_qos?: number;
  last_will_retain?: boolean;
  is_default?: boolean;
};

export type MqttBrokerStatusFilter = "active" | "all";

export type MqttBrokerFilter = {
  status?: MqttBrokerStatusFilter;
  name?: string;
  port?: number;
  default?: boolean;
  connected?: boolean;
};

export type MqttBrokerListParams = {
  page?: number;
  page_size?: number;
  filter: MqttBrokerFilter;
};

export type MqttBrokerListResponse = {
  items: MqttBrokerPublic[];
  total: number;
  page: number;
  page_size: number;
};

export type MqttBrokerUpdateInput = {
  uuid: string;
  is_active?: boolean;
};

