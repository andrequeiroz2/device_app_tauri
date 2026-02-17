import { useState, useMemo, ChangeEvent, useEffect } from "react";
import { toast } from "sonner";
import { Upload, ImageIcon, FileWarning, Loader2 } from "lucide-react";
import { locationApi } from "@/services/locationApi";
import { useAuth } from "@/context/AuthContext";
import type { LocationCreateInput } from "@/types/location";
import { Button } from "@/components/ui/button";

const MAX_IMAGE_BYTES = 5 * 1024 * 1024;
const ACCEPTED_TYPES = ["image/png", "image/jpeg", "image/webp"];

type FileState = {
  file: File | null;
  error: string | null;
};

const Locations = () => {
  const { token, logout } = useAuth();
  const [form, setForm] = useState<LocationCreateInput>({
    name: "",
    address: "",
    description: "",
  });
  const [fileState, setFileState] = useState<FileState>({ file: null, error: null });
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!token) {
      logout();
    }
  }, [token, logout]);

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
      setFileState(validateFile(f));
    }
  };

  const validateFile = (f: File): FileState => {
    if (!ACCEPTED_TYPES.includes(f.type)) {
      return { file: null, error: "Formato inválido. Use PNG, JPG ou WEBP." };
    }
    if (f.size > MAX_IMAGE_BYTES) {
      return { file: null, error: "Arquivo maior que 5 MB." };
    }
    return { file: f, error: null };
  };

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token) {
      logout();
      return;
    }
    if (!isValid) {
      toast.error("Preencha os campos obrigatórios e verifique o arquivo.");
      return;
    }

    setSubmitting(true);
    const payload: LocationCreateInput = {
      name: form.name.trim(),
      address: form.address.trim(),
      description: form.description?.trim() || undefined,
    };

    const result = await locationApi.createLocation(token, payload, fileState.file);
    setSubmitting(false);

    if (!result.success) {
      if (result.unauthorized) {
        toast.error("Sessão expirada. Faça login novamente.");
        logout();
        return;
      }
      toast.error(result.message ?? "Erro ao criar localização.");
      return;
    }

    toast.success("Local criado com sucesso.");
    setForm({ name: "", address: "", description: "" });
    setFileState({ file: null, error: null });
  };

  return (
    <div className="min-h-screen bg-secondary/20 text-foreground">
      <div className="max-w-4xl mx-auto py-10 px-4 space-y-6">
        <div>
          <h1 className="text-2xl font-semibold">Location</h1>
          <p className="text-muted-foreground text-sm">
            Create a new location with address and floor plan (PNG/JPG/WEBP up to 5 MB).
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

          <div className="flex justify-end">
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

export default Locations;

