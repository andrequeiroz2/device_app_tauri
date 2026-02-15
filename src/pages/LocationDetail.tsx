import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { convertFileSrc } from "@tauri-apps/api/core";
import { locationApi } from "@/services/locationApi";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
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
import { Loader2, Trash2, ImageOff } from "lucide-react";
import { toast } from "sonner";

const LocationDetail = () => {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const { data: location, isLoading } = useQuery({
    queryKey: ["location", uuid],
    queryFn: async () => {
      if (!token || !uuid) {
        logout();
        return null;
      }
      // Por enquanto, buscar da lista em cache ou fazer uma query separada
      // TODO: implementar get_location_by_uuid no backend
      const listData = queryClient.getQueryData<{ pages: Array<{ items: Array<any> }> }>(["locations-list"]);
      if (listData?.pages) {
        const allItems = listData.pages.flatMap((p) => p?.items ?? []);
        return allItems.find((loc) => loc.uuid === uuid) || null;
      }
      return null;
    },
    enabled: !!uuid && !!token,
  });

  const handleDelete = async () => {
    if (!token || !uuid) return;
    setIsDeleting(true);

    const result = await locationApi.deleteLocation(token, uuid);
    setIsDeleting(false);
    setDeleteDialogOpen(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to delete location.");
      return;
    }

    toast.success("Location deleted successfully.");
    queryClient.invalidateQueries({ queryKey: ["locations-list"] });
    navigate("/locations/list");
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!location) {
    return (
      <div className="flex items-center justify-center min-h-[60vh] text-muted-foreground">
        Location not found.
      </div>
    );
  }

  const imageSrc = location.image_path
    ? convertFileSrc(location.image_path)
    : location.thumb_path
    ? convertFileSrc(location.thumb_path)
    : null;

  const fallback =
    "data:image/svg+xml;utf8," +
    encodeURIComponent(
      `<svg xmlns='http://www.w3.org/2000/svg' width='800' height='600' viewBox='0 0 800 600'><rect width='800' height='600' fill='%23f1f5f9'/><text x='50%' y='50%' dominant-baseline='middle' text-anchor='middle' fill='%2394a3b8' font-family='Arial' font-size='24'>No image saved</text></svg>`
    );

  return (
    <>
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-semibold">{location.name}</h1>
            <p className="text-muted-foreground text-sm">{location.address}</p>
          </div>
          <Button
            variant="destructive"
            onClick={() => setDeleteDialogOpen(true)}
            className="flex items-center gap-2"
          >
            <Trash2 className="w-4 h-4" />
            Delete
          </Button>
        </div>

      {location.description && (
        <div className="text-sm text-muted-foreground">{location.description}</div>
      )}

      <div className="border border-border rounded-xl bg-card overflow-hidden">
        <div className="w-full bg-secondary/40 flex items-center justify-center min-h-[400px] max-h-[70vh] overflow-auto">
          {imageSrc ? (
            <img
              src={imageSrc}
              alt={location.name}
              className="w-full h-auto object-contain"
              onError={(e) => {
                e.currentTarget.src = fallback;
              }}
            />
          ) : (
            <div className="flex flex-col items-center gap-2 text-muted-foreground py-16">
              <ImageOff className="w-12 h-12" />
              <span>No image saved</span>
            </div>
          )}
        </div>
      </div>

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Location</AlertDialogTitle>
            <AlertDialogDescription>
              Do you really want to delete the location "{location.name}"?
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
                "Continue"
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
    </>
  );
};

export default LocationDetail;

