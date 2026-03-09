import { useQuery } from "@tanstack/react-query";
import { Icon } from "@iconify/react";
import { iconApi } from "@/services/iconApi";
import { useAuth } from "@/context/AuthContext";
import type { IconPublic } from "@/types/icon";
import { cn } from "@/lib/utils";
import { Loader2 } from "lucide-react";

type DeviceIconSelectorProps = {
  /** Tipo do device (vem do probe: sensor ou actuator) — exibe apenas ícones desta category */
  deviceType: "sensor" | "actuator";
  /** UUID do ícone selecionado (icon.uuid), null se nenhum */
  value: string | null;
  onChange: (uuid: string) => void;
};

/**
 * Seletor de ícone para device. Mostra apenas ícones cuja category
 * corresponde ao deviceType (sensor → ícones sensor, actuator → ícones actuator).
 */
export const DeviceIconSelector = ({
  deviceType,
  value,
  onChange,
}: DeviceIconSelectorProps) => {
  const { token, logout } = useAuth();

  const { data, isLoading } = useQuery({
    queryKey: ["icons-for-device", deviceType],
    queryFn: async () => {
      if (!token) {
        logout();
        return { items: [] };
      }
      const resp = await iconApi.listIcons(token, 1, 50, { category: deviceType });
      if (!resp.success) {
        if (resp.unauthorized) logout();
        return { items: [] };
      }
      return resp.data ?? { items: [], total: 0, page: 1, page_size: 50 };
    },
    retry: false,
  });

  const items: IconPublic[] = data?.items ?? [];

  if (isLoading) {
    return (
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Loader2 className="w-4 h-4 animate-spin" />
        Loading icons…
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        No {deviceType} icons available. Add icons in Settings → Icons.
      </p>
    );
  }

  return (
    <div className="grid grid-cols-4 sm:grid-cols-6 gap-2">
      {items.map((icon) => {
        const isSelected = value === icon.uuid;
        return (
          <button
            key={icon.uuid}
            type="button"
            onClick={() => onChange(icon.uuid)}
            className={cn(
              "flex flex-col items-center justify-center gap-1 p-2 rounded-lg border-2 transition-colors min-h-[64px]",
              isSelected
                ? "border-primary bg-primary/10"
                : "border-border hover:bg-muted/50 hover:border-muted-foreground/30"
            )}
            title={icon.name}
          >
            <div
              className="w-8 h-8 flex items-center justify-center rounded shrink-0"
              style={{
                backgroundColor: icon.color ? `${icon.color}20` : "var(--muted)",
              }}
            >
              <Icon
                icon={icon.iconify_id}
                className="w-5 h-5"
                style={{ color: icon.color ?? undefined }}
              />
            </div>
            <span className="text-xs truncate w-full text-center">
              {icon.name}
            </span>
          </button>
        );
      })}
    </div>
  );
};
