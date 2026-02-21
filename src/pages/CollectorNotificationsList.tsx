import { useState, useMemo } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { Loader2, Check, CheckCheck, Bell } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CollectorNotificationFilter as CollectorNotificationFilterPanel } from "@/components/CollectorNotificationFilter";
import {
  useCollectorNotificationsList,
  useCollectorNotificationsCount,
  useMarkCollectorNotificationRead,
  useMarkAllCollectorNotificationsRead,
} from "@/hooks/useCollectorNotifications";
import { cn } from "@/lib/utils";
import { toast } from "sonner";
import type { CollectorNotificationPublic, CollectorNotificationFilter } from "@/types/collectorNotifications";

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleString(undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    });
  } catch {
    return iso;
  }
}

function SeverityBadge({ severity }: { severity: CollectorNotificationPublic["severity"] }) {
  return (
    <span
      className={cn(
        "text-xs px-2 py-0.5 rounded font-medium",
        severity === "Critical" && "bg-destructive/20 text-destructive",
        severity === "Warn" && "bg-amber-500/20 text-amber-600 dark:text-amber-400",
        severity === "Info" && "bg-muted text-muted-foreground",
      )}
    >
      {severity}
    </span>
  );
}

const DEFAULT_FILTER: CollectorNotificationFilter = {
  is_read: "no_read",
  severity: "All",
};

const CollectorNotificationsList = () => {
  const [filter, setFilter] = useState<CollectorNotificationFilter>(DEFAULT_FILTER);
  const {
    data,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useCollectorNotificationsList(filter);
  const { data: unreadCount = 0 } = useCollectorNotificationsCount();
  const markRead = useMarkCollectorNotificationRead();
  const markAllRead = useMarkAllCollectorNotificationsRead();

  const items = useMemo(() => {
    if (!data?.pages) return [];
    return data.pages.flatMap((p) => p.items);
  }, [data]);

  const hasUnread = unreadCount > 0;

  const handleMarkRead = async (uuid: string, e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    try {
      await markRead.mutateAsync(uuid);
      toast.success("Marked as read");
    } catch {
      toast.error("Failed to mark as read");
    }
  };

  const handleMarkAllRead = async () => {
    try {
      await markAllRead.mutateAsync();
      toast.success("All marked as read");
    } catch {
      toast.error("Failed to mark all as read");
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
      className="flex flex-col h-[calc(100vh-120px)]"
    >
      <div className="flex items-center justify-between shrink-0 pb-4">
        <div>
          <h1 className="text-2xl font-semibold">Notifications</h1>
          <p className="text-muted-foreground text-sm">List of your notifications.</p>
        </div>
        <div className="flex items-center gap-2">
          {hasUnread && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleMarkAllRead}
              disabled={markAllRead.isPending}
            >
              {markAllRead.isPending ? (
                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              ) : (
                <CheckCheck className="w-4 h-4 mr-2" />
              )}
              Mark all as read
            </Button>
          )}
          <CollectorNotificationFilterPanel value={filter} onChange={setFilter} />
        </div>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center flex-1">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      ) : items.length === 0 ? (
        <div className="bg-background border border-border rounded-xl p-12 text-center flex-1 flex flex-col items-center justify-center">
          <Bell className="w-12 h-12 mb-4 text-muted-foreground" />
          <p className="text-muted-foreground">No notifications.</p>
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto border border-border rounded-xl">
          <div className="space-y-2 p-4">
            {items.map((n) => (
              <Link
                key={n.uuid}
                to={`/notifications/${n.uuid}`}
                className={cn(
                  "block rounded-lg border p-4 transition-colors hover:bg-muted/50",
                  !n.is_read && "bg-muted/30 border-l-4 border-l-primary",
                )}
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className={cn("font-medium", !n.is_read && "font-semibold")}>
                        {n.title}
                      </span>
                      <SeverityBadge severity={n.severity} />
                    </div>
                    <p className="mt-0.5 text-xs text-muted-foreground">
                      {n.is_read ? "Already read" : "New"}
                    </p>
                    <p className="mt-1 text-sm text-muted-foreground line-clamp-2">{n.message}</p>
                    <p className="mt-1 text-xs text-muted-foreground">{formatDate(n.created_at)}</p>
                  </div>
                  {!n.is_read && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => handleMarkRead(n.uuid, e)}
                      disabled={markRead.isPending}
                      aria-label="Mark as read"
                    >
                      {markRead.isPending ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <Check className="w-4 h-4" />
                      )}
                    </Button>
                  )}
                </div>
              </Link>
            ))}

            {hasNextPage && (
              <div className="flex justify-center pt-4">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => fetchNextPage()}
                  disabled={isFetchingNextPage}
                >
                  {isFetchingNextPage ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      Loading...
                    </>
                  ) : (
                    "Load more"
                  )}
                </Button>
              </div>
            )}
          </div>
        </div>
      )}
    </motion.div>
  );
};

export default CollectorNotificationsList;
