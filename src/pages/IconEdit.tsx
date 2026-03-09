import { useState, useMemo, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { Loader2, ArrowLeft, ImageIcon } from "lucide-react";
import { iconApi } from "@/services/iconApi";
import { useAuth } from "@/context/AuthContext";
import type { IconUpdateInput, IconCategory } from "@/types/icon";
import { Button } from "@/components/ui/button";
import { ColorPalette } from "@/components/ColorPalette";
import { cn } from "@/lib/utils";

const IconEdit = () => {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();
  const [form, setForm] = useState<{
    name: string;
    iconify_id: string;
    category: IconCategory;
    color: string | null;
  }>({
    name: "",
    iconify_id: "",
    category: "sensor",
    color: null,
  });
  const [submitting, setSubmitting] = useState(false);

  const { data: icon, isLoading } = useQuery({
    queryKey: ["icon", uuid],
    queryFn: async () => {
      if (!token || !uuid) {
        logout();
        return null;
      }
      const result = await iconApi.getIcon(token, uuid);
      if (!result.success) {
        if (result.unauthorized) {
          logout();
        } else {
          toast.error(result.message ?? "Failed to load icon.");
        }
        throw new Error(result.message ?? "Failed to load icon.");
      }
      return result.data ?? null;
    },
    enabled: !!uuid && !!token,
  });

  useEffect(() => {
    if (icon) {
      setForm({
        name: icon.name,
        iconify_id: icon.iconify_id,
        category: icon.category as IconCategory,
        color: icon.color ?? null,
      });
    }
  }, [icon]);

  useEffect(() => {
    if (!token) {
      logout();
    }
  }, [token, logout]);

  const isValid = useMemo(() => {
    const nameOk = form.name.trim().length > 0;
    const iconifyOk =
      form.iconify_id.trim().length > 0 && form.iconify_id.includes(":");
    return nameOk && iconifyOk;
  }, [form.name, form.iconify_id]);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !uuid) {
      logout();
      return;
    }
    if (!isValid) {
      toast.error("Please fill in the required fields (name and iconify_id in format prefix:icon-name).");
      return;
    }

    setSubmitting(true);
    const payload: IconUpdateInput = {
      uuid,
      name: form.name.trim(),
      iconify_id: form.iconify_id.trim(),
      category: form.category,
      color: form.color,
    };
    const result = await iconApi.updateIcon(token, payload);
    setSubmitting(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to update icon.");
      return;
    }

    toast.success("Icon updated successfully.");
    queryClient.invalidateQueries({ queryKey: ["icons-list"] });
    queryClient.invalidateQueries({ queryKey: ["icon", uuid] });
    navigate("/icons");
  };

  if (isLoading || !icon) {
    return (
      <div className="flex items-center justify-center min-h-[40vh]">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

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
            <h1 className="text-2xl font-semibold">Icons</h1>
            <p className="text-muted-foreground text-sm">
              Edit icon &quot;{icon.name}&quot;.
            </p>
          </div>
        </div>

        <form
          onSubmit={onSubmit}
          className="bg-background border border-border rounded-xl p-6 shadow-sm space-y-6 max-w-xl"
        >
          <div className="space-y-4">
          <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
            <ImageIcon className="w-4 h-4" />
            Icon details
          </div>

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
            <label className="text-sm font-medium">Iconify ID *</label>
            <input
              value={form.iconify_id}
              onChange={(e) =>
                setForm((prev) => ({ ...prev, iconify_id: e.target.value }))
              }
              className="w-full h-10 rounded-lg border border-input bg-transparent px-3 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              placeholder="E.g.: mdi:thermometer or lucide:droplets"
              required
            />
            <p className="text-xs text-muted-foreground">
              Format: prefix:icon-name (mdi, lucide, phosphor, etc.)
            </p>
          </div>

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
              disabled={!isValid || submitting}
            >
              {submitting && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
              Save changes
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default IconEdit;
