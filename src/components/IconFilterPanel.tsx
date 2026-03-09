import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import { PanelRight, X, Filter } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { IconFilter, IconStatusFilter } from "@/types/icon";
import { storage } from "@/lib/storage";

type IconFilterPanelProps = {
  value: IconFilter;
  onChange: (filter: IconFilter) => void;
};

export const IconFilterPanel = ({ value, onChange }: IconFilterPanelProps) => {
  const [isPanelOpen, setIsPanelOpen] = useState(false);
  const [isFilterExpanded, setIsFilterExpanded] = useState(false);

  useEffect(() => {
    const savedFilter = storage.getIconFilter();
    if (savedFilter?.status) {
      onChange(savedFilter);
    }
  }, [onChange]);

  const handleStatusChange = (status: IconStatusFilter) => {
    const newFilter: IconFilter = { ...value, status };
    onChange(newFilter);
    storage.setIconFilter(newFilter);
  };

  const hasActiveFilters = value.status === "all";

  return (
    <>
      <Button
        variant="outline"
        size="sm"
        onClick={() => setIsPanelOpen(true)}
        className="flex items-center gap-2"
      >
        <PanelRight className="w-4 h-4" />
        Panel
      </Button>

      {isPanelOpen && (
        <div
          className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm"
          onClick={() => setIsPanelOpen(false)}
        />
      )}

      <div
        className={cn(
          "fixed top-0 right-0 z-[60] h-full w-80 bg-background border-l border-border shadow-lg transition-transform duration-300 ease-in-out",
          isPanelOpen ? "translate-x-0" : "translate-x-full"
        )}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex flex-col h-full">
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

          <div className="flex-1 p-4 space-y-4 overflow-y-auto">
            <Button asChild variant="outline" size="sm" className="w-full justify-center">
              <Link to="/icons/create">
                Add icon
              </Link>
            </Button>

            <div className="space-y-2">
              <button
                type="button"
                onClick={() => setIsFilterExpanded(!isFilterExpanded)}
                className="w-full flex items-center justify-between p-3 rounded-lg border border-border hover:bg-accent transition-colors"
              >
                <div className="flex items-center gap-2">
                  <Filter className="w-4 h-4" />
                  <span className="text-sm font-medium text-foreground">Filter</span>
                  {hasActiveFilters && (
                    <span className="w-2 h-2 bg-primary rounded-full" />
                  )}
                </div>
                <div
                  className={cn(
                    "transition-transform duration-200",
                    isFilterExpanded && "rotate-180"
                  )}
                >
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

              {isFilterExpanded && (
                <div className="mt-2 space-y-4 pl-2" role="radiogroup" aria-label="Filter by status">
                  <button
                    type="button"
                    role="radio"
                    aria-checked={value.status === "active"}
                    onClick={() => handleStatusChange("active")}
                    className={cn(
                      "w-full text-left px-4 py-3 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2",
                      value.status === "active"
                        ? "bg-primary text-primary-foreground border-primary"
                        : "bg-card hover:bg-accent border-border"
                    )}
                  >
                    <div className="flex items-center gap-2">
                      <div
                        className={cn(
                          "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                          value.status === "active"
                            ? "border-primary-foreground bg-primary-foreground"
                            : "border-muted-foreground"
                        )}
                      >
                        {value.status === "active" && (
                          <div className="w-2 h-2 rounded-full bg-primary" />
                        )}
                      </div>
                      <div>
                        <div className="font-medium">Active</div>
                        <div className="text-xs opacity-80 mt-1">
                          Show only active icons
                        </div>
                      </div>
                    </div>
                  </button>
                  <button
                    type="button"
                    role="radio"
                    aria-checked={value.status === "all"}
                    onClick={() => handleStatusChange("all")}
                    className={cn(
                      "w-full text-left px-4 py-3 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2",
                      value.status === "all"
                        ? "bg-primary text-primary-foreground border-primary"
                        : "bg-card hover:bg-accent border-border"
                    )}
                  >
                    <div className="flex items-center gap-2">
                      <div
                        className={cn(
                          "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                          value.status === "all"
                            ? "border-primary-foreground bg-primary-foreground"
                            : "border-muted-foreground"
                        )}
                      >
                        {value.status === "all" && (
                          <div className="w-2 h-2 rounded-full bg-primary" />
                        )}
                      </div>
                      <div>
                        <div className="font-medium">All</div>
                        <div className="text-xs opacity-80 mt-1">
                          Show all icons (active and inactive)
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
