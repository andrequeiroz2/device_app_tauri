import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Terminal } from "lucide-react";

const PROVISIONING_LOG_EVENT = "provisioning-log";

type Payload = { message?: string };

export function SerialConsole() {
  const [lines, setLines] = useState<string[]>([]);

  useEffect(() => {
    const unlisten = listen<Payload>(PROVISIONING_LOG_EVENT, (event) => {
      const msg = event.payload?.message;
      if (typeof msg === "string") {
        setLines((prev) => [...prev, msg]);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="rounded-lg border border-border bg-muted/30 overflow-hidden">
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border bg-muted/50">
        <Terminal className="w-4 h-4 text-muted-foreground" />
        <span className="text-sm font-medium">Console</span>
      </div>
      <div className="p-3 font-mono text-xs max-h-[200px] overflow-y-auto min-h-[120px]">
        {lines.length === 0 ? (
          <p className="text-muted-foreground">Waiting for output…</p>
        ) : (
          lines.map((line, i) => (
            <div key={i} className="whitespace-pre-wrap break-all">
              {line}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
