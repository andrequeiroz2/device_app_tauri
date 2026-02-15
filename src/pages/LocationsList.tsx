import { useMemo } from "react";
import { useInfiniteQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { convertFileSrc } from "@tauri-apps/api/core";
import { locationApi } from "@/services/locationApi";
import { useAuth } from "@/context/AuthContext";
import type { LocationPublic, LocationListResponse } from "@/types/location";
import { Button } from "@/components/ui/button";
import { Loader2, ImageOff } from "lucide-react";
import { toast } from "sonner";

const PAGE_SIZE = 8;

const LocationsList = () => {
  const { token, logout } = useAuth();

  const {
    data,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery({
    queryKey: ["locations-list"],
    initialPageParam: 1,
    queryFn: async ({ pageParam }) => {
      if (!token) {
        logout();
        return null;
      }
      const resp = await locationApi.listLocations(token, pageParam, PAGE_SIZE);
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

  return (
    <div className="min-h-[60vh]">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h1 className="text-2xl font-semibold">Locations</h1>
          <p className="text-muted-foreground text-sm">List of your locations.</p>
        </div>
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
                  <LocationCard key={loc.uuid} location={loc} />
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
    </div>
  );
};

type LocationCardProps = {
  location: LocationPublic;
};

const LocationCard = ({ location }: LocationCardProps) => {
  const thumbSrc = location.thumb_path
    ? convertFileSrc(location.thumb_path)
    : location.image_path
    ? convertFileSrc(location.image_path)
    : null;
  const fallback =
    "data:image/svg+xml;utf8," +
    encodeURIComponent(
      `<svg xmlns='http://www.w3.org/2000/svg' width='200' height='120' viewBox='0 0 200 120'><rect width='200' height='120' fill='%23f1f5f9'/><text x='50%' y='50%' dominant-baseline='middle' text-anchor='middle' fill='%2394a3b8' font-family='Arial' font-size='14'>Image not available</text></svg>`
    );

  return (
    <Link
      to={`/locations/${location.uuid}`}
      className="block border border-border rounded-xl bg-card shadow-sm overflow-hidden hover:shadow-md transition cursor-pointer"
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
      </div>
    </Link>
  );
};

export default LocationsList;

