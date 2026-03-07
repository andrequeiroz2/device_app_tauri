import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Loader2, Wifi, WifiOff } from "lucide-react";
import { cn } from "@/lib/utils";

type ConnectDisconnectBrokerButtonProps = {
  brokerUuid: string;
  isActive: boolean;
  isConnected: boolean;
  onConnect: (brokerUuid: string) => Promise<{ success: boolean }>;
  onDisconnect: () => Promise<{ success: boolean }>;
  variant?: "default" | "outline" | "ghost";
  size?: "default" | "sm" | "lg" | "icon";
  className?: string;
};

export function ConnectDisconnectBrokerButton({
  brokerUuid,
  isActive,
  isConnected,
  onConnect,
  onDisconnect,
  variant = "outline",
  size = "sm",
  className,
}: ConnectDisconnectBrokerButtonProps) {
  const [isLoading, setIsLoading] = useState(false);

  const handleClick = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!isActive || isLoading) return;

    setIsLoading(true);
    try {
      if (isConnected) {
        await onDisconnect();
      } else {
        await onConnect(brokerUuid);
      }
    } finally {
      setIsLoading(false);
    }
  };

  const disabled = !isActive || isLoading;

  return (
    <Button
      variant={variant}
      size={size}
      onClick={handleClick}
      disabled={disabled}
      className={cn("gap-2 w-[8.5rem] justify-center", className)}
    >
      {isLoading ? (
        <Loader2 className="w-4 h-4 animate-spin" />
      ) : isConnected ? (
        <WifiOff className="w-4 h-4" />
      ) : (
        <Wifi className="w-4 h-4" />
      )}
      {isLoading ? "..." : isConnected ? "Disconnect" : "Connect"}
    </Button>
  );
}
