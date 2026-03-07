import { useState, useMemo, useEffect } from "react";
import { toast } from "sonner";
import { Loader2, Server, Lock, Settings, MessageSquare, ArrowLeft } from "lucide-react";
import { mqttBrokerApi } from "@/services/mqttBrokerApi";
import { useAuth } from "@/context/AuthContext";
import { useNavigate } from "react-router-dom";
import type { MqttBrokerCreateInput } from "@/types/mqttBroker";
import { Button } from "@/components/ui/button";

const MqttBrokerCreate = () => {
  const { token, logout } = useAuth();
  const navigate = useNavigate();
  const [form, setForm] = useState<MqttBrokerCreateInput>({
    name: "",
    description: "",
    host: "",
    port: 1883,
    username: "",
    password: "",
    use_tls: false,
    insecure_tls: false,
    client_id: "",
    keep_alive_interval: 60,
    clean_session: true,
    connection_timeout_secs: 30,
    operation_timeout_secs: 30,
    last_will_topic: "",
    last_will_message: "",
    last_will_qos: 0,
    last_will_retain: false,
    is_default: false,
  });
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!token) {
      logout();
    }
  }, [token, logout]);

  const isValid = useMemo(() => {
    const nameOk = form.name.trim().length > 0;
    const hostOk = form.host.trim().length > 0;
    const portOk = form.port !== undefined && form.port > 0 && form.port <= 65535;
    return nameOk && hostOk && portOk;
  }, [form.name, form.host, form.port]);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token) {
      logout();
      return;
    }
    if (!isValid) {
      toast.error("Please fill in the required fields.");
      return;
    }

    setSubmitting(true);
    const payload: MqttBrokerCreateInput = {
      name: form.name.trim(),
      description: form.description?.trim() || undefined,
      host: form.host.trim(),
      port: form.port,
      username: form.username?.trim() || undefined,
      password: form.password || undefined,
      use_tls: form.use_tls || undefined,
      insecure_tls: form.insecure_tls || undefined,
      client_id: form.client_id?.trim() || undefined,
      keep_alive_interval: form.keep_alive_interval || undefined,
      clean_session: form.clean_session !== undefined ? form.clean_session : undefined,
      connection_timeout_secs: form.connection_timeout_secs || undefined,
      operation_timeout_secs: form.operation_timeout_secs || undefined,
      last_will_topic: form.last_will_topic?.trim() || undefined,
      last_will_message: form.last_will_message?.trim() || undefined,
      last_will_qos: form.last_will_qos !== undefined ? form.last_will_qos : undefined,
      last_will_retain: form.last_will_retain !== undefined ? form.last_will_retain : undefined,
      is_default: form.is_default || undefined,
    };

    const result = await mqttBrokerApi.createMqttBroker(token, payload);
    setSubmitting(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to create broker.");
      return;
    }

    toast.success("Broker created successfully.");
    navigate("/mqtt-brokers/list");
  };

  return (
    <div className="min-h-screen bg-secondary/20 text-foreground">
      <div className="max-w-4xl mx-auto py-10 px-4 space-y-6">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate("/mqtt-brokers/list")}
            aria-label="Back"
          >
            <ArrowLeft className="w-5 h-5" />
          </Button>
          <div>
            <h1 className="text-2xl font-semibold">Broker</h1>
            <p className="text-muted-foreground text-sm">
              Create a new MQTT broker configuration.
            </p>
          </div>
        </div>

        <form
          onSubmit={onSubmit}
          className="bg-background border border-border rounded-xl p-6 shadow-sm space-y-6"
        >
          {/* Basic Information */}
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
              <Server className="w-4 h-4" />
              Basic Information
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Name *</label>
              <input
                value={form.name}
                onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                placeholder="E.g.: My MQTT Broker"
                required
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Description</label>
              <textarea
                value={form.description}
                onChange={(e) => setForm((prev) => ({ ...prev, description: e.target.value }))}
                className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                placeholder="Additional notes"
                rows={2}
              />
            </div>
          </div>

          {/* Connection Settings */}
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
              <Server className="w-4 h-4" />
              Connection Settings
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Host *</label>
                <input
                  value={form.host}
                  onChange={(e) => setForm((prev) => ({ ...prev, host: e.target.value }))}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                  placeholder="broker.example.com"
                  required
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Port *</label>
                <input
                  type="number"
                  min="1"
                  max="65535"
                  value={form.port}
                  onChange={(e) => setForm((prev) => ({ ...prev, port: parseInt(e.target.value) || 1883 }))}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                  required
                />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Username</label>
                <input
                  value={form.username}
                  onChange={(e) => setForm((prev) => ({ ...prev, username: e.target.value }))}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                  placeholder="Optional"
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Password</label>
                <input
                  type="password"
                  value={form.password}
                  onChange={(e) => setForm((prev) => ({ ...prev, password: e.target.value }))}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                  placeholder="Optional"
                />
              </div>
            </div>
          </div>

          {/* TLS/SSL Settings */}
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
              <Lock className="w-4 h-4" />
              TLS/SSL Settings
            </div>
            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="use_tls"
                checked={form.use_tls}
                onChange={(e) => setForm((prev) => ({ ...prev, use_tls: e.target.checked }))}
                className="w-4 h-4 rounded border-input"
              />
              <label htmlFor="use_tls" className="text-sm font-medium cursor-pointer">
                Use TLS/SSL
              </label>
            </div>
            {form.use_tls && (
              <div className="space-y-4 pl-6 border-l-2 border-border">
                <div className="space-y-3">
                  <div className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      id="insecure_tls"
                      checked={form.insecure_tls}
                      onChange={(e) => setForm((prev) => ({ ...prev, insecure_tls: e.target.checked }))}
                      className="w-4 h-4 rounded border-input"
                    />
                    <label htmlFor="insecure_tls" className="text-sm font-medium cursor-pointer">
                      Accept insecure certificates (not recommended)
                    </label>
                  </div>

                  {/* CA Certificate */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">CA Certificate Path</label>
                    <input
                      type="text"
                      value={form.ca_certificate_path || ""}
                      onChange={(e) => setForm((prev) => ({ ...prev, ca_certificate_path: e.target.value || undefined }))}
                      className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                      placeholder="Path to CA certificate file (optional)"
                    />
                    <p className="text-xs text-muted-foreground">
                      For CA signed or self-signed certificates
                    </p>
                  </div>

                  {/* Client Certificate (mTLS) */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">Client Certificate Path (mTLS)</label>
                    <input
                      type="text"
                      value={form.client_certificate_path || ""}
                      onChange={(e) => setForm((prev) => ({ ...prev, client_certificate_path: e.target.value || undefined }))}
                      className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                      placeholder="Path to client certificate file (optional)"
                    />
                  </div>

                  {/* Client Key (mTLS) */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">Client Key Path (mTLS)</label>
                    <input
                      type="text"
                      value={form.client_key_path || ""}
                      onChange={(e) => setForm((prev) => ({ ...prev, client_key_path: e.target.value || undefined }))}
                      className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                      placeholder="Path to client private key file (optional)"
                    />
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Advanced Settings */}
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
              <Settings className="w-4 h-4" />
              Advanced Settings
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Client ID</label>
                <input
                  value={form.client_id}
                  onChange={(e) => setForm((prev) => ({ ...prev, client_id: e.target.value }))}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                  placeholder="Auto-generated if empty"
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Keep Alive (seconds)</label>
                <input
                  type="number"
                  min="1"
                  max="65535"
                  value={form.keep_alive_interval}
                  onChange={(e) => setForm((prev) => ({ ...prev, keep_alive_interval: parseInt(e.target.value) || 60 }))}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Connection Timeout (seconds)</label>
                <input
                  type="number"
                  min="1"
                  max="300"
                  value={form.connection_timeout_secs}
                  onChange={(e) => setForm((prev) => ({ ...prev, connection_timeout_secs: parseInt(e.target.value) || 30 }))}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Operation Timeout (seconds)</label>
                <input
                  type="number"
                  min="1"
                  max="300"
                  value={form.operation_timeout_secs}
                  onChange={(e) => setForm((prev) => ({ ...prev, operation_timeout_secs: parseInt(e.target.value) || 30 }))}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
            </div>
            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="clean_session"
                checked={form.clean_session}
                onChange={(e) => setForm((prev) => ({ ...prev, clean_session: e.target.checked }))}
                className="w-4 h-4 rounded border-input"
              />
              <label htmlFor="clean_session" className="text-sm font-medium cursor-pointer">
                Clean Session
              </label>
            </div>
          </div>

          {/* Last Will and Testament */}
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
              <MessageSquare className="w-4 h-4" />
              Last Will and Testament (LWT)
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Last Will Topic</label>
              <input
                value={form.last_will_topic}
                onChange={(e) => setForm((prev) => ({ ...prev, last_will_topic: e.target.value }))}
                className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                placeholder="device/status"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Last Will Message</label>
              <textarea
                value={form.last_will_message}
                onChange={(e) => setForm((prev) => ({ ...prev, last_will_message: e.target.value }))}
                className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                placeholder="offline"
                rows={2}
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Last Will QoS</label>
                <input
                  type="number"
                  min="0"
                  max="2"
                  value={form.last_will_qos}
                  onChange={(e) => setForm((prev) => ({ ...prev, last_will_qos: parseInt(e.target.value) || 0 }))}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
              <div className="flex items-center space-x-2 pt-8">
                <input
                  type="checkbox"
                  id="last_will_retain"
                  checked={form.last_will_retain}
                  onChange={(e) => setForm((prev) => ({ ...prev, last_will_retain: e.target.checked }))}
                  className="w-4 h-4 rounded border-input"
                />
                <label htmlFor="last_will_retain" className="text-sm font-medium cursor-pointer">
                  Retain Last Will
                </label>
              </div>
            </div>
          </div>

          {/* Default Broker */}
          <div className="flex items-center space-x-2">
            <input
              type="checkbox"
              id="is_default"
              checked={form.is_default}
              onChange={(e) => setForm((prev) => ({ ...prev, is_default: e.target.checked }))}
              className="w-4 h-4 rounded border-input"
            />
            <label htmlFor="is_default" className="text-sm font-medium cursor-pointer">
              Set as default broker
            </label>
          </div>

          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => navigate("/mqtt-brokers/list")}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              variant="outline"
              size="sm"
              disabled={!isValid || submitting}
            >
              {submitting && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
              Create Broker
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default MqttBrokerCreate;

