import { t } from "../../i18n";
import { PwPort, portLabel } from "../../pipewire";
import { Icon } from "../ui/Icon";
import { cx } from "../ui/cx";
import { Patching } from "./usePatching";

/** A clickable port: arms it, or wires/unwires it with the armed port. */
export function PortButton({ port, patch }: { port: PwPort; patch: Patching }) {
  const armed = patch.armedPort?.id === port.id;
  const compatible = patch.isCompatible(port);
  const linked = patch.isLinkedToArmed(port);
  const count = patch.linkCount(port.id);
  const arrow = <Icon name="arrow-right-short" className="arrow" />;

  return (
    <button
      type="button"
      className={cx(
        "port",
        `mt-${port.mediaType}`,
        armed && "armed",
        compatible && "compatible",
        linked && "linked",
        patch.armed && !compatible && !armed && "dim",
      )}
      title={`${port.alias ?? port.name} · #${port.id}${linked ? ` — ${t("nodeList.portClickToDisconnect")}` : ""}`}
      onClick={(e) => patch.clickPort(port, e.shiftKey)}
    >
      {port.direction === "in" && arrow}
      <span className="dot" />
      <span className="port-name">{portLabel(port)}</span>
      {count > 0 && <span className="badge">{count}</span>}
      {port.direction === "out" && arrow}
    </button>
  );
}
