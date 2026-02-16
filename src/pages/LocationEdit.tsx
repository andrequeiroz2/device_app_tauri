import { useState, useMemo, ChangeEvent, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { Upload, ImageIcon, FileWarning, Loader2 } from "lucide-react";
import { locationApi } from "@/services/locationApi";
import { useAuth } from "@/context/AuthContext";
import type { LocationUpdateInput } from "@/types/location";
import { Button } from "@/components/ui/button";
import { convertFileSrc } from "@tauri-apps/api/core";

const MAX_IMAGE_BYTES = 5 * 1024 * 1024;
const ACCEPTED_TYPES = ["image/png", "image/jpeg", "image/webp"];

type FileState = {
  file: File | null;
  error: string | null;
};

const LocationEdit = () => {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const { token, logout } = useAuth();
  const queryClient = useQueryClient();
  const [form, setForm] = useState<{
    name: string;
    address: string;
    description: string;
  }>({
    name: "",
    address: "",
    description: "",
  });
  const [fileState, setFileState] = useState<FileState>({ file: null, error: null });
  const [newImagePreview, setNewImagePreview] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const { data: location, isLoading } = useQuery({
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
        throw new Error(result.message ?? "Failed to load location.");
      }
      return result.data ?? null;
    },
    enabled: !!uuid && !!token,
  });

  useEffect(() => {
    if (location) {
      setForm({
        name: location.name || "",
        address: location.address || "",
        description: location.description || "",
      });
    }
  }, [location]);

  useEffect(() => {
    if (!token) {
      logout();
    }
  }, [token, logout]);

  // Cleanup preview URL on unmount
  useEffect(() => {
    return () => {
      if (newImagePreview) {
        URL.revokeObjectURL(newImagePreview);
      }
    };
  }, [newImagePreview]);

  const isValid = useMemo(() => {
    const nameOk = form.name.trim().length > 0;
    const addressOk = form.address.trim().length > 0;
    const fileOk =
      !fileState.file ||
      (fileState.file.size <= MAX_IMAGE_BYTES && ACCEPTED_TYPES.includes(fileState.file.type));
    return nameOk && addressOk && fileOk;
  }, [form.name, form.address, fileState.file]);

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    const f = e.target.files?.[0];
    if (f) {
      const validated = validateFile(f);
      setFileState(validated);
      
      // Create preview URL for new image
      if (validated.file) {
        const previewUrl = URL.createObjectURL(validated.file);
        setNewImagePreview(previewUrl);
      } else {
        // Clear preview if file is invalid
        if (newImagePreview) {
          URL.revokeObjectURL(newImagePreview);
          setNewImagePreview(null);
        }
      }
    } else {
      // Clear preview if no file selected
      if (newImagePreview) {
        URL.revokeObjectURL(newImagePreview);
        setNewImagePreview(null);
      }
    }
  };

  const validateFile = (f: File): FileState => {
    if (!ACCEPTED_TYPES.includes(f.type)) {
      return { file: null, error: "Invalid format. Use PNG, JPG or WEBP." };
    }
    if (f.size > MAX_IMAGE_BYTES) {
      return { file: null, error: "File larger than 5 MB." };
    }
    return { file: f, error: null };
  };

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !uuid) {
      logout();
      return;
    }
    if (!isValid) {
      toast.error("Fill required fields and verify the file.");
      return;
    }

    setSubmitting(true);
    const payload: LocationUpdateInput = {
      uuid,
      name: form.name.trim(),
      address: form.address.trim(),
      description: form.description?.trim() || undefined,
    };

    const result = await locationApi.updateLocation(token, payload, fileState.file);
    setSubmitting(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Session expired. Please login again.");
        logout();
        return;
      }
      toast.error(result.message ?? "Failed to update location.");
      return;
    }

    toast.success("Location updated successfully.");
    queryClient.invalidateQueries({ queryKey: ["locations-list"] });
    queryClient.invalidateQueries({ queryKey: ["location", uuid] });
    navigate(`/locations/${uuid}`);
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

  if (!location.is_active) {
    return (
      <div className="flex items-center justify-center min-h-[60vh] text-muted-foreground">
        Location is inactive. Only activation is allowed.
      </div>
    );
  }

  // Add cache buster using updated_at timestamp to force reload after image update
  const cacheBuster = location.updated_at ? `?t=${new Date(location.updated_at).getTime()}` : '';
  const currentImageSrc = location.image_path
    ? `${convertFileSrc(location.image_path)}${cacheBuster}`
    : location.thumb_path
    ? `${convertFileSrc(location.thumb_path)}${cacheBuster}`
    : null;

  return (
    <div className="min-h-screen bg-secondary/20 text-foreground">
      <div className="max-w-4xl mx-auto py-10 px-4 space-y-6">
        <div>
          <h1 className="text-2xl font-semibold">Edit Location</h1>
          <p className="text-muted-foreground text-sm">
            Update location information and floor plan (PNG/JPG/WEBP up to 5 MB).
          </p>
        </div>

        <form
          onSubmit={onSubmit}
          className="bg-background border border-border rounded-xl p-6 shadow-sm space-y-4"
        >
          <div className="space-y-2">
            <label className="text-sm font-medium">Name</label>
            <input
              value={form.name}
              onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
              className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              placeholder="E.g.: My Home"
              required
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">Address</label>
            <input
              value={form.address}
              onChange={(e) => setForm((prev) => ({ ...prev, address: e.target.value }))}
              className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              placeholder="Street Example, 123"
              required
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">Description (optional)</label>
            <textarea
              value={form.description}
              onChange={(e) => setForm((prev) => ({ ...prev, description: e.target.value }))}
              className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              placeholder="Additional notes"
              rows={3}
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">Floor plan image</label>
            {(currentImageSrc || newImagePreview) && (
              <div className="mb-3 border border-border rounded-lg p-3 bg-secondary/40">
                <p className="text-xs text-muted-foreground mb-3">Images:</p>
                <div className="flex items-center gap-4">
                  {currentImageSrc && (
                    <div className="flex flex-col items-center gap-1">
                      <p className="text-xs text-muted-foreground mb-1">Current</p>
                      <img
                        src={currentImageSrc}
                        alt="Current location image"
                        className="max-h-32 w-auto rounded border border-border"
                      />
                    </div>
                  )}
                  {newImagePreview && (
                    <div className="flex flex-col items-center gap-1">
                      <p className="text-xs text-muted-foreground mb-1">New</p>
                      <img
                        src={newImagePreview}
                        alt="New location image preview"
                        className="max-h-32 w-auto rounded border-2 border-primary"
                      />
                    </div>
                  )}
                </div>
              </div>
            )}
            <div
              className="border border-dashed border-muted-foreground/40 rounded-lg p-4 bg-secondary/40 flex flex-col gap-3 items-center justify-center text-center cursor-pointer"
            >
              <input
                id="file-input"
                type="file"
                accept={ACCEPTED_TYPES.join(",")}
                className="hidden"
                onChange={handleFileChange}
              />
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Upload className="w-4 h-4" />
                <label htmlFor="file-input" className="text-primary cursor-pointer font-medium">
                  Select an image file
                </label>
              </div>
              <div className="text-xs text-muted-foreground">
                PNG, JPG or WEBP — up to 5 MB
              </div>
              {fileState.file && (
                <div className="flex items-center gap-2 text-sm text-foreground">
                  <ImageIcon className="w-4 h-4" />
                  <span>{fileState.file.name} ({(fileState.file.size / 1024 / 1024).toFixed(2)} MB)</span>
                </div>
              )}
              {fileState.error && (
                <div className="flex items-center gap-1 text-sm text-destructive">
                  <FileWarning className="w-4 h-4" />
                  <span>{fileState.error}</span>
                </div>
              )}
            </div>
          </div>

          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => navigate(`/locations/${uuid}`)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={!isValid || submitting}>
              {submitting && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
              Save
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default LocationEdit;

