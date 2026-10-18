import { useState } from "react";
import { t } from "../../i18n";
import { cx } from "../ui/cx";

/**
 * A UI-scale volume (1 = 100 %) as a rounded percentage. With `onCommit`, clicking it turns it
 * into a field to type an exact value: Enter or leaving the field applies it, Escape cancels.
 */
export function Percent({
  value,
  small,
  max = 1.5,
  unit = true,
  onCommit,
}: {
  value: number;
  small?: boolean;
  /** Upper bound of the typed value (UI scale). */
  max?: number;
  /** Show the "%" sign after the number. */
  unit?: boolean;
  onCommit?: (value: number) => void;
}) {
  const [text, setText] = useState<string | null>(null);
  const shown = Math.round(value * 100);
  const className = cx("vol-pct", small && "small");
  const sign = unit ? "%" : "";

  if (!onCommit)
    return (
      <span className={className}>
        {shown}
        {sign}
      </span>
    );

  if (text === null) {
    return (
      <button type="button" className={cx(className, "editable")} title={t("volume.edit")} onClick={() => setText(String(shown))}>
        {shown}
        {sign}
      </button>
    );
  }

  const commit = () => {
    const typed = parseFloat(text.replace(",", ".").replace("%", ""));
    if (Number.isFinite(typed)) onCommit(Math.min(max, Math.max(0, typed / 100)));
    setText(null);
  };

  return (
    <span className={cx(className, "editing")}>
      <input
        className="vol-pct-input"
        value={text}
        inputMode="decimal"
        autoFocus
        onFocus={(e) => e.currentTarget.select()}
        onChange={(e) => setText(e.currentTarget.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") e.currentTarget.blur();
          if (e.key === "Escape") {
            // Don't let the app-wide Escape handler clear the selection too.
            e.stopPropagation();
            setText(null);
          }
        }}
      />
      {sign}
    </span>
  );
}
