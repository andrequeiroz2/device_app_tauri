export type SensorReadingPublic = {
  device_uuid: string;
  measurement: string;
  value: number;
  scale: string;
  recorded_at: string;
  received_at: string;
};

export type SensorReadingLatest = {
  measurement: string;
  value: number;
  scale: string;
  recorded_at: string;
};

export type SensorReadingAggregated = {
  period?: string | null;
  avg_value?: number | null;
  min_value?: number | null;
  max_value?: number | null;
  count: number;
};

export type SensorReadingFilter = {
  device_uuid?: string;
  measurement?: string;
  start_date?: string;
  end_date?: string;
  limit?: number;
  offset?: number;
};

export type SensorReadingAggregatedFilter = {
  device_uuid: string;
  measurement: string;
  start_date: string;
  end_date: string;
  period?: "hour" | "day";
};
