import { t } from "../../i18n"
import { CATEGORY_ICONS, NodeCategory, categoryLabel } from "../../pipewire"
import { Icon } from "../ui/Icon"

/** Foldable category title: chevron, icon, label and node count. */
export function GroupHeader({
  category,
  count,
  open,
  onToggle
}: {
  category: NodeCategory
  count: number
  open: boolean
  onToggle: () => void
}) {
  return (
    <h3>
      <button
        type="button"
        className="group-toggle"
        title={t(open ? "nodeList.collapseGroup" : "nodeList.expandGroup")}
        aria-expanded={open}
        onClick={onToggle}
      >
        <Icon name="chevron-right" className="chevron" />
        <Icon name={CATEGORY_ICONS[category]} className="cat-icon" />
        <span className="group-label">{categoryLabel(category)}</span>
        <span className="group-count">{count}</span>
      </button>
    </h3>
  )
}
