import { useState, useEffect } from "react";
import { PanelRight, X, Filter } from "lucide-react";
import { Button } from "@/components/ui/button";
import { storage } from "@/lib/storage";
import { cn } from "@/lib/utils";
import type { MqttBrokerFilter as MqttBrokerFilterType, MqttBrokerStatusFilter } from "@/types/mqttBroker";

type MqttBrokerFilterProps = {
  value: MqttBrokerFilterType;
  onChange: (filter: MqttBrokerFilterType) => void;
};

export const MqttBrokerFilter = ({ value, onChange }: MqttBrokerFilterProps) => {
  const [isPanelOpen, setIsPanelOpen] = useState(false);
  const [isFilterExpanded, setIsFilterExpanded] = useState(false);
  const [nameFilter, setNameFilter] = useState(value.name || "");
  const [portFilter, setPortFilter] = useState(value.port?.toString() || "");

  useEffect(() => {
    // Load filter from localStorage on mount
    const savedFilter = storage.getMqttBrokerFilter();
    if (savedFilter) {
      onChange(savedFilter);
      setNameFilter(savedFilter.name || "");
      setPortFilter(savedFilter.port?.toString() || "");
    }
  }, [onChange]);

  const applyFilters = () => {
    const newFilter: MqttBrokerFilterType = {
      status: value.status || "active",
      name: nameFilter.trim() || undefined,
      port: portFilter.trim() ? parseInt(portFilter, 10) : undefined,
      default: value.default,
      connected: value.connected,
    };
    onChange(newFilter);
    storage.setMqttBrokerFilter(newFilter);
  };

  const handleStatusChange = (status: MqttBrokerStatusFilter) => {
    const newFilter: MqttBrokerFilterType = {
      ...value,
      status,
    };
    onChange(newFilter);
    storage.setMqttBrokerFilter(newFilter);
  };

  const handleDefaultChange = (checked: boolean) => {
    const newFilter: MqttBrokerFilterType = {
      ...value,
      default: checked ? true : undefined,
    };
    onChange(newFilter);
    storage.setMqttBrokerFilter(newFilter);
  };

  const handleConnectedChange = (checked: boolean) => {
    const newFilter: MqttBrokerFilterType = {
      ...value,
      connected: checked ? true : undefined,
    };
    onChange(newFilter);
    storage.setMqttBrokerFilter(newFilter);
  };

  const clearFilters = () => {
    const emptyFilter: MqttBrokerFilterType = {};
    setNameFilter("");
    setPortFilter("");
    onChange(emptyFilter);
    storage.setMqttBrokerFilter(emptyFilter);
  };

  const hasActiveFilters = value.name || value.port || value.default || value.connected;

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
          <div className="flex-1 p-4 space-y-4 overflow-y-auto">
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
                  {hasActiveFilters && (
                    <span className="w-2 h-2 bg-primary rounded-full" />
                  )}
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
                <div className="mt-2 space-y-4 pl-2" role="radiogroup" aria-label="Filter by status">
                  {/* Status Filter */}
                  <div className="space-y-2">
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
                            Show only active brokers
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
                            Show all brokers (active and inactive)
                          </div>
                        </div>
                      </div>
                    </button>
                  </div>

                  {/* Name Filter */}
                  <div className="space-y-2">
                    <label htmlFor="filter-name" className="text-sm font-medium">
                      Name
                    </label>
                    <input
                      id="filter-name"
                      type="text"
                      placeholder="Search by name..."
                      value={nameFilter}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setNameFilter(e.target.value)}
                      onBlur={applyFilters}
                      onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => {
                        if (e.key === "Enter") {
                          applyFilters();
                        }
                      }}
                      className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary h-9"
                    />
                  </div>

                  {/* Port Filter */}
                  <div className="space-y-2">
                    <label htmlFor="filter-port" className="text-sm font-medium">
                      Port
                    </label>
                    <input
                      id="filter-port"
                      type="number"
                      placeholder="Filter by port..."
                      value={portFilter}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPortFilter(e.target.value)}
                      onBlur={applyFilters}
                      onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => {
                        if (e.key === "Enter") {
                          applyFilters();
                        }
                      }}
                      className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary h-9"
                      min="1"
                      max="65535"
                    />
                  </div>

                  {/* Default Filter */}
                  <div className="flex items-center space-x-2">
                    <input
                      id="filter-default"
                      type="checkbox"
                      checked={value.default === true}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => handleDefaultChange(e.target.checked)}
                      className="w-4 h-4 rounded border-input text-primary focus:ring-2 focus:ring-primary"
                    />
                    <label
                      htmlFor="filter-default"
                      className="text-sm font-medium cursor-pointer"
                    >
                      Default only
                    </label>
                  </div>

                  {/* Connected Filter */}
                  <div className="flex items-center space-x-2">
                    <input
                      id="filter-connected"
                      type="checkbox"
                      checked={value.connected === true}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => handleConnectedChange(e.target.checked)}
                      className="w-4 h-4 rounded border-input text-primary focus:ring-2 focus:ring-primary"
                    />
                    <label
                      htmlFor="filter-connected"
                      className="text-sm font-medium cursor-pointer"
                    >
                      Connected only
                    </label>
                  </div>

                  {/* Clear Filters Button */}
                  {hasActiveFilters && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={clearFilters}
                      className="w-full mt-4"
                    >
                      Clear Filters
                    </Button>
                  )}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </>
  );
};

