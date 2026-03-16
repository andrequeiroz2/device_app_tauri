import { useState, useEffect } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { triggerApi } from "@/services/triggerApi";
import { deviceApi } from "@/services/deviceApi";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
import { Loader2, ArrowLeft, ChevronDown, Eye, EyeOff } from "lucide-react";
import { toast } from "sonner";
import type {
  TriggerCreateInput,
  TriggerUpdateInput,
  SourceEvent,
  ActionType,
  ConditionJson,
  ActionConfigJson,
  ConditionSensorReading,
  ConditionSchedule,
  ActionConfigTelegram,
} from "@/types/trigger";
import type { DevicePublic } from "@/types/device";
import { cn } from "@/lib/utils";

const SOURCE_EVENTS: SourceEvent[] = ["sensor_reading", "device_command", "schedule"];
const ACTION_TYPES: ActionType[] = ["discord", "telegram", "device_command"];
const SOURCE_EVENT_LABELS: Record<SourceEvent, string> = {
  sensor_reading: "Sensor reading",
  device_command: "Device command",
  schedule: "Schedule",
};
const ACTION_TYPE_LABELS: Record<ActionType, string> = {
  discord: "Discord",
  telegram: "Telegram",
  device_command: "Device command",
};
const OPERATORS = [">=", "<=", "==", "!=", ">", "<"] as const;

type MeasurementWithRange = {
  name: string;
  unit: string;
  min_reading?: number;
  max_reading?: number;
};

const DAYS = [
  { value: 0, label: "Sun" },
  { value: 1, label: "Mon" },
  { value: 2, label: "Tue" },
  { value: 3, label: "Wed" },
  { value: 4, label: "Thu" },
  { value: 5, label: "Fri" },
  { value: 6, label: "Sat" },
];

export default function TriggerForm() {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { token, logout } = useAuth();
  const isEdit = !!uuid;

  const [deviceUuid, setDeviceUuid] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [sourceEvent, setSourceEvent] = useState<SourceEvent>("sensor_reading");
  const [condition, setCondition] = useState<ConditionJson>({
    measurement: "temperature",
    operator: ">=",
    value: 0,
  });
  const [actionType, setActionType] = useState<ActionType>("discord");
  const [actionConfig, setActionConfig] = useState<ActionConfigJson>({
    webhook_url: "",
  });
  const [isActive, setIsActive] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [showBotToken, setShowBotToken] = useState(false);

  const { data: devicesData } = useQuery({
    queryKey: ["devices"],
    queryFn: async () => {
      if (!token) return { items: [] as DevicePublic[] };
      const r = await deviceApi.listDevices(token, { page: 1, page_size: 200 });
      if (!r.success || !r.data) return { items: [] as DevicePublic[] };
      const raw = r.data as { items?: DevicePublic[]; total?: number; page?: number; page_size?: number };
      const items = Array.isArray(raw.items) ? raw.items : [];
      return { ...raw, items };
    },
    enabled: !!token,
  });
  const devices: DevicePublic[] = Array.isArray(devicesData?.items) ? devicesData.items : [];

  const selectedDevice: DevicePublic | null = deviceUuid ? devices.find((d) => d.uuid === deviceUuid) ?? null : null;
  const sensorMeasurementsWithRange: MeasurementWithRange[] = (() => {
    if (selectedDevice?.device_type !== "sensor") return [];
    const pr = selectedDevice.parameter_ranges;
    if (pr) {
      return (Object.entries(pr) as [string, { unit: string; min_reading: number; max_reading: number }][]).map(
        ([name, range]) => ({
          name,
          unit: range.unit,
          min_reading: range.min_reading,
          max_reading: range.max_reading,
        })
      );
    }
    const scale = selectedDevice.device_scale;
    if (scale?.length) {
      return scale.map((pair) => ({
        name: pair[0],
        unit: "",
        min_reading: undefined,
        max_reading: undefined,
      }));
    }
    return [];
  })();

  const currentMeasurementRange =
    sourceEvent === "sensor_reading" &&
    sensorMeasurementsWithRange.length > 0 &&
    (condition as { measurement?: string }).measurement
      ? sensorMeasurementsWithRange.find(
          (m) => m.name === (condition as { measurement?: string }).measurement
        )
      : null;

  useEffect(() => {
    if (sourceEvent !== "sensor_reading" || !selectedDevice || sensorMeasurementsWithRange.length === 0) return;
    const c = condition as ConditionSensorReading;
    const current = c.measurement ?? "";
    const exists = sensorMeasurementsWithRange.some((m) => m.name === current);
    if (!exists) {
      const first = sensorMeasurementsWithRange[0];
      setCondition((prev) => ({
        ...(prev as ConditionSensorReading),
        measurement: first.name,
        value:
          first.min_reading !== undefined && first.max_reading !== undefined
            ? first.min_reading
            : typeof (prev as ConditionSensorReading).value === "number"
              ? (prev as ConditionSensorReading).value
              : 0,
      }));
      return;
    }
    const range = sensorMeasurementsWithRange.find((m) => m.name === current);
    const minR = range?.min_reading;
    const maxR = range?.max_reading;
    if (minR !== undefined && maxR !== undefined) {
      const val = Number(c.value);
      if (Number.isFinite(val) && (val < minR || val > maxR)) {
        setCondition((prev) => ({
          ...(prev as ConditionSensorReading),
          value: Math.max(minR, Math.min(maxR, val)),
        }));
      }
    }
  }, [sourceEvent, selectedDevice?.uuid, (condition as ConditionSensorReading).measurement]);

  const { data: triggerData, isLoading: loadingTrigger } = useQuery({
    queryKey: ["trigger", uuid],
    queryFn: async () => {
      if (!token || !uuid) return null;
      const r = await triggerApi.getTrigger(token, uuid);
      if (!r.success || !r.data) throw new Error(r.message ?? "Failed to load trigger");
      return r.data;
    },
    enabled: !!token && isEdit && !!uuid,
  });

  useEffect(() => {
    if (!triggerData) return;
    setName(triggerData.name);
    setDeviceUuid(triggerData.device_uuid ?? null);
    setSourceEvent(triggerData.source_event as SourceEvent);
    setCondition(triggerData.condition_json as ConditionJson);
    setActionType(triggerData.action_type as ActionType);
    setActionConfig(triggerData.action_config_json as ActionConfigJson);
    setIsActive(triggerData.is_active);
  }, [triggerData]);

  const buildCondition = (): ConditionJson => {
    if (sourceEvent === "sensor_reading") {
      const c = condition as { measurement?: string; operator?: string; value?: number };
      return {
        measurement: c.measurement ?? "temperature",
        operator: (c.operator as ">=") ?? ">=",
        value: Number(c.value) ?? 0,
      };
    }
    if (sourceEvent === "device_command") {
      const c = condition as { command?: string; command_pattern?: Record<string, unknown> };
      if (c.command !== undefined) return { command: c.command };
      return { command_pattern: c.command_pattern ?? {} };
    }
    const c = condition as { days_of_week?: number[]; time?: string; start_date?: string; end_date?: string };
    return {
      days_of_week: c.days_of_week ?? [1, 2, 3, 4, 5],
      time: c.time ?? "08:00",
      start_date: c.start_date ?? "",
      end_date: c.end_date ?? "",
    };
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token) return;
    const errors: Record<string, string> = {};
    if (!name.trim()) errors.name = "Name is required.";
    if (sourceEvent !== "schedule" && !deviceUuid) errors.device = "Device is required for this event type.";
    if (actionType === "discord") {
      const url = (actionConfig as { webhook_url?: string }).webhook_url?.trim();
      if (!url) errors.webhook_url = "Webhook URL is required.";
    }
    if (actionType === "telegram") {
      if (!(actionConfig as { bot_token?: string }).bot_token?.trim()) errors.bot_token = "Bot token is required.";
      if (!(actionConfig as { chat_id?: string }).chat_id?.trim()) errors.chat_id = "Chat ID is required.";
    }
    if (actionType === "device_command") {
      if (!(actionConfig as { target_device_uuid?: string }).target_device_uuid) errors.target_device = "Target device is required.";
      if (!(actionConfig as { command?: string }).command?.trim()) errors.command = "Command is required.";
    }
    if (Object.keys(errors).length > 0) {
      setFieldErrors(errors);
      toast.error("Please fix the errors below.");
      return;
    }
    setFieldErrors({});
    setSubmitting(true);
    const conditionJson = buildCondition();

    if (isEdit && uuid) {
      const payload: TriggerUpdateInput = {
        uuid,
        device_uuid: deviceUuid,
        name: name.trim(),
        source_event: sourceEvent,
        condition_json: conditionJson,
        action_type: actionType,
        action_config_json: actionConfig,
        is_active: isActive,
      };
      const result = await triggerApi.updateTrigger(token, payload);
      setSubmitting(false);
      if (result.unauthorized) {
        logout();
        return;
      }
      if (result.success) {
        toast.success("Trigger updated.");
        navigate("/triggers/list");
      } else {
        toast.error(result.message ?? "Failed to update trigger");
      }
    } else {
      const payload: TriggerCreateInput = {
        device_uuid: deviceUuid ?? undefined,
        name: name.trim(),
        source_event: sourceEvent,
        condition_json: conditionJson,
        action_type: actionType,
        action_config_json: actionConfig,
        is_active: isActive,
      };
      const result = await triggerApi.createTrigger(token, payload);
      setSubmitting(false);
      if (result.unauthorized) {
        logout();
        return;
      }
      if (result.success) {
        toast.success("Trigger created.");
        navigate("/triggers/list");
      } else {
        toast.error(result.message ?? "Failed to create trigger");
      }
    }
  };

  if (isEdit && loadingTrigger) {
    return (
      <div className="min-h-screen bg-secondary/20 text-foreground flex items-center justify-center">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-secondary/20 text-foreground">
      <div className="max-w-4xl mx-auto py-10 px-4 space-y-6">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" asChild aria-label="Voltar">
            <Link to="/triggers/list">
              <ArrowLeft className="w-5 h-5" />
            </Link>
          </Button>
          <div>
            <h1 className="text-2xl font-semibold">
              {isEdit ? "Edit trigger" : "Create trigger"}
            </h1>
            <p className="text-muted-foreground text-sm mt-0.5">
              {isEdit
                ? "Update trigger condition, action and status."
                : "Create a trigger to automate notifications or device commands."}
            </p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="bg-background border border-border rounded-xl p-6 shadow-sm space-y-6">
          <section className="space-y-4">
            <div>
              <label className="text-sm font-medium mb-1 block">Name *</label>
              <input
                type="text"
                className={cn(
                  "w-full rounded-lg border px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary",
                  fieldErrors.name ? "border-destructive bg-transparent" : "border-input bg-transparent"
                )}
                value={name}
                onChange={(e) => {
                  setName(e.target.value);
                  if (fieldErrors.name) setFieldErrors((prev) => ({ ...prev, name: "" }));
                }}
                placeholder="e.g. Temperature alert"
                aria-invalid={!!fieldErrors.name}
                aria-describedby={fieldErrors.name ? "name-error" : undefined}
              />
              {fieldErrors.name && (
                <p id="name-error" className="text-sm text-destructive mt-1">
                  {fieldErrors.name}
                </p>
              )}
            </div>

            <div>
              <label className="text-sm font-medium mb-1 block">Event Type</label>
              <div className="rounded-lg border border-input p-2">
                <div className="flex flex-wrap gap-2">
                  {SOURCE_EVENTS.map((ev) => {
                    const isSelected = sourceEvent === ev;
                    return (
                      <button
                        key={ev}
                        type="button"
                        onClick={() => {
                          setSourceEvent(ev);
                          if (ev === "sensor_reading")
                            setCondition({ measurement: "temperature", operator: ">=", value: 0 });
                          if (ev === "device_command") setCondition({ command: "ON" });
                          if (ev === "schedule") {
                            setDeviceUuid(null);
                            setCondition({
                              days_of_week: [1, 2, 3, 4, 5],
                              time: "08:00",
                              start_date: "",
                              end_date: "",
                            });
                          }
                        }}
                        className={cn(
                          "flex items-center justify-center gap-1.5 px-4 py-2 rounded-lg border-2 transition-colors text-sm font-medium",
                          isSelected
                            ? "border-primary bg-primary/10"
                            : "border-border hover:bg-muted/50 hover:border-muted-foreground/30"
                        )}
                      >
                        {SOURCE_EVENT_LABELS[ev]}
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>

            {(sourceEvent === "sensor_reading" || sourceEvent === "device_command") && (
              <div>
                <label className="text-sm font-medium mb-1 block">Device *</label>
                <div className="relative">
                  <select
                    key={`device-select-${devices.length}`}
                    className={cn(
                      "h-10 w-full appearance-none rounded-lg border bg-card pr-9 pl-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary",
                      fieldErrors.device ? "border-destructive" : "border-input"
                    )}
                    value={deviceUuid ?? ""}
                    onChange={(e) => {
                      setDeviceUuid(e.target.value || null);
                      if (fieldErrors.device) setFieldErrors((prev) => ({ ...prev, device: "" }));
                    }}
                    aria-invalid={!!fieldErrors.device}
                    aria-describedby={fieldErrors.device ? "device-error" : undefined}
                  >
                    <option value="">— Select device —</option>
                    {devices.map((d) => (
                      <option key={d.uuid} value={d.uuid}>
                        {d.name} ({d.device_type})
                      </option>
                    ))}
                  </select>
                  <ChevronDown className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                </div>
                {fieldErrors.device && (
                  <p id="device-error" className="text-sm text-destructive mt-1">
                    {fieldErrors.device}
                  </p>
                )}
                <p className="text-xs text-muted-foreground mt-1">
                  Required for this event type.
                </p>
              </div>
            )}
          </section>

          <section className="space-y-4">
            <div>
              <label className="text-sm font-medium mb-1 block">Action *</label>
              <div className="rounded-lg border border-input p-2">
                <div className="flex flex-wrap gap-2">
                  {ACTION_TYPES.map((a) => {
                    const isSelected = actionType === a;
                    return (
                      <button
                        key={a}
                        type="button"
                        onClick={() => {
                          setActionType(a);
                          if (a === "discord") setActionConfig({ webhook_url: "" });
                          if (a === "telegram") setActionConfig({ bot_token: "", chat_id: "" });
                          if (a === "device_command")
                            setActionConfig({ target_device_uuid: "", command: "" });
                          setFieldErrors((prev) => {
                            const next = { ...prev };
                            delete next.webhook_url;
                            delete next.bot_token;
                            delete next.chat_id;
                            delete next.target_device;
                            delete next.command;
                            return next;
                          });
                        }}
                        className={cn(
                          "flex items-center justify-center gap-1.5 px-4 py-2 rounded-lg border-2 transition-colors text-sm font-medium",
                          isSelected
                            ? "border-primary bg-primary/10"
                            : "border-border hover:bg-muted/50 hover:border-muted-foreground/30"
                        )}
                      >
                        {ACTION_TYPE_LABELS[a]}
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium block">Action config *</label>
              {actionType === "discord" && (
                <div>
                  <input
                    type="url"
                    className={cn(
                      "w-full rounded-lg border px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary",
                      fieldErrors.webhook_url ? "border-destructive bg-transparent" : "border-input bg-transparent"
                    )}
                    placeholder="https://discord.com/api/webhooks/..."
                    value={(actionConfig as { webhook_url?: string }).webhook_url ?? ""}
                    onChange={(e) => {
                      setActionConfig({ webhook_url: e.target.value });
                      if (fieldErrors.webhook_url) setFieldErrors((prev) => ({ ...prev, webhook_url: "" }));
                    }}
                    aria-invalid={!!fieldErrors.webhook_url}
                  />
                  {fieldErrors.webhook_url && (
                    <p className="text-sm text-destructive mt-1">{fieldErrors.webhook_url}</p>
                  )}
                </div>
              )}
              {actionType === "telegram" && (
                <div className="space-y-2">
                  <div className="relative">
                    <input
                      type={showBotToken ? "text" : "password"}
                      className={cn(
                        "w-full rounded-lg border px-3 py-2 pr-10 text-sm focus:outline-none focus:ring-2 focus:ring-primary",
                        fieldErrors.bot_token ? "border-destructive bg-transparent" : "border-input bg-transparent"
                      )}
                      placeholder="Bot token"
                      value={(actionConfig as { bot_token?: string }).bot_token ?? ""}
                      onChange={(e) => {
                        setActionConfig((c) => ({
                          ...(c as ActionConfigTelegram),
                          bot_token: e.target.value,
                        }));
                        if (fieldErrors.bot_token) setFieldErrors((prev) => ({ ...prev, bot_token: "" }));
                      }}
                      aria-invalid={!!fieldErrors.bot_token}
                    />
                    <button
                      type="button"
                      onClick={() => setShowBotToken((v) => !v)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded text-muted-foreground hover:text-foreground hover:bg-muted/50 focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-0"
                      aria-label={showBotToken ? "Hide bot token" : "Show bot token"}
                    >
                      {showBotToken ? (
                        <EyeOff className="h-4 w-4" />
                      ) : (
                        <Eye className="h-4 w-4" />
                      )}
                    </button>
                  </div>
                  {fieldErrors.bot_token && (
                    <p className="text-sm text-destructive mt-1">{fieldErrors.bot_token}</p>
                  )}
                  <div>
                    <input
                      type="text"
                      className={cn(
                        "w-full rounded-lg border px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary",
                        fieldErrors.chat_id ? "border-destructive bg-transparent" : "border-input bg-transparent"
                      )}
                      placeholder="Chat ID"
                      value={(actionConfig as { chat_id?: string }).chat_id ?? ""}
                      onChange={(e) => {
                        setActionConfig((c) => ({
                          ...(c as ActionConfigTelegram),
                          chat_id: e.target.value,
                        }));
                        if (fieldErrors.chat_id) setFieldErrors((prev) => ({ ...prev, chat_id: "" }));
                      }}
                      aria-invalid={!!fieldErrors.chat_id}
                    />
                    {fieldErrors.chat_id && (
                      <p className="text-sm text-destructive mt-1">{fieldErrors.chat_id}</p>
                    )}
                  </div>
                </div>
              )}
              {actionType === "device_command" && (
                <div className="space-y-2">
                  <div>
                    <label className="text-xs text-muted-foreground">Target device (actuator)</label>
                    <div className="relative mt-0.5">
                      <select
                        className={cn(
                          "h-10 w-full appearance-none rounded-lg border bg-card pr-9 pl-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary",
                          fieldErrors.target_device ? "border-destructive" : "border-input"
                        )}
                        value={(actionConfig as { target_device_uuid?: string }).target_device_uuid ?? ""}
                        onChange={(e) => {
                          setActionConfig((prev) => ({
                            ...prev,
                            target_device_uuid: e.target.value,
                            command: (prev as { command?: string }).command ?? "",
                          }));
                          if (fieldErrors.target_device) setFieldErrors((prev) => ({ ...prev, target_device: "" }));
                        }}
                        aria-invalid={!!fieldErrors.target_device}
                      >
                        <option value="">— Select device —</option>
                        {devices
                          .filter((d) => d.device_type === "actuator")
                          .map((d) => (
                            <option key={d.uuid} value={d.uuid}>
                              {d.name}
                            </option>
                          ))}
                      </select>
                      <ChevronDown className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    </div>
                    {fieldErrors.target_device && (
                      <p className="text-sm text-destructive mt-1">{fieldErrors.target_device}</p>
                    )}
                  </div>
                  <div>
                    <input
                      type="text"
                      className={cn(
                        "w-full rounded-lg border px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary",
                        fieldErrors.command ? "border-destructive bg-transparent" : "border-input bg-transparent"
                      )}
                      placeholder="Command (e.g. ON, OFF)"
                      value={(actionConfig as { command?: string }).command ?? ""}
                      onChange={(e) => {
                        setActionConfig((prev) => ({
                          ...prev,
                          target_device_uuid: (prev as { target_device_uuid?: string }).target_device_uuid ?? "",
                          command: e.target.value,
                        }));
                        if (fieldErrors.command) setFieldErrors((prev) => ({ ...prev, command: "" }));
                      }}
                      aria-invalid={!!fieldErrors.command}
                    />
                    {fieldErrors.command && (
                      <p className="text-sm text-destructive mt-1">{fieldErrors.command}</p>
                    )}
                  </div>
                </div>
              )}
            </div>
          </section>

          <section className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium block">Condition *</label>
              {sourceEvent === "sensor_reading" && (
                <>
                  {sensorMeasurementsWithRange.length > 0 ? (
                    <div className="space-y-3">
                      <div>
                        <label className="text-xs text-muted-foreground block mb-1.5">Measurement</label>
                        <div className="rounded-lg border border-input p-2">
                          <div className="flex flex-wrap gap-2">
                            {sensorMeasurementsWithRange.map((m) => {
                              const isSelected =
                                (condition as { measurement?: string }).measurement === m.name;
                              return (
                                <button
                                  key={m.name}
                                  type="button"
                                  onClick={() => {
                                    const range =
                                      m.min_reading !== undefined && m.max_reading !== undefined
                                        ? { min: m.min_reading, max: m.max_reading }
                                        : null;
                                    setCondition((c) => ({
                                      ...(c as ConditionSensorReading),
                                      measurement: m.name,
                                      value: range
                                        ? Math.max(range.min, Math.min(range.max, Number((c as ConditionSensorReading).value) || 0))
                                        : (c as ConditionSensorReading).value ?? 0,
                                    }));
                                  }}
                                  className={cn(
                                    "flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg border-2 transition-colors text-sm font-medium capitalize",
                                    isSelected
                                      ? "border-primary bg-primary/10"
                                      : "border-border hover:bg-muted/50 hover:border-muted-foreground/30"
                                  )}
                                >
                                  {m.name}
                                </button>
                              );
                            })}
                          </div>
                          <p className="text-xs text-muted-foreground mt-1.5">
                            Grandezas do sensor selecionado.
                          </p>
                        </div>
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        {(condition as { measurement?: string }).measurement && (
                          <span className="text-sm font-medium capitalize">
                            {(condition as { measurement?: string }).measurement}
                          </span>
                        )}
                        <div className="relative w-full sm:min-w-[5rem] sm:w-auto">
                          <select
                            className="h-10 w-full appearance-none rounded-lg border border-input bg-card pr-9 pl-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                            value={(condition as { operator?: string }).operator ?? ">="}
                            onChange={(e) =>
                              setCondition((c) => ({
                                ...(c as ConditionSensorReading),
                                operator: e.target.value as ConditionSensorReading["operator"],
                              }))
                            }
                          >
                            {OPERATORS.map((op) => (
                              <option key={op} value={op}>
                                {op}
                              </option>
                            ))}
                          </select>
                          <ChevronDown className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        </div>
                        <input
                          type="number"
                          className="w-full sm:w-28 rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary min-w-0 h-10"
                          value={(condition as { value?: number }).value ?? 0}
                          min={currentMeasurementRange?.min_reading}
                          max={currentMeasurementRange?.max_reading}
                          step={currentMeasurementRange?.min_reading != null ? "any" : undefined}
                          onChange={(e) =>
                            setCondition((c) => ({
                              ...(c as ConditionSensorReading),
                              value: Number(e.target.value),
                            }))
                          }
                        />
                        {currentMeasurementRange?.min_reading !== undefined &&
                          currentMeasurementRange?.max_reading !== undefined && (
                            <span className="text-xs text-muted-foreground whitespace-nowrap">
                              Range: {currentMeasurementRange.min_reading}–{currentMeasurementRange.max_reading}
                              {currentMeasurementRange.unit ? ` ${currentMeasurementRange.unit}` : ""}
                            </span>
                          )}
                      </div>
                    </div>
                  ) : (
                    <p className="text-sm text-muted-foreground">
                      Select a sensor device above to configure the condition.
                    </p>
                  )}
                </>
              )}
              {sourceEvent === "device_command" && (
                <div className="space-y-2">
                  <input
                    type="text"
                    className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                    placeholder="command (e.g. ON, OFF)"
                    value={(condition as { command?: string }).command ?? ""}
                    onChange={(e) => setCondition({ command: e.target.value })}
                  />
                  <p className="text-xs text-muted-foreground">
                    Or use command_pattern as JSON in advanced (e.g. &#123;&quot;action&quot;: &quot;set_temp&quot;, &quot;value&quot;: 45&#125;).
                  </p>
                </div>
              )}
              {sourceEvent === "schedule" && (
                <div className="grid gap-2">
                  <div className="rounded-lg border border-input p-2">
                    <div className="flex flex-wrap gap-2">
                      {DAYS.map((d) => {
                        const arr = (condition as { days_of_week?: number[] }).days_of_week ?? [];
                        const isSelected = arr.includes(d.value);
                        return (
                          <button
                            key={d.value}
                            type="button"
                            onClick={() => {
                              const next = isSelected
                                ? arr.filter((x) => x !== d.value)
                                : [...arr, d.value].sort((a, b) => a - b);
                              setCondition((c) => ({
                                ...(c as ConditionSchedule),
                                days_of_week: next,
                              }));
                            }}
                            className={cn(
                              "min-w-[2.5rem] px-2 py-2 rounded-lg border-2 transition-colors text-sm font-medium",
                              isSelected
                                ? "border-primary bg-primary/10"
                                : "border-border hover:bg-muted/50 hover:border-muted-foreground/30"
                            )}
                          >
                            {d.label}
                          </button>
                        );
                      })}
                    </div>
                  </div>
                  <div className="flex flex-col sm:flex-row flex-wrap gap-2">
                    <input
                      type="time"
                      className="w-full sm:w-auto min-w-0 rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                      value={(condition as { time?: string }).time ?? "08:00"}
                      onChange={(e) =>
                        setCondition((c) => ({
                          ...(c as ConditionSchedule),
                          time: e.target.value,
                        }))
                      }
                    />
                    <input
                      type="date"
                      className="w-full sm:w-auto min-w-0 rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                      placeholder="Start date"
                      value={(condition as { start_date?: string }).start_date ?? ""}
                      onChange={(e) =>
                        setCondition((c) => ({
                          ...(c as ConditionSchedule),
                          start_date: e.target.value,
                        }))
                      }
                    />
                    <input
                      type="date"
                      className="w-full sm:w-auto min-w-0 rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                      placeholder="End date"
                      value={(condition as { end_date?: string }).end_date ?? ""}
                      onChange={(e) =>
                        setCondition((c) => ({
                          ...(c as ConditionSchedule),
                          end_date: e.target.value,
                        }))
                      }
                    />
                  </div>
                </div>
              )}
            </div>
          </section>

          <section className="space-y-4">
            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="is_active"
                checked={isActive}
                onChange={(e) => setIsActive(e.target.checked)}
                className="h-4 w-4 rounded border-2 border-input bg-background accent-primary focus:ring-2 focus:ring-primary focus:ring-offset-0 cursor-pointer"
              />
              <label htmlFor="is_active" className="text-sm font-medium cursor-pointer">
                Active
              </label>
            </div>

            <div className="flex gap-2">
              <Button type="submit" disabled={submitting}>
                {submitting ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Saving...
                  </>
                ) : isEdit ? (
                  "Update"
                ) : (
                  "Create"
                )}
              </Button>
              <Button type="button" variant="outline" asChild>
                <Link to="/triggers/list">Cancel</Link>
              </Button>
            </div>
          </section>
        </form>
      </div>
    </div>
  );
}
