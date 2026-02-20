import { useState } from "react";
import { PanelRight, X, Edit, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Link } from "react-router-dom";
import { ConnectDisconnectBrokerButton } from "@/components/ConnectDisconnectBrokerButton";

type MqttBrokerActionsPanelProps = {
  brokerUuid: string;
  isActive: boolean;
  isConnected: boolean;
  onConnect: (brokerUuid: string) => Promise<{ success: boolean }>;
  onDisconnect: () => Promise<{ success: boolean }>;
  onDelete: () => void;
};

export const MqttBrokerActionsPanel = ({
  brokerUuid,
  isActive,
  isConnected,
  onConnect,
  onDisconnect,
  onDelete,
}: MqttBrokerActionsPanelProps) => {
  const [isPanelOpen, setIsPanelOpen] = useState(false);

  return (
    <>
      {/* Panel Button */}
      <Button
        variant="outline"
        size="sm"
        onClick={() => setIsPanelOpen(true)}
        className="flex items-center gap-2"
      >
        <PanelRight className="w-4 h-4" />
        Panel
      </Button>

      {/* Sidebar Overlay */}
      {isPanelOpen && (
        <div
          className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm"
          onClick={() => setIsPanelOpen(false)}
        />
      )}

      {/* Sidebar Panel */}
      <div
        className={cn(
          "fixed top-0 right-0 z-[60] h-full w-80 bg-background border-l border-border shadow-lg transition-transform duration-300 ease-in-out",
          isPanelOpen ? "translate-x-0" : "translate-x-full"
        )}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex flex-col h-full">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-border">
            <div className="flex items-center gap-2">
              <PanelRight className="w-5 h-5" />
              <h2 className="text-lg font-semibold">Actions</h2>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setIsPanelOpen(false)}
              className="h-8 w-8"
            >
              <X className="w-4 h-4" />
            </Button>
          </div>

          {/* Content */}
          <div className="flex-1 p-4 space-y-4">
            <div className="space-y-3">
              {isActive && (
                <ConnectDisconnectBrokerButton
                  brokerUuid={brokerUuid}
                  isActive={isActive}
                  isConnected={isConnected}
                  onConnect={onConnect}
                  onDisconnect={onDisconnect}
                  variant="default"
                  size="default"
                  className="w-full justify-start"
                />
              )}
              {isActive && (
                <Button
                  asChild
                  variant="outline"
                  className="w-full justify-start gap-2"
                  onClick={() => setIsPanelOpen(false)}
                >
                  <Link to={`/mqtt-brokers/${brokerUuid}/edit`}>
                    <Edit className="w-4 h-4" />
                    Edit Broker
                  </Link>
                </Button>
              )}

              <Button
                variant="destructive"
                className="w-full justify-start gap-2"
                onClick={() => {
                  setIsPanelOpen(false);
                  onDelete();
                }}
              >
                <Trash2 className="w-4 h-4" />
                Delete Broker
              </Button>
            </div>
          </div>
        </div>
      </div>
    </>
  );
};

