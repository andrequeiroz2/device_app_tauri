import { useState, useMemo, useEffect, useCallback } from "react";
import { toast } from "sonner";
import { Icon } from "@iconify/react";
import { Loader2, ArrowLeft, HelpCircle } from "lucide-react";
import { iconApi } from "@/services/iconApi";
import { useAuth } from "@/context/AuthContext";
import { useNavigate } from "react-router-dom";
import type { IconCreateInput, IconCategory } from "@/types/icon";
import { Button } from "@/components/ui/button";
import { ColorPalette } from "@/components/ColorPalette";
import { cn } from "@/lib/utils";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

const IconCreate = () => {
  const { token, logout } = useAuth();
  const navigate = useNavigate();
  const [form, setForm] = useState<IconCreateInput>({
    name: "",
    iconify_id: "",
    category: "sensor",
    color: null,
  });
  const [submitting, setSubmitting] = useState(false);
  const [iconifyHelpOpen, setIconifyHelpOpen] = useState(false);
  const [iconPreviewStatus, setIconPreviewStatus] = useState<"idle" | "loading" | "valid" | "invalid">("idle");

  const checkIconExists = useCallback(async (iconifyId: string): Promise<boolean> => {
    try {
      const id = iconifyId.trim();
      const [prefix, iconName] = id.split(":");
      if (!prefix || !iconName) return false;
      const res = await fetch(
        `https://api.iconify.design/${encodeURIComponent(prefix)}.json?icons=${encodeURIComponent(iconName)}`
      );
      if (!res.ok) return false;
      const data = await res.json();
      return !!(data?.icons?.[iconName]);
    } catch {
      return false;
    }
  }, []);

  useEffect(() => {
    if (!token) {
      logout();
    }
  }, [token, logout]);

  useEffect(() => {
    const id = form.iconify_id.trim();
    if (!id || !id.includes(":")) {
      setIconPreviewStatus("idle");
      return;
    }
    setIconPreviewStatus("loading");
    let active = true;
    const t = setTimeout(async () => {
      const valid = await checkIconExists(id);
      if (active) {
        setIconPreviewStatus(valid ? "valid" : "invalid");
      }
    }, 400);
    return () => {
      active = false;
      clearTimeout(t);
    };
  }, [form.iconify_id, checkIconExists]);

  const isValid = useMemo(() => {
    const nameOk = form.name.trim().length > 0;
    const iconifyOk =
      form.iconify_id.trim().length > 0 && form.iconify_id.includes(":");
    return nameOk && iconifyOk;
  }, [form.name, form.iconify_id]);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token) {
      logout();
      return;
    }
    if (!isValid) {
      toast.error("Please fill in the required fields (name and iconify_id in format prefix:icon-name).");
      return;
    }

    setSubmitting(true);
    const result = await iconApi.createIcon(token, {
      ...form,
      name: form.name.trim(),
      iconify_id: form.iconify_id.trim(),
      color: form.color ?? undefined,
    });
    setSubmitting(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to create icon.");
      return;
    }

    toast.success("Icon created successfully.");
    navigate("/icons");
  };

  return (
    <div className="min-h-screen bg-secondary/20 text-foreground">
      <div className="max-w-4xl mx-auto py-10 px-4 space-y-6">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate("/icons")}
            aria-label="Back"
          >
            <ArrowLeft className="w-5 h-5" />
          </Button>
          <div>
            <h1 className="text-2xl font-semibold">Icon</h1>
            <p className="text-muted-foreground text-sm">
              Create a new icon.
            </p>
          </div>
        </div>

        <form
          onSubmit={onSubmit}
          className="bg-background border border-border rounded-xl p-6 shadow-sm space-y-6"
        >
          <div className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">Name *</label>
            <input
              value={form.name}
              onChange={(e) =>
                setForm((prev) => ({ ...prev, name: e.target.value }))
              }
              className="w-full h-10 rounded-lg border border-input bg-transparent px-3 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              placeholder="E.g.: Temperature"
              required
            />
          </div>

          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <label className="text-sm font-medium">Iconify ID *</label>
              <button
                type="button"
                onClick={() => setIconifyHelpOpen(true)}
                className="inline-flex text-muted-foreground hover:text-foreground transition-colors"
                aria-label="How to fill Iconify ID"
              >
                <HelpCircle className="w-4 h-4" />
              </button>
            </div>
            <input
              value={form.iconify_id}
              onChange={(e) =>
                setForm((prev) => ({ ...prev, iconify_id: e.target.value }))
              }
              className="w-full h-10 rounded-lg border border-input bg-transparent px-3 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              placeholder="E.g.: mdi:thermometer or lucide:droplets"
              required
            />
            {iconPreviewStatus !== "idle" && (
              <div
                className={cn(
                  "flex items-center gap-3 rounded-lg border p-3",
                  iconPreviewStatus === "valid" && "border-green-500/30 bg-green-500/5",
                  iconPreviewStatus === "invalid" && "border-destructive/30 bg-destructive/5",
                  iconPreviewStatus === "loading" && "border-border bg-muted/30"
                )}
              >
                {iconPreviewStatus === "loading" && (
                  <>
                    <Loader2 className="w-6 h-6 animate-spin text-muted-foreground shrink-0" />
                    <span className="text-sm text-muted-foreground">Checking icon…</span>
                  </>
                )}
                {iconPreviewStatus === "valid" && (
                  <>
                    <div
                      className="w-10 h-10 flex items-center justify-center rounded-lg shrink-0"
                      style={{
                        backgroundColor: form.color ? `${form.color}20` : "var(--muted)",
                      }}
                    >
                      <Icon
                        icon={form.iconify_id.trim()}
                        className="w-6 h-6"
                        style={{ color: form.color ?? undefined }}
                      />
                    </div>
                    <span className="text-sm text-green-700 dark:text-green-400">Valid icon</span>
                  </>
                )}
                {iconPreviewStatus === "invalid" && (
                  <span className="text-sm text-destructive">Icon not found in Iconify</span>
                )}
              </div>
            )}
          </div>

          <AlertDialog open={iconifyHelpOpen} onOpenChange={setIconifyHelpOpen}>
            <AlertDialogContent className="max-w-md">
              <AlertDialogHeader>
                <AlertDialogTitle>Iconify ID</AlertDialogTitle>
                <AlertDialogDescription asChild>
                  <div className="space-y-3 pt-2">
                    <p>
                      Enter the icon identifier from Iconify in the format <strong>prefix:icon-name</strong>.
                    </p>
                    <p className="text-sm">
                      <strong>Examples:</strong>
                    </p>
                    <ul className="space-y-2 text-sm text-muted-foreground">
                      <li><span className="text-foreground font-medium">Temperature</span><br /><code className="text-xs">mdi:thermometer</code></li>
                      <li><span className="text-foreground font-medium">Humidity</span><br /><code className="text-xs">lucide:droplets</code></li>
                      <li><span className="text-foreground font-medium">Light</span><br /><code className="text-xs">lucide:lightbulb</code></li>
                      <li><span className="text-foreground font-medium">Fan</span><br /><code className="text-xs">ph:fan-fill</code></li>
                      <li><span className="text-foreground font-medium">Door</span><br /><code className="text-xs">mdi:door-open</code></li>
                    </ul>
                    <p className="text-sm">
                      Browse icons at{" "}
                      <a
                        href="https://icon-sets.iconify.design/"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-primary underline hover:no-underline"
                      >
                        icon-sets.iconify.design
                      </a>
                    </p>
                  </div>
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <Button onClick={() => setIconifyHelpOpen(false)}>OK</Button>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>

          <div className="space-y-2">
            <label className="text-sm font-medium">Category *</label>
            <div className="flex gap-2">
              {(["sensor", "actuator"] as IconCategory[]).map((c) => (
                <button
                  key={c}
                  type="button"
                  onClick={() =>
                    setForm((prev) => ({ ...prev, category: c }))
                  }
                  className={cn(
                    "px-4 py-2 rounded-lg border text-sm font-medium transition-colors",
                    form.category === c
                      ? "border-primary bg-primary/10 text-primary"
                      : "border-border hover:bg-accent hover:text-accent-foreground"
                  )}
                >
                  {c}
                </button>
              ))}
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">Color</label>
            <ColorPalette
              value={form.color}
              onChange={(hex) =>
                setForm((prev) => ({ ...prev, color: hex }))
              }
            />
          </div>
        </div>

          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => navigate("/icons")}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              variant="outline"
              size="sm"
              disabled={!isValid || submitting || iconPreviewStatus !== "valid"}
            >
              {submitting && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
              Create icon
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default IconCreate;
