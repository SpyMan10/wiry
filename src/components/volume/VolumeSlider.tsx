import { CSSProperties } from "react"
import { cx } from "../ui/cx"

/** Upper bound of the PipeWire volume sliders (UI scale, 1 = 100 %). */
export const MAX_VOLUME = 1.5

interface Props {
  value: number
  onChange: (v: number) => void
  onEnd: () => void
  /** Upper bound (default 150 %). Without headroom above 100 % no unity mark is drawn. */
  max?: number
  /** Value set on middle-click; `null` disables it. */
  resetValue?: number | null
  title?: string
  /** "vertical": 0 at the bottom, max at the top (length set in CSS, `--slider-length`). */
  orientation?: "horizontal" | "vertical"
}

/**
 * Volume range input (1 = 100 %), 0..150 % by default with a 100 % mark; a middle-click (wheel
 * click) resets it to `resetValue`. Horizontal by default, or vertical.
 */
export function VolumeSlider({
  value,
  onChange,
  onEnd,
  max = MAX_VOLUME,
  resetValue = 1,
  title,
  orientation = "horizontal"
}: Props) {
  const fill = Math.min(value / max, 1) * 100
  return (
    <input
      type="range"
      className={cx("slider", orientation, value > 1.0001 && "over", max <= 1 && "no-unity")}
      min={0}
      max={max}
      step={0.01}
      value={value}
      title={title}
      aria-orientation={orientation}
      style={{ "--fill": `${fill}%`, "--unity": `${100 / max}%` } as CSSProperties}
      onChange={e => onChange(parseFloat(e.currentTarget.value))}
      onPointerUp={onEnd}
      onKeyUp={onEnd}
      onBlur={onEnd}
      onMouseDown={e => {
        if (e.button !== 1 || resetValue === null) return
        // Middle button: reset, and keep Linux from starting an autoscroll or a primary paste.
        e.preventDefault()
        onChange(resetValue)
        onEnd()
      }}
      onAuxClick={e => e.button === 1 && e.preventDefault()}
    />
  )
}
