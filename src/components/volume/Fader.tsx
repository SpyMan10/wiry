import { ReactNode } from "react"
import { cx } from "../ui/cx"
import { Percent } from "./Percent"
import { VolumeSlider } from "./VolumeSlider"

/**
 * Vertical slider with its editable value above and a footer below (a label, or a button for
 * the master fader). Footers share one height so every slider of a strip ends at the same level.
 */
export function Fader({
  value,
  onChange,
  onEnd,
  footer,
  variant = "master"
}: {
  value: number
  onChange: (v: number) => void
  onEnd: () => void
  footer?: ReactNode
  /** "master": highlighted container; "channel": plain, smaller, duller fill. */
  variant?: "master" | "channel"
}) {
  return (
    <div className={cx("fader", variant)}>
      <Percent
        value={value}
        small={variant === "channel"}
        unit={false}
        onCommit={v => {
          onChange(v)
          onEnd()
        }}
      />
      <VolumeSlider value={value} onChange={onChange} onEnd={onEnd} orientation="vertical" />
      <div className="fader-footer">{footer}</div>
    </div>
  )
}
