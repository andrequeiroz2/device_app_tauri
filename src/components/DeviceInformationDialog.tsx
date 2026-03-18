import { useEffect, useState } from "react";
import type { DevicePublic } from "@/types/device";
import { provisioningApi } from "@/services/provisioningApi";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

type BrokerInfo = {
  host: string;
  port: number;
  broker_url: string;
};

export function DeviceInformationDialog({
  open,
  device,
  onOpenChange,
}: {
  open: boolean;
  device: DevicePublic | null;
  onOpenChange: (open: boolean) => void;
}) {
  const { token, logout } = useAuth();
  const [brokerInfo, setBrokerInfo] = useState<BrokerInfo | null>(null);
  const [brokerLoading, setBrokerLoading] = useState(false);

  useEffect(() => {
    if (!open || !token) return;
    if (!device) return;

    setBrokerLoading(true);
    provisioningApi
      .getDefaultBroker(token)
      .then((r) => {
        setBrokerLoading(false);
        if (r.unauthorized) {
          logout();
          return;
        }
        if (r.success && r.data) {
          setBrokerInfo({
            host: r.data.host,
            port: r.data.port,
            broker_url: r.data.broker_url,
          });
        } else {
          setBrokerInfo(null);
        }
      })
      .catch(() => {
        setBrokerLoading(false);
        setBrokerInfo(null);
      });
  }, [open, token, device, logout]);

  return (
    <AlertDialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) {
          setBrokerInfo(null);
          setBrokerLoading(false);
        }
        onOpenChange(nextOpen);
      }}
    >
      <AlertDialogContent className="max-w-md">
        <AlertDialogHeader>
          <AlertDialogTitle>Device Information</AlertDialogTitle>
          <AlertDialogDescription asChild>
            {device && (
              <div className="space-y-3 pt-2 text-left">
                <div>
                  <p className="font-semibold text-foreground mb-0.5">Name</p>
                  <p className="text-sm text-muted-foreground">{device.name}</p>
                </div>
                {device.description && (
                  <div>
                    <p className="font-semibold text-foreground mb-0.5">
                      Description
                    </p>
                    <p className="text-sm text-muted-foreground">
                      {device.description}
                    </p>
                  </div>
                )}
                <div>
                  <p className="font-semibold text-foreground mb-0.5">Type</p>
                  <p className="text-sm text-muted-foreground capitalize">
                    {device.device_type}
                  </p>
                </div>
                <div>
                  <p className="font-semibold text-foreground mb-0.5">Model</p>
                  <p className="text-sm text-muted-foreground">{device.model}</p>
                </div>
                <div>
                  <p className="font-semibold text-foreground mb-0.5">
                    MAC Address
                  </p>
                  <p className="text-sm text-muted-foreground font-mono">
                    {device.mac_address}
                  </p>
                </div>
                {device.operation_status && (
                  <div>
                    <p className="font-semibold text-foreground mb-0.5">Status</p>
                    <p className="text-sm text-muted-foreground capitalize">
                      {device.operation_status}
                    </p>
                  </div>
                )}
                {device.sensor_type && (
                  <div>
                    <p className="font-semibold text-foreground mb-0.5">
                      Sensor Type
                    </p>
                    <p className="text-sm text-muted-foreground">
                      {device.sensor_type}
                    </p>
                  </div>
                )}
                {device.actuator_type && (
                  <div>
                    <p className="font-semibold text-foreground mb-0.5">
                      Actuator Type
                    </p>
                    <p className="text-sm text-muted-foreground">
                      {device.actuator_type}
                    </p>
                  </div>
                )}

                <div>
                  <p className="font-semibold text-foreground mb-0.5">Broker</p>
                  {brokerLoading ? (
                    <p className="text-sm text-muted-foreground">Loading...</p>
                  ) : brokerInfo ? (
                    <p className="text-sm text-muted-foreground">
                      {brokerInfo.broker_url}
                    </p>
                  ) : (
                    <p className="text-sm text-muted-foreground">
                      No default broker configured
                    </p>
                  )}
                </div>

                <div>
                  <p className="font-semibold text-foreground mb-0.5">
                    Topic Base
                  </p>
                  <p className="text-sm text-muted-foreground">
                    <span className="font-mono">
                      {device.user_uuid}/{device.uuid}
                    </span>
                  </p>
                </div>

                {device.device_type === "sensor" &&
                  device.parameter_ranges &&
                  Object.keys(device.parameter_ranges).length > 0 && (
                    <div>
                      <p className="font-semibold text-foreground mb-0.5">
                        Reading ranges
                      </p>
                      <ul className="text-sm text-muted-foreground list-none space-y-1">
                        {Object.entries(device.parameter_ranges).map(
                          ([measurement, range]) => (
                            <li key={measurement} className="font-mono">
                              {measurement}: {range.min_reading}–{range.max_reading}{" "}
                              {range.unit}
                            </li>
                          )
                        )}
                      </ul>
                    </div>
                  )}
                {device.device_type === "actuator" &&
                  device.command_spec && (
                    <div>
                      <p className="font-semibold text-foreground mb-0.5">
                        Command spec
                      </p>
                      <p className="text-sm text-muted-foreground">
                        {device.command_spec.type === "discrete"
                          ? `Commands: ${device.command_spec.commands.join(", ")}`
                          : `Range: ${device.command_spec.min}–${device.command_spec.max} ${device.command_spec.unit}`}
                      </p>
                    </div>
                  )}
              </div>
            )}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <Button onClick={() => onOpenChange(false)}>Close</Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

