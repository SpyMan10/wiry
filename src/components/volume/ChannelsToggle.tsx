import { t } from "../../i18n";
import { Button } from "../ui/Button";

/** Shows/hides the per-channel sliders; renders an empty slot when there's a single channel. */
export function ChannelsToggle({
  visible,
  expanded,
  onToggle,
}: {
  visible: boolean;
  expanded: boolean;
  onToggle: () => void;
}) {
  if (!visible) return <span className="expand" />;
  return (
    <Button
      className="expand"
      icon={expanded ? "chevron-down" : "chevron-right"}
      title={t("volume.channels")}
      onClick={onToggle}
    />
  );
}
