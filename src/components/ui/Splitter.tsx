import { PointerEvent, useRef } from "react"
import { cx } from "./cx"

interface Props {
  /** "x": drag left/right to resize a width; "y": drag up/down to resize a height. */
  axis: "x" | "y"
  value: number
  onChange: (size: number) => void
  min: number
  /** Upper bound, evaluated when the drag starts (e.g. from the window size). */
  max: () => number
  /** Size restored on double-click. */
  defaultValue: number
  /** The panel grows when dragging towards negative coordinates (left / up). */
  invert?: boolean
  className?: string
}

/** Drag handle resizing an adjacent panel; double-click restores the default size. */
export function Splitter({
  axis,
  value,
  onChange,
  min,
  max,
  defaultValue,
  invert,
  className
}: Props) {
  const drag = useRef<{ start: number; size: number; max: number } | null>(null)
  const pos = (e: PointerEvent) => (axis === "x" ? e.clientX : e.clientY)

  return (
    <div
      className={cx("splitter", `splitter-${axis}`, className)}
      role="separator"
      aria-orientation={axis === "x" ? "vertical" : "horizontal"}
      onPointerDown={e => {
        e.currentTarget.setPointerCapture(e.pointerId)
        drag.current = { start: pos(e), size: value, max: max() }
        document.body.classList.add(`resizing-${axis}`)
      }}
      onPointerMove={e => {
        const d = drag.current
        if (!d) return
        const delta = (pos(e) - d.start) * (invert ? -1 : 1)
        onChange(Math.round(Math.min(Math.max(d.size + delta, min), Math.max(min, d.max))))
      }}
      onPointerUp={e => {
        e.currentTarget.releasePointerCapture(e.pointerId)
        drag.current = null
        document.body.classList.remove(`resizing-${axis}`)
      }}
      onDoubleClick={() => onChange(defaultValue)}
    />
  )
}
