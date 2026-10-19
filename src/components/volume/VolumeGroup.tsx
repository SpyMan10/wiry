import { ReactNode } from "react"
import { t } from "../../i18n"
import { CATEGORY_ICONS, NodeCategory, categoryLabel } from "../../pipewire"
import { Icon } from "../ui/Icon"
import { cx } from "../ui/cx"

/**
 * A category of the mixer: vertical title (click folds/unfolds it) followed by its strips.
 * Folded, only the title column remains, with the number of nodes.
 */
export function VolumeGroup({
  category,
  count,
  open,
  onToggle,
  children
}: {
  category: NodeCategory
  count: number
  open: boolean
  onToggle: () => void
  children: ReactNode
}) {
  return (
    <section className={cx("volume-group", `cat-${category}`, !open && "collapsed")}>
      <h3>
        <button
          type="button"
          className="volume-group-toggle"
          title={t(open ? "nodeList.collapseGroup" : "nodeList.expandGroup")}
          aria-expanded={open}
          onClick={onToggle}
        >
          <Icon name="chevron-down" className="chevron" />
          <Icon name={CATEGORY_ICONS[category]} className="cat-icon" />
          <span className="volume-group-label">{categoryLabel(category)}</span>
          {!open && <span className="group-count">{count}</span>}
        </button>
      </h3>
      {open && <div className="volume-group-strips">{children}</div>}
    </section>
  )
}
