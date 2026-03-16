import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "react-router-dom";
import { triggerApi } from "@/services/triggerApi";
import { deviceApi } from "@/services/deviceApi";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
import { Loader2, ArrowLeft, Zap, Pencil, Trash2, Send, ChevronDown } from "lucide-react";
import { toast } from "sonner";
import type { TriggerPublic, TriggerFilter } from "@/types/trigger";
import type { DevicePublic } from "@/types/device";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
const PAGE_SIZE = 20;

export default function TriggersList() {
  const { token, logout } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<TriggerFilter>({});
  const [deleteTrigger, setDeleteTrigger] = useState<TriggerPublic | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [testingUuid, setTestingUuid] = useState<string | null>(null);

  const { data: devicesData } = useQuery({
    queryKey: ["devices"],
    queryFn: async () => {
      if (!token) return { items: [] as DevicePublic[], total: 0 };
      const r = await deviceApi.listDevices(token, { page: 1, page_size: 200 });
      return r.success && r.data ? r.data : { items: [], total: 0 };
    },
    enabled: !!token,
  });
  const devices = devicesData?.items ?? [];

  const { data, isLoading, error } = useQuery({
    queryKey: ["triggers-list", filter],
    queryFn: async () => {
      if (!token) {
        logout();
        return null;
      }
      const result = await triggerApi.listTriggers(token, {
        page: 1,
        page_size: PAGE_SIZE,
        filter,
      });
      if (!result.success) {
        if (result.unauthorized) {
          toast.error("Session expired. Please login again.");
          logout();
          return null;
        }
        throw new Error(result.message ?? "Failed to load triggers");
      }
      return result.data ?? { items: [], total: 0, page: 1, page_size: PAGE_SIZE };
    },
    enabled: !!token,
  });

  const triggers = data?.items ?? [];

  const handleDelete = async () => {
    if (!token || !deleteTrigger) return;
    setIsDeleting(true);
    const result = await triggerApi.deleteTrigger(token, deleteTrigger.uuid);
    setIsDeleting(false);
    setDeleteTrigger(null);
    if (result.unauthorized) {
      logout();
      return;
    }
    if (result.success) {
      toast.success("Trigger deleted.");
      queryClient.invalidateQueries({ queryKey: ["triggers-list"] });
    } else {
      toast.error(result.message ?? "Failed to delete trigger");
    }
  };

  const handleTest = async (t: TriggerPublic) => {
    if (!token) return;
    if (t.action_type !== "discord" && t.action_type !== "telegram") {
      toast.info("Test is only available for Discord and Telegram triggers.");
      return;
    }
    setTestingUuid(t.uuid);
    const result = await triggerApi.sendTest(token, t.uuid);
    setTestingUuid(null);
    if (result.unauthorized) {
      logout();
      return;
    }
    if (result.success) {
      toast.success("Test notification sent.");
    } else {
      toast.error(result.message ?? "Failed to send test");
    }
  };

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] text-muted-foreground gap-2">
        <p className="font-semibold text-destructive">
          {error instanceof Error ? error.message : "Failed to load triggers"}
        </p>
        <Button onClick={() => navigate("/")} variant="outline" size="sm">
          Back to Home
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={() => navigate("/")} aria-label="Back">
            <ArrowLeft className="w-5 h-5" />
          </Button>
          <div>
            <h1 className="text-2xl font-semibold">Triggers</h1>
            <p className="text-muted-foreground text-sm">
              Automate notifications and device commands.
            </p>
          </div>
        </div>
        <Button asChild>
          <Link to="/triggers/create">Create trigger</Link>
        </Button>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <span className="text-sm text-muted-foreground">Device:</span>
        <div className="relative min-w-[160px]">
          <select
            className="h-10 w-full appearance-none rounded-lg border border-input bg-card pr-9 pl-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            value={filter.device_uuid ?? "all"}
            onChange={(e) =>
              setFilter((f) => ({
                ...f,
                device_uuid: e.target.value === "all" ? undefined : e.target.value,
              }))
            }
          >
            <option value="all">All devices</option>
            {devices.map((d) => (
              <option key={d.uuid} value={d.uuid}>
                {d.name}
              </option>
            ))}
          </select>
          <ChevronDown className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        </div>
        <span className="text-sm text-muted-foreground ml-2">Status:</span>
        <div className="relative min-w-[120px]">
          <select
            className="h-10 w-full appearance-none rounded-lg border border-input bg-card pr-9 pl-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            value={
              filter.is_active === undefined ? "all" : filter.is_active ? "active" : "inactive"
            }
            onChange={(e) =>
              setFilter((f) => ({
                ...f,
                is_active:
                  e.target.value === "all" ? undefined : e.target.value === "active",
              }))
            }
          >
            <option value="all">All</option>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
          </select>
          <ChevronDown className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        </div>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      ) : triggers.length === 0 ? (
        <div className="bg-background border border-border rounded-xl p-12 text-center">
          <Zap className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
          <p className="text-muted-foreground mb-4">No triggers found.</p>
          <Button asChild variant="outline" size="sm">
            <Link to="/triggers/create">Create your first trigger</Link>
          </Button>
        </div>
      ) : (
        <div className="rounded-xl border border-border overflow-hidden bg-background">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-muted/50 border-b border-border">
                <tr>
                  <th className="text-left p-3 font-medium">Name</th>
                  <th className="text-left p-3 font-medium">Event</th>
                  <th className="text-left p-3 font-medium">Action</th>
                  <th className="text-left p-3 font-medium">Device</th>
                  <th className="text-left p-3 font-medium">Status</th>
                  <th className="text-right p-3 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {triggers.map((t) => (
                  <tr key={t.uuid} className="border-t border-border">
                    <td className="p-3 font-medium">{t.name}</td>
                    <td className="p-3 text-muted-foreground">{t.source_event}</td>
                    <td className="p-3 text-muted-foreground">{t.action_type}</td>
                    <td className="p-3 text-muted-foreground">
                      {t.device_uuid
                        ? devices.find((d) => d.uuid === t.device_uuid)?.name ?? t.device_uuid
                        : "—"}
                    </td>
                    <td className="p-3">
                      <span
                        className={
                          t.is_active
                            ? "text-green-600 dark:text-green-400"
                            : "text-muted-foreground"
                        }
                      >
                        {t.is_active ? "Active" : "Inactive"}
                      </span>
                    </td>
                    <td className="p-3 text-right">
                      <div className="flex items-center justify-end gap-1">
                        {(t.action_type === "discord" || t.action_type === "telegram") && (
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => handleTest(t)}
                            disabled={testingUuid !== null}
                            title="Send test notification"
                          >
                            {testingUuid === t.uuid ? (
                              <Loader2 className="w-4 h-4 animate-spin" />
                            ) : (
                              <Send className="w-4 h-4" />
                            )}
                          </Button>
                        )}
                        <Button variant="ghost" size="icon" asChild title="Edit">
                          <Link to={`/triggers/${t.uuid}/edit`}>
                            <Pencil className="w-4 h-4" />
                          </Link>
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => setDeleteTrigger(t)}
                          title="Delete"
                        >
                          <Trash2 className="w-4 h-4 text-destructive" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      <AlertDialog open={!!deleteTrigger} onOpenChange={(open) => !open && setDeleteTrigger(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete trigger</AlertDialogTitle>
            <AlertDialogDescription>
              Delete &quot;{deleteTrigger?.name}&quot;? This cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              disabled={isDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isDeleting ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Deleting...
                </>
              ) : (
                "Delete"
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
