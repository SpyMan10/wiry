import { t } from "../../i18n";
import { portLabel } from "../../pipewire";
import { useUiState } from "../../state";
import { Button } from "../ui/Button";
import { Icon } from "../ui/Icon";

/** Floating notice, centred over the whole window, explaining what to click next while connecting. */
export function ArmedHint() {
  const { armed, setArmed } = useUiState();
  if (!armed) return null;
  const armedPort = armed.kind === "port" ? armed.port : null;

  return (
    <div className="armed-hint" role="status">
      <Icon name="link-45deg" className="armed-hint-icon" />
      <span className="armed-hint-text">
        {armedPort
          ? t("nodeList.armedPort", {
              port: portLabel(armedPort),
              direction: t(armedPort.direction === "out" ? "nodeList.directionInput" : "nodeList.directionOutput"),
            })
          : t("nodeList.armedNode")}
      </span>
      <Button className="armed-hint-close" icon="x-lg" title={t("nodeList.cancel")} onClick={() => setArmed(null)} />
    </div>
  );
}
