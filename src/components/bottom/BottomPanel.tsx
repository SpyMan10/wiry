import { useState } from "react";
import { t } from "../../i18n";
import { GraphSnapshot } from "../../pipewire";
import { usePersistentState } from "../../hooks/usePersistentState";
import { AlsaMixerTab } from "../alsa/AlsaMixerTab";
import { PropertiesTab } from "../properties/PropertiesTab";
import { VolumesTab } from "../volume/VolumesTab";
import { Button } from "../ui/Button";
import { Splitter } from "../ui/Splitter";
import { TabDef, Tabs } from "../ui/Tabs";
import { cx } from "../ui/cx";

type TabId = "properties" | "volumes" | "alsa";

const TABS: TabDef<TabId>[] = [
  { id: "properties", label: t("bottom.properties"), icon: "card-list" },
  { id: "volumes", label: t("bottom.volumes"), icon: "volume-up" },
  { id: "alsa", label: t("bottom.alsa"), icon: "sliders" },
];

const HEIGHT_DEFAULT = 256;
const HEIGHT_MIN = 120;

/**
 * Foldable, resizable bottom panel with tabs. Each tab renders its body here and its own
 * header tools (filter, card picker…) into the header slot.
 */
export function BottomPanel({ graph }: { graph: GraphSnapshot }) {
  const [folded, setFolded] = usePersistentState("wiry.propertiesFolded", false);
  const [height, setHeight] = usePersistentState("wiry.propertiesHeight", HEIGHT_DEFAULT);
  const [tab, setTab] = usePersistentState<TabId>("wiry.bottomTab", "properties");
  const [toolsSlot, setToolsSlot] = useState<HTMLDivElement | null>(null);

  const selectTab = (id: TabId) => {
    setTab(id);
    if (folded) setFolded(false);
  };

  return (
    <section className={cx("panel bottom-panel", folded && "folded")} style={folded ? undefined : { height }}>
      {!folded && (
        <Splitter
          axis="y"
          className="bottom-splitter"
          value={height}
          onChange={setHeight}
          min={HEIGHT_MIN}
          max={() => window.innerHeight * 0.7}
          defaultValue={HEIGHT_DEFAULT}
          invert
        />
      )}
      <header className="panel-header bottom-header">
        <Tabs tabs={TABS} active={tab} onChange={selectTab} />
        <div className="bottom-tools" ref={setToolsSlot} />
        <Button
          className="fold-btn"
          icon={folded ? "chevron-bar-up" : "chevron-bar-down"}
          title={t(folded ? "bottom.expand" : "bottom.collapse")}
          onClick={() => setFolded(!folded)}
        />
      </header>
      {!folded &&
        (tab === "properties" ? (
          <PropertiesTab graph={graph} toolsSlot={toolsSlot} />
        ) : tab === "volumes" ? (
          <VolumesTab graph={graph} toolsSlot={toolsSlot} />
        ) : (
          <AlsaMixerTab graph={graph} toolsSlot={toolsSlot} />
        ))}
    </section>
  );
}
