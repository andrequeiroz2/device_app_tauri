import { useState, useMemo, useEffect } from "react";
import { useInfiniteQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Icon } from "@iconify/react";
import { iconApi } from "@/services/iconApi";
import { useAuth } from "@/context/AuthContext";
import type { IconPublic, IconCategory } from "@/types/icon";
import { Button } from "@/components/ui/button";
import { IconFilterPanel } from "@/components/IconFilterPanel";
import { storage } from "@/lib/storage";
import { Loader2, ArrowLeft, Pencil, Trash2, RotateCcw, CircleSlash } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
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

type CategoryFilter = IconCategory | "all";

const IconsList = () => {
  const { token, logout } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [category, setCategory] = useState<CategoryFilter>("all");
  const [filter, setFilter] = useState<{ status?: "active" | "all" }>({ status: "active" });
  const [deleteTarget, setDeleteTarget] = useState<IconPublic | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [inactiveModalOpen, setInactiveModalOpen] = useState(false);
  const [selectedInactiveIcon, setSelectedInactiveIcon] = useState<IconPublic | null>(null);
  const [isReactivating, setIsReactivating] = useState(false);

  useEffect(() => {
    const savedFilter = storage.getIconFilter();
    setFilter(savedFilter);
  }, []);

  const params = useMemo(
    () => ({
      ...(category === "all" ? {} : { category }),
      status: filter.status ?? "active",
    }),
    [category, filter.status]
  );

  const PAGE_SIZE = 10;

  const {
    data,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery({
    queryKey: ["icons-list", params],
    initialPageParam: 1,
    queryFn: async ({ pageParam }) => {
      if (!token) {
        logout();
        return null;
      }
      const resp = await iconApi.listIcons(token, pageParam, PAGE_SIZE, params);
      if (!resp.success) {
        if (resp.unauthorized) {
          logout();
        } else {
          toast.error(resp.message ?? "Failed to load icons.");
        }
        throw new Error(resp.message ?? "Failed to load icons.");
      }
      return resp.data ?? { items: [], total: 0, page: 1, page_size: PAGE_SIZE };
    },
    getNextPageParam: (lastPage) => {
      if (!lastPage) return undefined;
      const { page, page_size, total } = lastPage;
      const loaded = page * page_size;
      return loaded < total ? page + 1 : undefined;
    },
    retry: false,
  });

  const items: IconPublic[] = useMemo(() => {
    if (!data?.pages) return [];
    return data.pages.flatMap((p) => p?.items ?? []);
  }, [data]);

  const handleReactivate = async () => {
    if (!token || !selectedInactiveIcon) return;
    setIsReactivating(true);
    const result = await iconApi.updateIcon(token, {
      uuid: selectedInactiveIcon.uuid,
      is_active: true,
    });
    setIsReactivating(false);
    setInactiveModalOpen(false);
    setSelectedInactiveIcon(null);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to activate icon.");
      return;
    }

    toast.success("Icon activated successfully.");
    queryClient.invalidateQueries({ queryKey: ["icons-list"] });
  };

  const handleDelete = async () => {
    if (!token || !deleteTarget) return;
    setDeleting(true);
    const result = await iconApi.deleteIcon(token, deleteTarget.uuid);
    setDeleting(false);
    setDeleteTarget(null);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to delete icon.");
      return;
    }

    toast.success("Icon deleted successfully.");
    queryClient.invalidateQueries({ queryKey: ["icons-list"] });
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
      className="flex flex-col h-[calc(100vh-120px)]"
    >
      <div className="flex items-center justify-between shrink-0 pb-4">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate("/")}
            aria-label="Back"
          >
            <ArrowLeft className="w-5 h-5" />
          </Button>
          <div>
            <h1 className="text-2xl font-semibold">Icons</h1>
            <p className="text-muted-foreground text-sm">
              Manage icons.
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex rounded-lg border border-border overflow-hidden">
            {(["all", "sensor", "actuator"] as const).map((c) => (
              <button
                key={c}
                type="button"
                onClick={() => setCategory(c)}
                className={cn(
                  "px-3 py-1.5 text-sm font-medium transition-colors",
                  category === c
                    ? "bg-primary text-primary-foreground"
                    : "bg-background hover:bg-accent hover:text-accent-foreground"
                )}
              >
                {c === "all" ? "All" : c}
              </button>
            ))}
          </div>
          <IconFilterPanel value={filter} onChange={setFilter} />
        </div>
      </div>

      {isLoading ? (
        <div className="flex flex-1 items-center justify-center">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      ) : items.length === 0 ? (
        <div className="flex-1 flex flex-col items-center justify-center border border-border rounded-xl bg-muted/20 p-12">
          <p className="text-muted-foreground mb-4">No icons found.</p>
          <Button asChild variant="outline" size="sm">
            <Link to="/icons/create">Add your first icon</Link>
          </Button>
        </div>
      ) : (
        <div className="flex-1 min-h-0 overflow-y-auto border border-border rounded-xl">
          <div className="p-4 space-y-2">
            {items.map((icon) =>
              icon.is_active ? (
                <Link
                  key={icon.uuid}
                  to={`/icons/${icon.uuid}/edit`}
                  className="block rounded-lg border border-border p-4 transition-colors hover:bg-muted/50"
                >
                  <div className="flex items-center gap-4">
                    <div
                      className="w-10 h-10 flex items-center justify-center rounded-lg shrink-0"
                      style={{
                        backgroundColor: icon.color
                          ? `${icon.color}20`
                          : "var(--muted)",
                      }}
                    >
                      <Icon
                        icon={icon.iconify_id}
                        className="w-6 h-6"
                        style={{ color: icon.color ?? undefined }}
                      />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="font-medium">{icon.name}</p>
                      <p className="text-sm text-muted-foreground font-mono">
                        {icon.code} · {icon.iconify_id}
                      </p>
                      <span
                        className={cn(
                          "inline-block mt-1 text-xs px-2 py-0.5 rounded font-medium",
                          icon.category === "sensor"
                            ? "bg-blue-500/20 text-blue-700 dark:text-blue-300"
                            : "bg-amber-500/20 text-amber-700 dark:text-amber-300"
                        )}
                      >
                        {icon.category}
                      </span>
                    </div>
                    <div
                      className="flex items-center gap-2 shrink-0"
                      onClick={(e) => e.preventDefault()}
                    >
                      <Button asChild variant="outline" size="sm">
                        <Link to={`/icons/${icon.uuid}/edit`} onClick={(e) => e.stopPropagation()}>
                          <Pencil className="w-4 h-4" />
                        </Link>
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={(e) => {
                          e.preventDefault();
                          setDeleteTarget(icon);
                        }}
                        className="text-destructive hover:text-destructive"
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                </Link>
              ) : (
                <div
                  key={icon.uuid}
                  className={cn(
                    "flex items-center gap-4 p-4 rounded-lg border",
                    "bg-muted/30 border-l-4 border-l-amber-500/50"
                  )}
                >
                  <div
                    className="w-10 h-10 flex items-center justify-center rounded-lg shrink-0 opacity-60"
                    style={{
                      backgroundColor: icon.color
                        ? `${icon.color}20`
                        : "var(--muted)",
                    }}
                  >
                    <Icon
                      icon={icon.iconify_id}
                      className="w-6 h-6"
                      style={{ color: icon.color ?? undefined }}
                    />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="font-medium">{icon.name}</p>
                      <span
                        className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded font-medium bg-amber-500/20 text-amber-700 dark:text-amber-300 border border-amber-500/30"
                        aria-label="Inactive"
                      >
                        <CircleSlash className="w-3.5 h-3.5" />
                        Inactive
                      </span>
                    </div>
                    <p className="text-sm text-muted-foreground font-mono">
                      {icon.code} · {icon.iconify_id}
                    </p>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setSelectedInactiveIcon(icon);
                      setInactiveModalOpen(true);
                    }}
                  >
                    <RotateCcw className="w-4 h-4 mr-1.5" />
                    Activate
                  </Button>
                </div>
              )
            )}

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

      <AlertDialog
        open={inactiveModalOpen}
        onOpenChange={(open) => {
          setInactiveModalOpen(open);
          if (!open) setSelectedInactiveIcon(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Icon Inactive</AlertDialogTitle>
            <AlertDialogDescription>
              The icon &quot;{selectedInactiveIcon?.name}&quot; is currently inactive. You can activate it to make it available again for devices.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel
              disabled={isReactivating}
              onClick={() => {
                setInactiveModalOpen(false);
                setSelectedInactiveIcon(null);
              }}
            >
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={handleReactivate}
              disabled={isReactivating}
            >
              {isReactivating ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Activating...
                </>
              ) : (
                "Activate"
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={!!deleteTarget}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Icon</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete &quot;{deleteTarget?.name}&quot;?
              This will deactivate the icon.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              disabled={deleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {deleting ? (
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
    </motion.div>
  );
};

export default IconsList;
