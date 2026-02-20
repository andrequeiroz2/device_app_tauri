export type CollectorNotificationSeverity = "Info" | "Warn" | "Critical";

export type CollectorNotificationIsReadFilter = "no_read" | "is_read" | "all";
export type CollectorNotificationSeverityFilter = CollectorNotificationSeverity | "All";

export interface CollectorNotificationFilter {
  is_read?: CollectorNotificationIsReadFilter;
  severity?: CollectorNotificationSeverityFilter;
}

export interface CollectorNotificationPublic {
  uuid: string;
  notification_type: string;
  severity: CollectorNotificationSeverity;
  title: string;
  message: string;
  broker_uuid: string | null;
  device_uuid: string | null;
  is_read: boolean;
  created_at: string;
}

export interface CollectorNotificationListResponse {
  items: CollectorNotificationPublic[];
  total: number;
  page: number;
  page_size: number;
}
