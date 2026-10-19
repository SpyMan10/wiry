import { Icon } from "./Icon"
import { cx } from "./cx"

export interface TabDef<T extends string> {
  id: T
  label: string
  icon: string
}

/** Tab strip sized to sit in a panel header. */
export function Tabs<T extends string>({
  tabs,
  active,
  onChange
}: {
  tabs: TabDef<T>[]
  active: T
  onChange: (id: T) => void
}) {
  return (
    <div className="tabs" role="tablist">
      {tabs.map(tab => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          aria-selected={tab.id === active}
          className={cx("tab", tab.id === active && "active")}
          onClick={() => onChange(tab.id)}
        >
          <Icon name={tab.icon} /> {tab.label}
        </button>
      ))}
    </div>
  )
}
