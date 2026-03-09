import { cn } from "@/lib/utils";
import { ICON_COLORS } from "@/types/icon";

type ColorPaletteProps = {
  value: string | null | undefined;
  onChange: (hex: string | null) => void;
  allowEmpty?: boolean;
};

export const ColorPalette = ({
  value,
  onChange,
  allowEmpty = true,
}: ColorPaletteProps) => {
  return (
    <div className="flex flex-wrap gap-1">
      {allowEmpty && (
        <button
          type="button"
          onClick={() => onChange(null)}
          className={cn(
            "w-8 h-8 rounded-md border-2 transition-colors",
            !value
              ? "ring-2 ring-primary ring-offset-2 border-primary"
              : "border-border hover:border-muted-foreground/50"
          )}
          style={{ backgroundColor: "#f0f0f0" }}
          title="Sem cor"
          aria-label="No color"
        />
      )}
      {ICON_COLORS.map(({ hex, name }) => (
        <button
          key={hex}
          type="button"
          onClick={() => onChange(hex)}
          className={cn(
            "w-8 h-8 rounded-md border-2 transition-colors",
            value === hex
              ? "ring-2 ring-primary ring-offset-2 border-primary"
              : "border-border hover:border-muted-foreground/50"
          )}
          style={{ backgroundColor: hex }}
          title={name}
          aria-label={name}
        />
      ))}
    </div>
  );
};
