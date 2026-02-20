import { useParams, useNavigate, Link } from "react-router-dom";
import { motion } from "framer-motion";
import { ArrowLeft, Loader2, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  useCollectorNotification,
  useMarkCollectorNotificationRead,
} from "@/hooks/useCollectorNotifications";
import { cn } from "@/lib/utils";
import { toast } from "sonner";
import type { CollectorNotificationPublic } from "@/types/collectorNotifications";

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleString(undefined, {
      dateStyle: "full",
      timeStyle: "medium",
    });
  } catch {
    return iso;
  }
}

function SeverityBadge({ severity }: { severity: CollectorNotificationPublic["severity"] }) {
  return (
    <span
      className={cn(
        "text-xs px-2 py-1 rounded font-medium",
        severity === "Critical" && "bg-destructive/20 text-destructive",
        severity === "Warn" && "bg-amber-500/20 text-amber-600 dark:text-amber-400",
        severity === "Info" && "bg-muted text-muted-foreground",
      )}
    >
      {severity}
    </span>
  );
}

const CollectorNotificationDetail = () => {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { data: notification, isLoading } = useCollectorNotification(uuid ?? null);
  const markRead = useMarkCollectorNotificationRead();

  const handleMarkRead = async () => {
    if (!uuid) return;
    try {
      await markRead.mutateAsync(uuid);
      toast.success("Marked as read");
    } catch {
      toast.error("Failed to mark as read");
    }
  };

  if (!uuid) {
    navigate("/notifications");
    return null;
  }

  if (isLoading || !notification) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
      className="space-y-6"
    >
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" asChild aria-label="Back to notifications">
          <Link to="/notifications">
            <ArrowLeft className="w-5 h-5" />
          </Link>
        </Button>
        <h1 className="text-2xl font-semibold truncate flex-1">{notification.title}</h1>
        {!notification.is_read && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleMarkRead}
            disabled={markRead.isPending}
          >
            {markRead.isPending ? (
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
            ) : (
              <Check className="w-4 h-4 mr-2" />
            )}
            Mark as read
          </Button>
        )}
      </div>

      <div className="rounded-lg border p-6 space-y-4">
        <div className="flex items-center gap-2">
          <SeverityBadge severity={notification.severity} />
          <span className="text-sm text-muted-foreground">
            {notification.notification_type} · {formatDate(notification.created_at)}
          </span>
        </div>
        <p className="text-foreground whitespace-pre-wrap">{notification.message}</p>
        {(notification.broker_uuid || notification.device_uuid) && (
          <div className="pt-4 border-t space-y-1 text-sm text-muted-foreground">
            {notification.broker_uuid && (
              <p>
                <span className="font-medium">Broker:</span> {notification.broker_uuid}
              </p>
            )}
            {notification.device_uuid && (
              <p>
                <span className="font-medium">Device:</span> {notification.device_uuid}
              </p>
            )}
          </div>
        )}
      </div>
    </motion.div>
  );
};

export default CollectorNotificationDetail;
