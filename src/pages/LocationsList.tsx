import { useMemo, useState, useEffect } from "react";
import { useInfiniteQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { convertFileSrc } from "@tauri-apps/api/core";
import { locationApi } from "@/services/locationApi";
import { useAuth } from "@/context/AuthContext";
import type { LocationPublic, LocationListResponse, LocationFilter } from "@/types/location";
import { Button } from "@/components/ui/button";
import { Loader2, ImageOff } from "lucide-react";
import { toast } from "sonner";
import { LocationFilter as LocationFilterPanel } from "@/components/LocationFilter";
import { storage } from "@/lib/storage";
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

const PAGE_SIZE = 8;

const LocationsList = () => {
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<LocationFilter>({ status: "active" });
  const [inactiveModalOpen, setInactiveModalOpen] = useState(false);
  const [selectedInactiveLocation, setSelectedInactiveLocation] = useState<LocationPublic | null>(null);
  const [isActivating, setIsActivating] = useState(false);

  // Load filter from localStorage on mount
  useEffect(() => {
    const savedFilter = storage.getLocationFilter();
    setFilter(savedFilter);
  }, []);

  const {
    data,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery({
    queryKey: ["locations-list", filter],
    initialPageParam: 1,
    queryFn: async ({ pageParam }) => {
      if (!token) {
        logout();
        return null;
      }
      const resp = await locationApi.listLocations(token, pageParam, PAGE_SIZE, filter);
      if (!resp.success) {
        if (resp.unauthorized) {
          logout();
        } else {
          toast.error(resp.message ?? "Failed to load locations.");
        }
        throw new Error(resp.message ?? "Failed to load locations.");
      }
      return resp.data as LocationListResponse;
    },
    getNextPageParam: (lastPage) => {
      if (!lastPage) return undefined;
      const { page, page_size, total } = lastPage;
      const loaded = page * page_size;
      return loaded < total ? page + 1 : undefined;
    },
    retry: false,
  });

  const items: LocationPublic[] = useMemo(() => {
    if (!data?.pages) return [];
    return data.pages.flatMap((p) => p?.items ?? []);
  }, [data]);

  const handleCardClick = (location: LocationPublic, e: React.MouseEvent) => {
    if (!location.is_active) {
      e.preventDefault();
      e.stopPropagation();
      setSelectedInactiveLocation(location);
      setInactiveModalOpen(true);
    }
  };

  const handleActivate = async () => {
    if (!token || !selectedInactiveLocation) return;
    setIsActivating(true);

    const result = await locationApi.updateLocation(token, {
      uuid: selectedInactiveLocation.uuid,
      is_active: true,
    });

    setIsActivating(false);
    setInactiveModalOpen(false);
    setSelectedInactiveLocation(null);

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
  };

  return (
    <div className="min-h-[60vh]">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h1 className="text-2xl font-semibold">Locations</h1>
          <p className="text-muted-foreground text-sm">List of your locations.</p>
        </div>
        <LocationFilterPanel value={filter} onChange={setFilter} />
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-16 text-muted-foreground">
          <Loader2 className="w-5 h-5 animate-spin mr-2" />
          Loading...
        </div>
      ) : (
        <>
          {items.length === 0 ? (
            <div className="flex items-center justify-center py-16 text-muted-foreground">
              No locations found.
            </div>
          ) : (
            <div className="max-h-[70vh] overflow-y-auto pr-1">
              <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
                {items.map((loc) => (
                  <LocationCard key={loc.uuid} location={loc} onClick={handleCardClick} />
                ))}
              </div>
            </div>
          )}

          {hasNextPage && (
            <div className="flex justify-center mt-6">
              <Button onClick={() => fetchNextPage()} disabled={isFetchingNextPage}>
                {isFetchingNextPage && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
                Load more
              </Button>
            </div>
          )}
        </>
      )}

      <AlertDialog open={inactiveModalOpen} onOpenChange={setInactiveModalOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Location Inactive</AlertDialogTitle>
            <AlertDialogDescription>
              The location "{selectedInactiveLocation?.name}" is currently inactive. Only activation is allowed for inactive locations.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isActivating}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleActivate}
              disabled={isActivating}
            >
              {isActivating ? (
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
    </div>
  );
};

type LocationCardProps = {
  location: LocationPublic;
  onClick: (location: LocationPublic, e: React.MouseEvent) => void;
};

const LocationCard = ({ location, onClick }: LocationCardProps) => {
  // Add cache buster using updated_at timestamp to force reload after image update
  const cacheBuster = location.updated_at ? `?t=${new Date(location.updated_at).getTime()}` : '';
  const thumbSrc = location.thumb_path
    ? `${convertFileSrc(location.thumb_path)}${cacheBuster}`
    : location.image_path
    ? `${convertFileSrc(location.image_path)}${cacheBuster}`
    : null;
  const fallback =
    "data:image/svg+xml;utf8," +
    encodeURIComponent(
      `<svg xmlns='http://www.w3.org/2000/svg' width='200' height='120' viewBox='0 0 200 120'><rect width='200' height='120' fill='%23f1f5f9'/><text x='50%' y='50%' dominant-baseline='middle' text-anchor='middle' fill='%2394a3b8' font-family='Arial' font-size='14'>Image not available</text></svg>`
    );

  const isInactive = !location.is_active;

  return (
    <Link
      to={`/locations/${location.uuid}`}
      onClick={(e: React.MouseEvent<HTMLAnchorElement>) => onClick(location, e)}
      className={cn(
        "block border border-border rounded-xl bg-card shadow-sm overflow-hidden hover:shadow-md transition cursor-pointer",
        isInactive && "opacity-60 grayscale"
      )}
    >
      <div className="aspect-video bg-secondary/40 flex items-center justify-center overflow-hidden">
        {thumbSrc ? (
          <img
            src={thumbSrc}
            alt={location.name}
            className="w-full h-full object-cover"
            loading="lazy"
            onError={(e) => {
              e.currentTarget.src = fallback;
            }}
          />
        ) : (
          <div className="flex items-center gap-2 text-muted-foreground text-sm">
            <ImageOff className="w-4 h-4" />
            No image
          </div>
        )}
      </div>
      <div className="p-3 space-y-1">
        <div className="font-semibold text-sm">{location.name}</div>
        <div className="text-xs text-muted-foreground line-clamp-2">{location.address}</div>
        {isInactive && (
          <div className="text-xs text-muted-foreground/70 italic">Inactive</div>
        )}
      </div>
    </Link>
  );
};

export default LocationsList;

