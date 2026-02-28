import { useState } from "react";
import { PanelRight, X, Filter } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type {
  CollectorNotificationFilter as CollectorNotificationFilterType,
  CollectorNotificationIsReadFilter,
  CollectorNotificationSeverityFilter,
} from "@/types/collectorNotifications";

const IS_READ_OPTIONS: { value: CollectorNotificationIsReadFilter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "no_read", label: "Not read" },
  { value: "is_read", label: "Read" },
];

const SEVERITY_OPTIONS: { value: CollectorNotificationSeverityFilter; label: string }[] = [
  { value: "All", label: "All" },
  { value: "Critical", label: "Critical" },
  { value: "Warn", label: "Warn" },
  { value: "Info", label: "Info" },
];

type CollectorNotificationFilterProps = {
  value: CollectorNotificationFilterType;
  onChange: (filter: CollectorNotificationFilterType) => void;
};

export const CollectorNotificationFilter = ({ value, onChange }: CollectorNotificationFilterProps) => {
  const [isPanelOpen, setIsPanelOpen] = useState(false);
  const [isFilterExpanded, setIsFilterExpanded] = useState(true);

  const isRead = value.is_read ?? "no_read";
  const severity = value.severity ?? "All";

  const hasActiveFilters = isRead !== "no_read" || severity !== "All";

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
          isPanelOpen ? "translate-x-0" : "translate-x-full",
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
            <button
              type="button"
              onClick={() => setIsFilterExpanded(!isFilterExpanded)}
              className="w-full flex items-center justify-between p-3 rounded-lg border border-border hover:bg-accent transition-colors"
            >
              <div className="flex items-center gap-2">
                <Filter className="w-4 h-4" />
                <span className="text-sm font-medium text-foreground">Filter</span>
                {hasActiveFilters && <span className="w-2 h-2 bg-primary rounded-full" />}
              </div>
              <div
                className={cn(
                  "transition-transform duration-200",
                  isFilterExpanded && "rotate-180",
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
              <div className="mt-2 space-y-4 pl-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Read status</label>
                  <div className="space-y-2">
                    {IS_READ_OPTIONS.map((opt) => (
                      <button
                        key={opt.value}
                        type="button"
                        role="radio"
                        aria-checked={isRead === opt.value}
                        onClick={() => onChange({ ...value, is_read: opt.value })}
                        className={cn(
                          "w-full text-left px-4 py-3 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2",
                          isRead === opt.value
                            ? "bg-primary text-primary-foreground border-primary"
                            : "bg-card hover:bg-accent border-border",
                        )}
                      >
                        <div className="flex items-center gap-2">
                          <div
                            className={cn(
                              "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                              isRead === opt.value
                                ? "border-primary-foreground bg-primary-foreground"
                                : "border-muted-foreground",
                            )}
                          >
                            {isRead === opt.value && (
                              <div className="w-2 h-2 rounded-full bg-primary" />
                            )}
                          </div>
                          <span className="font-medium">{opt.label}</span>
                        </div>
                      </button>
                    ))}
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">Severity</label>
                  <div className="space-y-2">
                    {SEVERITY_OPTIONS.map((opt) => (
                      <button
                        key={opt.value}
                        type="button"
                        role="radio"
                        aria-checked={severity === opt.value}
                        onClick={() => onChange({ ...value, severity: opt.value })}
                        className={cn(
                          "w-full text-left px-4 py-3 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2",
                          severity === opt.value
                            ? "bg-primary text-primary-foreground border-primary"
                            : "bg-card hover:bg-accent border-border",
                        )}
                      >
                        <div className="flex items-center gap-2">
                          <div
                            className={cn(
                              "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                              severity === opt.value
                                ? "border-primary-foreground bg-primary-foreground"
                                : "border-muted-foreground",
                            )}
                          >
                            {severity === opt.value && (
                              <div className="w-2 h-2 rounded-full bg-primary" />
                            )}
                          </div>
                          <span className="font-medium">{opt.label}</span>
                        </div>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  );
};
