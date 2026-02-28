import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useAuth } from "@/context/AuthContext";
import { provisioningApi } from "@/services/provisioningApi";
import { SerialConsole } from "@/components/provisioning/SerialConsole";
import { Button } from "@/components/ui/button";
import { Loader2, Cpu, Wifi, Server, ArrowLeft } from "lucide-react";
import { toast } from "sonner";
import {
  BAUDRATES,
  type SerialPortInfo,
  type ProbeDeviceResult,
  type AdoptDeviceInput,
  type DeviceInfoInput,
} from "@/types/provisioning";

const DeviceAdoptionWizard = () => {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { token, logout } = useAuth();

  const [ports, setPorts] = useState<SerialPortInfo[]>([]);
  const [portsLoading, setPortsLoading] = useState(false);
  const [port, setPort] = useState("");
  const [baudRate, setBaudRate] = useState(115200);

  const [probeResult, setProbeResult] = useState<ProbeDeviceResult | null>(null);
  const [probing, setProbing] = useState(false);
  const [probeError, setProbeError] = useState<string | null>(null);

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [wifiSsid, setWifiSsid] = useState("");
  const [wifiPassword, setWifiPassword] = useState("");

  const [brokerInfo, setBrokerInfo] = useState<{
    host: string;
    port: number;
    broker_url: string;
  } | null>(null);
  const [brokerLoading, setBrokerLoading] = useState(false);

  const [adopting, setAdopting] = useState(false);

  useEffect(() => {
    if (!token) return;
    setBrokerLoading(true);
    provisioningApi.getDefaultBroker(token).then((r) => {
      setBrokerLoading(false);
      if (r.success && r.data) {
        setBrokerInfo({
          host: r.data.host,
          port: r.data.port,
          broker_url: r.data.broker_url,
        });
      }
    });
  }, [token]);

  const refreshPorts = async () => {
    setPortsLoading(true);
    const result = await provisioningApi.listSerialPorts();
    setPortsLoading(false);
    if (result.success && result.data) {
      setPorts(result.data);
      if (result.data.length > 0 && !port) {
        setPort(result.data[0].port_name);
      }
    }
  };

  useEffect(() => {
    refreshPorts();
  }, []);

  const handleProbe = async () => {
    setProbeError(null);
    if (!port) {
      toast.error("Select a serial port.");
      return;
    }
    setProbeResult(null);
    setProbing(true);
    const result = await provisioningApi.probeDevice({ port, baud_rate: baudRate });
    setProbing(false);
    if (!result.success) {
      setProbeError(result.message ?? "Probe failed.");
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
      }
      return;
    }
    if (result.data) {
      setProbeResult(result.data);
      if (result.data.can_adopt) {
        setName(result.data.device_info.boarder_type ?? result.data.device_info.model ?? "Device");
      }
    }
  };

  const handleAdopt = async () => {
    if (!token || !uuid || !probeResult) return;
    if (!name.trim()) {
      toast.error("Device name is required.");
      return;
    }
    if (!brokerInfo) {
      toast.error("No default broker configured. Create and set a broker as default first.");
      return;
    }
    if (!wifiSsid.trim()) {
      toast.error("WiFi SSID is required.");
      return;
    }
    if (!wifiPassword) {
      toast.error("WiFi password is required.");
      return;
    }
    setAdopting(true);

    const deviceInfo: DeviceInfoInput = {
      device_type: probeResult.device_info.device_type,
      model: probeResult.device_info.boarder_type ?? probeResult.device_info.model ?? "",
      mac_address: probeResult.device_info.mac_address,
      sensor_type: probeResult.device_info.sensor_type ?? undefined,
      actuator_type: probeResult.device_info.actuator_type ?? undefined,
      device_scale: probeResult.device_info.device_scale ?? undefined,
      firmware_version: probeResult.device_info.firmware_version ?? undefined,
    };

    const payload: AdoptDeviceInput = {
      port,
      baud_rate: baudRate,
      name: name.trim(),
      location_uuid: uuid,
      description: description.trim() || undefined,
      broker_url: brokerInfo.broker_url,
      wifi_ssid: wifiSsid.trim(),
      wifi_password: wifiPassword,
      device_info: deviceInfo,
    };

    const result = await provisioningApi.adoptDevice(token, payload);
    setAdopting(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Adoption failed.");
      return;
    }

    toast.success("Device adopted successfully.");
    if (result.data?.uuid) {
      navigate(`/devices/${result.data.uuid}/dashboard`);
    } else {
      navigate(`/locations/${uuid}`);
    }
  };

  return (
    <div className="min-h-screen bg-secondary/20 text-foreground">
      <div className="max-w-4xl mx-auto py-10 px-4 space-y-6">
        <div className="flex items-center gap-4">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => navigate(`/locations/${uuid}`)}
          aria-label="Back"
        >
          <ArrowLeft className="w-5 h-5" />
        </Button>
        <div>
          <h1 className="text-2xl font-semibold">Device Adoption</h1>
          <p className="text-sm text-muted-foreground">
            Connect the device via USB and fill in the fields below.
          </p>
        </div>
      </div>

      <div className="bg-background border border-border rounded-xl p-6 shadow-sm space-y-6">
        {/* Serial port */}
        <div className="space-y-3">
          <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
            <Cpu className="w-4 h-4" />
            Serial Port and Baud Rate
          </div>
          <div className="flex flex-wrap gap-4 items-end">
            <div className="space-y-2 min-w-[200px]">
              <label className="text-sm font-medium">Port</label>
              <select
                value={port}
                onChange={(e) => setPort(e.target.value)}
                className="h-10 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary disabled:cursor-not-allowed disabled:opacity-50"
                disabled={portsLoading}
              >
                <option value="">Select a port</option>
                {ports.map((p) => (
                  <option key={p.port_name} value={p.port_name}>
                    {p.port_name}
                    {p.manufacturer || p.product ? ` (${p.manufacturer ?? p.product ?? ""})` : ""}
                  </option>
                ))}
              </select>
            </div>
            <div className="space-y-2 min-w-[140px]">
              <label className="text-sm font-medium">Baud rate</label>
              <select
                value={baudRate}
                onChange={(e) => setBaudRate(Number(e.target.value))}
                className="h-10 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
              >
                {BAUDRATES.map((b) => (
                  <option key={b} value={b}>
                    {b}
                  </option>
                ))}
              </select>
            </div>
            <Button
              variant="outline"
              size="default"
              onClick={refreshPorts}
              disabled={portsLoading}
            >
              {portsLoading ? <Loader2 className="w-4 h-4 animate-spin" /> : "Refresh"}
            </Button>
            <Button
              onClick={handleProbe}
              disabled={!port || probing}
            >
              {probing ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Detecting…
                </>
              ) : (
                "Detect"
              )}
            </Button>
          </div>
        </div>

        {/* SerialConsole */}
        <SerialConsole />

        {probeError && (
          <div className="rounded-lg border border-red-200 dark:border-red-900 bg-red-50 dark:bg-red-950/50 px-3 py-2">
            <p className="text-sm font-medium text-red-700 dark:text-red-400">{probeError}</p>
          </div>
        )}

        {probeResult && (
          <>
            {/* Device name and description */}
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-sm font-semibold text-foreground">Device</div>
              <div className="space-y-2">
                <div>
                  <label className="text-sm font-medium">Name *</label>
                  <input
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="e.g. Living Room Sensor"
                    className="mt-1 h-10 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                  />
                </div>
                <div>
                  <label className="text-sm font-medium">Description</label>
                  <input
                    type="text"
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    placeholder="Optional"
                    className="mt-1 h-10 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                  />
                </div>
              </div>
            </div>

            {/* WiFi */}
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
                <Wifi className="w-4 h-4" />
                WiFi
              </div>
              <div className="grid gap-2 sm:grid-cols-2">
                <div>
                  <label className="text-sm font-medium">SSID *</label>
                  <input
                    type="text"
                    value={wifiSsid}
                    onChange={(e) => setWifiSsid(e.target.value)}
                    placeholder="Network name"
                    className="mt-1 h-10 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                  />
                </div>
                <div>
                  <label className="text-sm font-medium">Password *</label>
                  <input
                    type="password"
                    value={wifiPassword}
                    onChange={(e) => setWifiPassword(e.target.value)}
                    placeholder="Password"
                    className="mt-1 h-10 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                  />
                </div>
              </div>
            </div>

            {/* Broker (read-only) */}
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
                <Server className="w-4 h-4" />
                Broker
              </div>
              {brokerLoading ? (
                <p className="text-sm text-muted-foreground">Loading…</p>
              ) : brokerInfo ? (
                <div className="flex h-10 items-center rounded-lg border border-input bg-muted/50 px-3 text-sm text-muted-foreground">
                  {brokerInfo.broker_url}
                  <span className="ml-2 text-xs">
                    ({brokerInfo.host}:{brokerInfo.port})
                  </span>
                </div>
              ) : (
                <div className="rounded-lg border border-red-200 dark:border-red-900 bg-red-50 dark:bg-red-950/50 px-3 py-2">
                <p className="text-sm font-medium text-red-700 dark:text-red-400">
                  No default broker configured. Create and set a broker as default first.
                </p>
              </div>
              )}
            </div>

            {probeResult.can_adopt ? (
              <Button
                onClick={handleAdopt}
                disabled={adopting || !brokerInfo || !wifiSsid.trim() || !wifiPassword}
                className="w-full"
              >
                {adopting ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Adopting…
                  </>
                ) : (
                  "Adopt Device"
                )}
              </Button>
            ) : (
              <div className="rounded-lg border border-amber-200 dark:border-amber-900 bg-amber-50 dark:bg-amber-950/50 px-3 py-2">
              <p className="text-sm font-medium text-amber-800 dark:text-amber-400">
                {probeResult.message ?? "Device cannot be adopted (may already be adopted)."}
              </p>
            </div>
            )}
          </>
        )}
      </div>
      </div>
    </div>
  );
};

export default DeviceAdoptionWizard;
