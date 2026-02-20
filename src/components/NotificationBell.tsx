import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { Bell } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import { useAuth } from "@/context/AuthContext";
import {
  useCollectorNotificationsPreview,
  useCollectorNotificationsCount,
  COLLECTOR_NOTIFICATIONS_KEYS,
} from "@/hooks/useCollectorNotifications";
import { useQueryClient } from "@tanstack/react-query";
import { cn } from "@/lib/utils";
import type { CollectorNotificationPublic } from "@/types/collectorNotifications";

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, {
      day: "2-digit",
      month: "short",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return iso;
  }
}

function SeverityBadge({ severity }: { severity: CollectorNotificationPublic["severity"] }) {
  return (
    <span
      className={cn(
        "text-xs px-1.5 py-0.5 rounded",
        severity === "Critical" && "bg-destructive/20 text-destructive",
        severity === "Warn" && "bg-amber-500/20 text-amber-600 dark:text-amber-400",
        severity === "Info" && "bg-muted text-muted-foreground",
      )}
    >
      {severity}
    </span>
  );
}

const COLLECTOR_NOTIFICATION_ADDED_EVENT = "collector-notification-added";

export function NotificationBell() {
  const { token } = useAuth();
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const { data: preview = [], isLoading: listLoading, refetch: refetchPreview } = useCollectorNotificationsPreview();
  const { data: count = 0, isLoading: countLoading } = useCollectorNotificationsCount();

  const handleOpenChange = (nextOpen: boolean) => {
    setOpen(nextOpen);
    if (nextOpen) {
      refetchPreview();
    }
  };

  // Real-time: invalidate queries when backend persists a new notification
  useEffect(() => {
    if (!token) return;
    const unlisten = listen(COLLECTOR_NOTIFICATION_ADDED_EVENT, () => {
      queryClient.invalidateQueries({ queryKey: COLLECTOR_NOTIFICATIONS_KEYS.all });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [token, queryClient]);

  if (!token) return null;

  const hasUnread = count > 0;

  return (
    <DropdownMenu open={open} onOpenChange={handleOpenChange}>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" aria-label="Notifications" className="relative">
          <Bell className="w-5 h-5" />
          {!countLoading && hasUnread && (
            <span className="absolute -top-1 -right-1 min-w-[18px] h-[18px] flex items-center justify-center rounded-full bg-destructive text-destructive-foreground text-xs font-medium px-1">
              {count > 99 ? "99+" : count}
            </span>
          )}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="z-50 w-80 max-h-[400px] overflow-y-auto">
        <div className="px-2 py-2 text-sm font-medium text-muted-foreground">
          Notifications
        </div>
        <DropdownMenuSeparator />
        {listLoading ? (
          <div className="px-4 py-6 text-center text-muted-foreground text-sm">
            Loading...
          </div>
        ) : preview.length === 0 ? (
          <div className="px-4 py-6 text-center text-muted-foreground text-sm">
            No notifications
          </div>
        ) : (
          preview.map((n) => (
            <DropdownMenuItem key={n.uuid} asChild>
              <Link
                to={`/notifications/${n.uuid}`}
                className="flex flex-col items-start gap-1 py-2 cursor-pointer hover:bg-muted/70 focus:bg-muted/70"
                onClick={() => setOpen(false)}
              >
                <div className="flex items-center justify-between w-full gap-2">
                  <span className={cn("font-medium truncate flex-1", !n.is_read && "font-semibold")}>
                    {n.title}
                  </span>
                  <SeverityBadge severity={n.severity} />
                </div>
                <span className="text-xs text-muted-foreground line-clamp-2">{n.message}</span>
                <span className="text-xs text-muted-foreground">{formatDate(n.created_at)}</span>
              </Link>
            </DropdownMenuItem>
          ))
        )}
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild>
          <Link
            to="/notifications"
            className="w-full justify-center"
            onClick={() => setOpen(false)}
          >
            List
          </Link>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
