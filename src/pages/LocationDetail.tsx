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
import { Loader2, ImageOff, AlertCircle, Plus } from "lucide-react";
import { toast } from "sonner";
import { LocationActionsPanel } from "@/components/LocationActionsPanel";

const LocationDetail = () => {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [isActivating, setIsActivating] = useState(false);

  const { data: location, isLoading, error: queryError } = useQuery({
    queryKey: ["location", uuid],
    queryFn: async () => {
      if (!token || !uuid) {
        logout();
        return null;
      }
      const result = await locationApi.getLocation(token, uuid);
      if (!result.success) {
        if (result.unauthorized) {
          toast.error("Session expired. Please login again.");
          logout();
          return null;
        }
        const errorMsg = result.message ?? "Failed to load location.";
        console.error("getLocation error:", errorMsg);
        throw new Error(errorMsg);
      }
      if (!result.data) {
        console.error("getLocation: no data returned");
        return null;
      }
      return result.data;
    },
    enabled: !!uuid && !!token,
    retry: false,
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

  const handleActivate = async () => {
    if (!token || !uuid) return;
    setIsActivating(true);

    const result = await locationApi.updateLocation(token, {
      uuid,
      is_active: true,
    });

    setIsActivating(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to activate location.");
      return;
    }

    toast.success("Location activated successfully.");
    queryClient.invalidateQueries({ queryKey: ["locations-list"] });
    queryClient.invalidateQueries({ queryKey: ["location", uuid] });
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (queryError) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] text-muted-foreground gap-2">
        <p className="font-semibold">Error loading location</p>
        <p className="text-sm">{queryError.message}</p>
        <Button onClick={() => navigate("/locations/list")} variant="outline">
          Back to List
        </Button>
      </div>
    );
  }

  if (!location) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] text-muted-foreground gap-2">
        <p>Location not found.</p>
        <Button onClick={() => navigate("/locations/list")} variant="outline">
          Back to List
        </Button>
      </div>
    );
  }

  // Add cache buster using updated_at timestamp to force reload after image update
  const cacheBuster = location.updated_at ? `?t=${new Date(location.updated_at).getTime()}` : '';
  const imageSrc = location.image_path
    ? `${convertFileSrc(location.image_path)}${cacheBuster}`
    : location.thumb_path
    ? `${convertFileSrc(location.thumb_path)}${cacheBuster}`
    : null;

  const fallback =
    "data:image/svg+xml;utf8," +
    encodeURIComponent(
      `<svg xmlns='http://www.w3.org/2000/svg' width='800' height='600' viewBox='0 0 800 600'><rect width='800' height='600' fill='%23f1f5f9'/><text x='50%' y='50%' dominant-baseline='middle' text-anchor='middle' fill='%2394a3b8' font-family='Arial' font-size='24'>No image saved</text></svg>`
    );

  const isInactive = !location.is_active;

  return (
    <>
      <div className="space-y-4">
        {isInactive && (
          <div className="border border-yellow-500/50 bg-yellow-500/10 rounded-lg p-4 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <AlertCircle className="w-5 h-5 text-yellow-600 dark:text-yellow-500" />
              <div>
                <p className="font-semibold text-yellow-900 dark:text-yellow-100">
                  Location Inactive
                </p>
                <p className="text-sm text-yellow-700 dark:text-yellow-300">
                  This location is currently inactive. Only activation is allowed.
                </p>
              </div>
            </div>
            <Button
              onClick={handleActivate}
              disabled={isActivating}
              className="bg-yellow-600 hover:bg-yellow-700 text-white"
            >
              {isActivating ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Activating...
                </>
              ) : (
                "Activate"
              )}
            </Button>
          </div>
        )}

        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-semibold">{location.name}</h1>
          </div>
          <div className="flex items-center gap-2">
            {location.is_active && (
              <Button
                variant="default"
                size="sm"
                onClick={() => navigate(`/locations/${location.uuid}/devices/adopt`)}
                className="gap-2"
              >
                <Plus className="w-4 h-4" />
                Adopt Device
              </Button>
            )}
            <LocationActionsPanel
            locationUuid={location.uuid}
            isActive={location.is_active}
            name={location.name}
            address={location.address}
            description={location.description}
            onDelete={() => setDeleteDialogOpen(true)}
            />
          </div>
        </div>

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

