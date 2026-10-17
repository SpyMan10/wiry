import { t } from "../../i18n";
import { Icon } from "../ui/Icon";

/** Thin vertical strip shown in place of a folded side panel. */
export function FoldRail({
  side,
  icon,
  label,
  onUnfold,
}: {
  side: "left" | "right";
  icon: string;
  label: string;
  onUnfold: () => void;
}) {
  return (
    <button type="button" className={`fold-rail ${side}`} title={t("fold.expand", { label })} onClick={onUnfold}>
      <span className="rail-head">
        <Icon name={side === "left" ? "chevron-bar-right" : "chevron-bar-left"} />
      </span>
      <span className="rail-body">
        <Icon name={icon} className="rail-icon" />
        <span className="rail-label">{label}</span>
      </span>
    </button>
  );
}
