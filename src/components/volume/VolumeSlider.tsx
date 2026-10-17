import { CSSProperties } from "react";
import { cx } from "../ui/cx";

/** Upper bound of the PipeWire volume sliders (UI scale, 1 = 100 %). */
export const MAX_VOLUME = 1.5;

interface Props {
  value: number;
  onChange: (v: number) => void;
  onEnd: () => void;
  /** Upper bound (default 150 %). Without headroom above 100 % no unity mark is drawn. */
  max?: number;
  /** Value set on double-click; `null` disables it. */
  resetValue?: number | null;
  title?: string;
}

/** Volume range input (1 = 100 %), 0..150 % by default with a 100 % mark; double-click resets to `resetValue`. */
export function VolumeSlider({ value, onChange, onEnd, max = MAX_VOLUME, resetValue = 1, title }: Props) {
  const fill = Math.min(value / max, 1) * 100;
  return (
    <input
      type="range"
      className={cx("slider", value > 1.0001 && "over", max <= 1 && "no-unity")}
      min={0}
      max={max}
      step={0.01}
      value={value}
      title={title}
      style={{ "--fill": `${fill}%`, "--unity": `${100 / max}%` } as CSSProperties}
      onChange={(e) => onChange(parseFloat(e.currentTarget.value))}
      onPointerUp={onEnd}
      onKeyUp={onEnd}
      onBlur={onEnd}
      onDoubleClick={() => {
        if (resetValue === null) return;
        onChange(resetValue);
        onEnd();
      }}
    />
  );
}
