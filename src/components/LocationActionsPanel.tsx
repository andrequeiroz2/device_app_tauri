import { useState } from "react";
import { PanelRight, X, Edit, Trash2, Info } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Link } from "react-router-dom";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

type LocationActionsPanelProps = {
  locationUuid: string;
  isActive: boolean;
  name: string;
  address: string;
  description?: string | null;
  onDelete: () => void;
};

export const LocationActionsPanel = ({
  locationUuid,
  isActive,
  name,
  address,
  description,
  onDelete,
}: LocationActionsPanelProps) => {
  const [isPanelOpen, setIsPanelOpen] = useState(false);
  const [infoModalOpen, setInfoModalOpen] = useState(false);

  return (
    <>
      {/* Panel Button */}
      <Button
        variant="outline"
        size="sm"
        onClick={() => setIsPanelOpen(true)}
        className="flex items-center gap-2"
      >
        <PanelRight className="w-4 h-4" />
        Panel
      </Button>

      {/* Sidebar Overlay */}
      {isPanelOpen && (
        <div
          className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm"
          onClick={() => setIsPanelOpen(false)}
        />
      )}

      {/* Sidebar Panel */}
      <div
        className={cn(
          "fixed top-0 right-0 z-[60] h-full w-80 bg-background border-l border-border shadow-lg transition-transform duration-300 ease-in-out",
          isPanelOpen ? "translate-x-0" : "translate-x-full"
        )}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex flex-col h-full">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-border">
            <div className="flex items-center gap-2">
              <PanelRight className="w-5 h-5" />
              <h2 className="text-lg font-semibold">Actions</h2>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setIsPanelOpen(false)}
              className="h-8 w-8"
            >
              <X className="w-4 h-4" />
            </Button>
          </div>

          {/* Content */}
          <div className="flex-1 p-4 space-y-4">
            <div className="space-y-3">
              <Button
                variant="outline"
                className="w-full justify-start gap-2"
                onClick={() => {
                  setIsPanelOpen(false);
                  setInfoModalOpen(true);
                }}
              >
                <Info className="w-4 h-4" />
                Info
              </Button>

              {isActive && (
                <Button
                  asChild
                  variant="default"
                  className="w-full justify-start gap-2"
                  onClick={() => setIsPanelOpen(false)}
                >
                  <Link to={`/locations/${locationUuid}/edit`}>
                    <Edit className="w-4 h-4" />
                    Edit Location
                  </Link>
                </Button>
              )}

              <Button
                variant="destructive"
                className="w-full justify-start gap-2"
                onClick={() => {
                  setIsPanelOpen(false);
                  onDelete();
                }}
              >
                <Trash2 className="w-4 h-4" />
                Delete Location
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Info Modal */}
      <AlertDialog open={infoModalOpen} onOpenChange={setInfoModalOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Location Information</AlertDialogTitle>
            <AlertDialogDescription asChild>
              <div className="space-y-4 pt-2">
                <div>
                  <p className="font-semibold text-foreground mb-1">Name</p>
                  <p className="text-sm text-muted-foreground">{name}</p>
                </div>
                <div>
                  <p className="font-semibold text-foreground mb-1">Address</p>
                  <p className="text-sm text-muted-foreground">{address}</p>
                </div>
                {description && (
                  <div>
                    <p className="font-semibold text-foreground mb-1">Description</p>
                    <p className="text-sm text-muted-foreground">{description}</p>
                  </div>
                )}
                {!description && (
                  <div>
                    <p className="font-semibold text-foreground mb-1">Description</p>
                    <p className="text-sm text-muted-foreground italic">No description provided</p>
                  </div>
                )}
              </div>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <Button
              onClick={() => setInfoModalOpen(false)}
              className="w-full sm:w-auto"
            >
              Close
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
};

