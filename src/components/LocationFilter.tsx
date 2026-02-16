import { useState, useEffect } from "react";
import { PanelRight, X, Filter } from "lucide-react";
import { Button } from "@/components/ui/button";
import { storage } from "@/lib/storage";
import { cn } from "@/lib/utils";
import type { LocationFilter as LocationFilterType, LocationStatusFilter } from "@/types/location";

type LocationFilterProps = {
  value: LocationFilterType;
  onChange: (filter: LocationFilterType) => void;
};

export const LocationFilter = ({ value, onChange }: LocationFilterProps) => {
  const [isPanelOpen, setIsPanelOpen] = useState(false);
  const [isFilterExpanded, setIsFilterExpanded] = useState(false);

  useEffect(() => {
    // Load filter from localStorage on mount
    const savedFilter = storage.getLocationFilter();
    if (savedFilter) {
      onChange(savedFilter);
    }
  }, [onChange]);

  const handleFilterChange = (status: LocationStatusFilter) => {
    const newFilter: LocationFilterType = {
      status,
    };
    onChange(newFilter);
    storage.setLocationFilter(newFilter);
  };

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
              <h2 className="text-lg font-semibold">Panel</h2>
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
            {/* Filter Section */}
            <div className="space-y-2">
              <button
                type="button"
                onClick={() => setIsFilterExpanded(!isFilterExpanded)}
                className="w-full flex items-center justify-between p-3 rounded-lg border border-border hover:bg-accent transition-colors"
              >
                <div className="flex items-center gap-2">
                  <Filter className="w-4 h-4" />
                  <span className="text-sm font-medium text-foreground">Filter</span>
                </div>
                <div className={cn(
                  "transition-transform duration-200",
                  isFilterExpanded && "rotate-180"
                )}>
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M19 9l-7 7-7-7"
                    />
                  </svg>
                </div>
              </button>

              {/* Filter Options (Collapsible) */}
              {isFilterExpanded && (
                <div className="mt-2 space-y-2 pl-2" role="radiogroup" aria-label="Filter by status">
                <button
                  type="button"
                  role="radio"
                  aria-checked={value.status === "active"}
                  onClick={() => handleFilterChange("active")}
                  className={cn(
                    "w-full text-left px-4 py-3 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2",
                    value.status === "active"
                      ? "bg-primary text-primary-foreground border-primary"
                      : "bg-card hover:bg-accent border-border"
                  )}
                >
                  <div className="flex items-center gap-2">
                    <div className={cn(
                      "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                      value.status === "active"
                        ? "border-primary-foreground bg-primary-foreground"
                        : "border-muted-foreground"
                    )}>
                      {value.status === "active" && (
                        <div className="w-2 h-2 rounded-full bg-primary" />
                      )}
                    </div>
                    <div>
                      <div className="font-medium">Active</div>
                      <div className="text-xs opacity-80 mt-1">
                        Show only active locations
                      </div>
                    </div>
                  </div>
                </button>
                <button
                  type="button"
                  role="radio"
                  aria-checked={value.status === "all"}
                  onClick={() => handleFilterChange("all")}
                  className={cn(
                    "w-full text-left px-4 py-3 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2",
                    value.status === "all"
                      ? "bg-primary text-primary-foreground border-primary"
                      : "bg-card hover:bg-accent border-border"
                  )}
                >
                  <div className="flex items-center gap-2">
                    <div className={cn(
                      "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                      value.status === "all"
                        ? "border-primary-foreground bg-primary-foreground"
                        : "border-muted-foreground"
                    )}>
                      {value.status === "all" && (
                        <div className="w-2 h-2 rounded-full bg-primary" />
                      )}
                    </div>
                    <div>
                      <div className="font-medium">All</div>
                      <div className="text-xs opacity-80 mt-1">
                        Show all locations (active and inactive)
                      </div>
                    </div>
                  </div>
                </button>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </>
  );
};

