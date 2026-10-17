import { t } from "../../i18n";
import { GraphSnapshot } from "../../pipewire";
import { Icon } from "../ui/Icon";

/** Shown while PipeWire is not (or no longer) connected. */
export function ConnectionBanner({ graph }: { graph: GraphSnapshot }) {
  if (graph.connected) return null;
  return (
    <div className="banner">
      <Icon name={graph.error ? "exclamation-triangle-fill" : "arrow-repeat"} /> {graph.error ?? t("app.connecting")}
    </div>
  );
}
