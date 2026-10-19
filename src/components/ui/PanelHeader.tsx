import { ReactNode } from "react"
import { Icon } from "./Icon"

/** Fixed-height panel title bar: icon, title, optional count, then actions. */
export function PanelHeader({
  icon,
  title,
  count,
  children
}: {
  icon: string
  title: string
  count?: number
  children?: ReactNode
}) {
  return (
    <header className="panel-header">
      <h2>
        <Icon name={icon} /> {title}
      </h2>
      {count !== undefined && <span className="count">{count}</span>}
      {children}
    </header>
  )
}
