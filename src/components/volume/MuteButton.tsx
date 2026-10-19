import { t } from "../../i18n"
import { Button } from "../ui/Button"
import { cx } from "../ui/cx"

export function MuteButton({ muted, onToggle }: { muted: boolean; onToggle: () => void }) {
  return (
    <Button
      className={cx("mute", muted && "on")}
      icon={muted ? "volume-mute-fill" : "volume-up-fill"}
      title={muted ? t("volume.unmute") : t("volume.mute")}
      onClick={onToggle}
    />
  )
}
