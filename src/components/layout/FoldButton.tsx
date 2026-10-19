import { t } from "../../i18n"
import { Button } from "../ui/Button"

/** Panel-header button that folds a side panel. */
export function FoldButton({ side, onFold }: { side: "left" | "right"; onFold: () => void }) {
  return (
    <Button
      className="fold-btn"
      icon={side === "left" ? "chevron-bar-left" : "chevron-bar-right"}
      title={t("fold.collapse")}
      onClick={onFold}
    />
  )
}
