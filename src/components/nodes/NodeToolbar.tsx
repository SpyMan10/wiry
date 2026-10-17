import { t } from "../../i18n";
import { SearchField } from "../ui/SearchField";
import { ToggleButton } from "../ui/ToggleButton";

/** Search + "monitors" toggle strip under the node list header. */
export function NodeToolbar({
  query,
  onQueryChange,
  showMonitors,
  onShowMonitorsChange,
}: {
  query: string;
  onQueryChange: (q: string) => void;
  showMonitors: boolean;
  onShowMonitorsChange: (v: boolean) => void;
}) {
  return (
    <div className="toolbar">
      <SearchField value={query} onChange={onQueryChange} placeholder={t("nodeList.filter")} />
      <ToggleButton
        pressed={showMonitors}
        onToggle={onShowMonitorsChange}
        icon="broadcast"
        title={t("nodeList.monitorsTitle")}
      >
        {t("nodeList.monitors")}
      </ToggleButton>
    </div>
  );
}
