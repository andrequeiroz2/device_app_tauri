import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useAuth } from "@/context/AuthContext";
import { provisioningApi } from "@/services/provisioningApi";
import { SerialConsole } from "@/components/provisioning/SerialConsole";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Loader2, Cpu, Wifi, Server, ArrowLeft, CircuitBoard, CheckCircle2, AlertCircle, Fingerprint, Layers, Thermometer, Eye, EyeOff } from "lucide-react";
import { toast } from "sonner";
import { motion, AnimatePresence } from "framer-motion";
import {
  BAUDRATES,
  type SerialPortInfo,
  type ProbeDeviceResult,
  type AdoptDeviceInput,
  type DeviceInfoInput,
} from "@/types/provisioning";
import { deviceApi } from "@/services/deviceApi";
import type { DevicePublic } from "@/types/device";

const DeviceAdoptionWizard = () => {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { token, logout, user } = useAuth();

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
  const [showWifiPassword, setShowWifiPassword] = useState(false);
  const [divergenceDialogOpen, setDivergenceDialogOpen] = useState(false);
  const [otherUserPopupOpen, setOtherUserPopupOpen] = useState(false);
  const [deviceFromDb, setDeviceFromDb] = useState<DevicePublic | null | undefined>(undefined);

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
      if (result.data.can_adopt && token) {
        const mac = result.data.device_info.mac_address;
        console.debug("[DeviceAdoption] can_adopt=true, checking MAC in DB:", mac);
        const check = await provisioningApi.checkDeviceByMacForAdoption(token, mac);
        console.debug(
          "[DeviceAdoption] checkDeviceByMacForAdoption result:",
          "success:",
          check.success,
          "exists:",
          check.data?.exists,
          "owner_user_uuid:",
          check.data?.owner_user_uuid ? `${check.data.owner_user_uuid.slice(0, 8)}...` : "-",
          "message:",
          check.message ?? "-"
        );
        if (check.success && check.data?.exists) {
          const ownerUuid = check.data.owner_user_uuid?.trim();
          const loggedUuid = user?.uuid?.trim();
          const sameUser = ownerUuid && loggedUuid && ownerUuid === loggedUuid;
          console.debug(
            "[DeviceAdoption] MAC in DB:",
            mac,
            "ownerUuid:",
            ownerUuid ? `${ownerUuid.slice(0, 8)}...` : "-",
            "loggedUuid:",
            loggedUuid ? `${loggedUuid.slice(0, 8)}...` : "-",
            "sameUser:",
            sameUser,
            "->",
            sameUser ? "divergence popup" : "other user popup"
          );
          setProbeResult(null);
          if (sameUser) {
            setDivergenceDialogOpen(true);
          } else {
            setOtherUserPopupOpen(true);
          }
          return;
        }
        console.debug("[DeviceAdoption] MAC not in DB or check failed, proceeding with adoption");
      }
      setProbeResult(result.data);
      if (result.data.can_adopt) {
        setName(result.data.device_info.boarder_type ?? result.data.device_info.model ?? "Device");
        setDeviceFromDb(undefined);
        setOtherUserPopupOpen(false);
      } else {
        const deviceUserUuid = (result.data.device_info as { user_uuid?: string }).user_uuid?.trim();
        const loggedUserUuid = user?.uuid?.trim();
        console.debug(
          "[DeviceAdoption] can_adopt=false:",
          "deviceUserUuid:",
          deviceUserUuid ? `${deviceUserUuid.slice(0, 8)}...` : "-",
          "loggedUserUuid:",
          loggedUserUuid ? `${loggedUserUuid.slice(0, 8)}...` : "-"
        );
        if (!deviceUserUuid || deviceUserUuid !== loggedUserUuid) {
          console.debug("[DeviceAdoption] different user or no user_uuid -> other user popup");
          setProbeResult(null);
          setDeviceFromDb(undefined);
          setOtherUserPopupOpen(true);
          return;
        }
        if (token) {
          setDeviceFromDb(undefined);
          const r = await deviceApi.getDeviceByMac(
            token,
            result.data.device_info.mac_address
          );
          if (r.success && r.data) {
            console.debug("[DeviceAdoption] can_adopt=false same user: device found in DB");
            setDeviceFromDb(r.data);
          } else {
            console.debug("[DeviceAdoption] can_adopt=false same user: device NOT in DB -> other user popup");
            setProbeResult(null);
            setOtherUserPopupOpen(true);
          }
        }
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

  const step = adopting ? 3 : probeResult?.can_adopt ? 2 : 1;

  const steps = [
    { label: "Connect" },
    { label: "Configure" },
    { label: "Adopt" },
  ];

  return (
    <div className="min-h-screen bg-secondary/20 text-foreground">
      <motion.div
        className="max-w-4xl mx-auto py-10 px-4 space-y-6"
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.25, ease: "easeOut" }}
      >
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

      {/* Step indicator */}
      <div className="flex items-center gap-0">
        {steps.map((s, i) => {
          const stepNum = i + 1;
          const isCompleted = step > stepNum;
          const isActive = step === stepNum;
          return (
            <div key={s.label} className="flex items-center flex-1 last:flex-none">
              <div className="flex flex-col items-center gap-1">
                <div
                  className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold border-2 transition-colors ${
                    isCompleted
                      ? "bg-primary border-primary text-primary-foreground"
                      : isActive
                      ? "border-primary text-primary bg-background"
                      : "border-border text-muted-foreground bg-background"
                  }`}
                >
                  {isCompleted ? <CheckCircle2 className="w-4 h-4" /> : stepNum}
                </div>
                <span
                  className={`text-xs font-medium ${
                    isActive ? "text-primary" : isCompleted ? "text-foreground" : "text-muted-foreground"
                  }`}
                >
                  {s.label}
                </span>
              </div>
              {i < steps.length - 1 && (
                <div
                  className={`h-0.5 flex-1 mx-2 mb-5 transition-colors ${
                    step > stepNum ? "bg-primary" : "bg-border"
                  }`}
                />
              )}
            </div>
          );
        })}
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

        <AnimatePresence>
          {probeError && (
            <motion.div
              key="probe-error"
              initial={{ opacity: 0, y: -8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              transition={{ duration: 0.2, ease: "easeOut" }}
              className="rounded-lg border border-red-200 dark:border-red-900 bg-red-50 dark:bg-red-950/50 px-3 py-2 space-y-0.5"
            >
              <p className="text-sm font-medium text-red-700 dark:text-red-400">{probeError}</p>
              {probeError.toLowerCase().includes("not responding") && (
                <p className="text-xs text-red-600 dark:text-red-500">
                  Make sure the device is connected and in configuration mode (not yet adopted).
                  If it was recently adopted, it may be starting up — access it through its dashboard instead.
                </p>
              )}
            </motion.div>
          )}
        </AnimatePresence>

        <AnimatePresence>
          {probeResult && (
            <motion.div
              key="probe-result"
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 12 }}
              transition={{ duration: 0.25, ease: "easeOut" }}
              className="space-y-6"
            >
            {/* Device detected - card compacta quando Already adopted */}
            <div className="rounded-xl border border-border bg-muted/30 p-4 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
                  <CircuitBoard className="w-4 h-4" />
                  {probeResult.can_adopt ? "Detected Device" : "Device detected"}
                </div>
                {probeResult.can_adopt ? (
                  <span className="inline-flex items-center gap-1.5 rounded-full bg-green-100 dark:bg-green-950/60 px-2.5 py-0.5 text-xs font-medium text-green-700 dark:text-green-400">
                    <CheckCircle2 className="w-3.5 h-3.5" />
                    Ready to adopt
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1.5 rounded-full bg-amber-100 dark:bg-amber-950/60 px-2.5 py-0.5 text-xs font-medium text-amber-700 dark:text-amber-400">
                    <AlertCircle className="w-3.5 h-3.5" />
                    Already adopted
                  </span>
                )}
              </div>
              {probeResult.can_adopt ? (
                <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                  <div className="space-y-0.5">
                    <p className="text-xs text-muted-foreground flex items-center gap-1">
                      <Cpu className="w-3 h-3" /> Board
                    </p>
                    <p className="text-sm font-medium">{probeResult.device_info.boarder_type}</p>
                  </div>
                  <div className="space-y-0.5">
                    <p className="text-xs text-muted-foreground flex items-center gap-1">
                      <Layers className="w-3 h-3" /> Type
                    </p>
                    <p className="text-sm font-medium">{probeResult.device_info.device_type}</p>
                  </div>
                  {probeResult.device_info.sensor_type && (
                    <div className="space-y-0.5">
                      <p className="text-xs text-muted-foreground flex items-center gap-1">
                        <Thermometer className="w-3 h-3" /> Sensor
                      </p>
                      <p className="text-sm font-medium">{probeResult.device_info.sensor_type}</p>
                    </div>
                  )}
                  <div className="space-y-0.5">
                    <p className="text-xs text-muted-foreground flex items-center gap-1">
                      <Fingerprint className="w-3 h-3" /> MAC
                    </p>
                    <p className="text-sm font-mono font-medium">{probeResult.device_info.mac_address}</p>
                  </div>
                  {probeResult.firmware_version && (
                    <div className="space-y-0.5">
                      <p className="text-xs text-muted-foreground">Firmware</p>
                      <p className="text-sm font-medium">{probeResult.firmware_version}</p>
                    </div>
                  )}
                </div>
              ) : (
                <div className="space-y-0.5">
                  <p className="text-xs text-muted-foreground flex items-center gap-1">
                    <Fingerprint className="w-3 h-3" /> MAC
                  </p>
                  <p className="text-sm font-mono font-medium">{probeResult.device_info.mac_address}</p>
                </div>
              )}
            </div>

            {/* Cards quando Already adopted e mesmo usuário */}
            {!probeResult.can_adopt && (
              <>
                {deviceFromDb === undefined ? (
                  <div className="rounded-xl border border-border bg-muted/20 p-4">
                    <p className="text-sm text-muted-foreground flex items-center gap-2">
                      <Loader2 className="w-4 h-4 animate-spin" />
                      Searching database...
                    </p>
                  </div>
                ) : deviceFromDb ? (
                  <>
                    <div className="rounded-xl border border-border bg-muted/20 p-4 space-y-3">
                      <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
                        <Server className="w-4 h-4" />
                        Registered device
                      </div>
                      <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                        <div className="space-y-0.5">
                          <p className="text-xs text-muted-foreground">Name</p>
                          <p className="text-sm font-medium">{deviceFromDb.name}</p>
                        </div>
                        <div className="space-y-0.5">
                          <p className="text-xs text-muted-foreground">Type</p>
                          <p className="text-sm font-medium">{deviceFromDb.device_type}</p>
                        </div>
                        <div className="space-y-0.5">
                          <p className="text-xs text-muted-foreground">Status</p>
                          <p className="text-sm font-medium">{deviceFromDb.operation_status ?? "offline"}</p>
                        </div>
                        <div className="space-y-0.5">
                          <p className="text-xs text-muted-foreground">Location</p>
                          <button
                            type="button"
                            onClick={() => navigate(`/locations/${deviceFromDb.location_uuid}`)}
                            className="text-sm font-medium text-primary hover:underline truncate block text-left"
                            title={deviceFromDb.location_uuid}
                          >
                            View location →
                          </button>
                        </div>
                      </div>
                      <Button
                        variant="outline"
                        onClick={() => navigate(`/devices/${deviceFromDb.uuid}/dashboard`)}
                      >
                        Open dashboard
                      </Button>
                    </div>
                    <div className="rounded-xl border border-border bg-muted/10 p-4 space-y-4">
                      <div className="text-sm font-semibold text-foreground">Device details</div>
                      <div className="grid gap-4 sm:grid-cols-2">
                        <section className="space-y-2">
                          <h4 className="text-xs font-medium uppercase text-muted-foreground">Last Will</h4>
                          <div className="space-y-1 text-sm">
                            <p><span className="text-muted-foreground">Enabled:</span> {deviceFromDb.lwt_enabled ? "Yes" : "No"}</p>
                            <p><span className="text-muted-foreground">Message:</span> {deviceFromDb.lwt_message ?? "-"}</p>
                            <p><span className="text-muted-foreground">QoS:</span> {deviceFromDb.lwt_qos}</p>
                            <p><span className="text-muted-foreground">Retain:</span> {deviceFromDb.lwt_retain ? "Yes" : "No"}</p>
                          </div>
                        </section>
                        <section className="space-y-2">
                          <h4 className="text-xs font-medium uppercase text-muted-foreground">QoS</h4>
                          <div className="space-y-1 text-sm">
                            <p><span className="text-muted-foreground">Publish:</span> {deviceFromDb.publish_qos}</p>
                            <p><span className="text-muted-foreground">Subscribe:</span> {deviceFromDb.subscribe_qos}</p>
                          </div>
                        </section>
                        <section className="space-y-2">
                          <h4 className="text-xs font-medium uppercase text-muted-foreground">Retain</h4>
                          <div className="space-y-1 text-sm">
                            <p><span className="text-muted-foreground">Status:</span> {deviceFromDb.status_retain ? "Yes" : "No"}</p>
                            <p><span className="text-muted-foreground">Data:</span> {deviceFromDb.data_retain ? "Yes" : "No"}</p>
                          </div>
                        </section>
                        <section className="space-y-2">
                          <h4 className="text-xs font-medium uppercase text-muted-foreground">Topic</h4>
                          <p className="text-sm text-muted-foreground">Base: {"{user_uuid}/{device_uuid}"}</p>
                          <p className="text-xs text-muted-foreground">See dashboard for full topic.</p>
                        </section>
                        <section className="space-y-2">
                          <h4 className="text-xs font-medium uppercase text-muted-foreground">Heartbeat</h4>
                          <div className="space-y-1 text-sm">
                            <p><span className="text-muted-foreground">Interval:</span> {deviceFromDb.heartbeat_interval}s</p>
                            <p><span className="text-muted-foreground">Offline threshold:</span> {deviceFromDb.offline_threshold}s</p>
                          </div>
                        </section>
                      </div>
                    </div>
                  </>
                ) : null}
              </>
            )}

            {/* Formulário de adoção - apenas quando can_adopt */}
            {probeResult.can_adopt && (
            <>
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
                  <label className="text-sm font-medium">Password <span className="text-muted-foreground font-normal">(leave empty for open networks)</span></label>
                  <div className="relative mt-1">
                    <input
                      type={showWifiPassword ? "text" : "password"}
                      value={wifiPassword}
                      onChange={(e) => setWifiPassword(e.target.value)}
                      placeholder="Password"
                      className="h-10 w-full rounded-lg border border-input bg-background px-3 py-2 pr-10 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                    />
                    <button
                      type="button"
                      onClick={() => setShowWifiPassword((v) => !v)}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                      tabIndex={-1}
                      aria-label={showWifiPassword ? "Hide password" : "Show password"}
                    >
                      {showWifiPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </button>
                  </div>
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

            <Button
                onClick={handleAdopt}
                disabled={adopting || !brokerInfo || !wifiSsid.trim()}
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
            </>
            )}
            </motion.div>
          )}
        </AnimatePresence>
      </div>
      </motion.div>

      {/* Divergence popup: device ready for adoption but already in DB (same user) */}
      <AlertDialog open={divergenceDialogOpen} onOpenChange={setDivergenceDialogOpen}>
        <AlertDialogContent className="max-w-md">
          <AlertDialogHeader>
            <AlertDialogTitle>Inconsistent information</AlertDialogTitle>
            <AlertDialogDescription className="text-muted-foreground">
              There are divergences in the information. The device indicates it is ready for
              adoption, but it is already registered in your account. Access the device through
              its dashboard.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel
              onClick={() => {
                setDivergenceDialogOpen(false);
                navigate("/");
              }}
            >
              Cancel
            </AlertDialogCancel>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Popup: device adopted by another user */}
      <AlertDialog open={otherUserPopupOpen} onOpenChange={setOtherUserPopupOpen}>
        <AlertDialogContent className="max-w-md">
          <AlertDialogHeader>
            <AlertDialogTitle>Device unavailable</AlertDialogTitle>
            <AlertDialogDescription className="text-muted-foreground">
              This device is already adopted by another user and cannot be adopted again.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel
              onClick={() => {
                setOtherUserPopupOpen(false);
                navigate("/");
              }}
            >
              Cancel
            </AlertDialogCancel>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

export default DeviceAdoptionWizard;
