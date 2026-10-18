import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { t } from "../../i18n";
import { CATEGORY_ORDER, GraphSnapshot, NodeCategory, PwNode, PwPort, nodeCategory } from "../../pipewire";
import { useHorizontalWheel } from "../../hooks/useHorizontalWheel";
import { usePersistentState } from "../../hooks/usePersistentState";
import { useRevealNode } from "../../hooks/useRevealNode";
import { useUiState } from "../../state";
import { EmptyState } from "../ui/EmptyState";
import { SearchField } from "../ui/SearchField";
import { VolumeGroup } from "./VolumeGroup";
import { VolumeStrip } from "./VolumeStrip";

/** Mixer view: a vertical strip per node with a volume, grouped by category, with a filter in the header. */
export function VolumesTab({ graph, toolsSlot }: { graph: GraphSnapshot; toolsSlot: HTMLElement | null }) {
  const { selectedNode, setSelectedNode } = useUiState();
  const [filter, setFilter] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);
  useHorizontalWheel(scrollRef);
  const [collapsed, setCollapsed] = usePersistentState<NodeCategory[]>("wiry.volumeCollapsed", []);
  const toggle = (cat: NodeCategory) =>
    setCollapsed(collapsed.includes(cat) ? collapsed.filter((c) => c !== cat) : [...collapsed, cat]);
  // While filtering, every group is shown open so no match is hidden.
  const filtering = filter.trim() !== "";

  // Selecting a node anywhere (node list, graph…) unfolds its category and reveals its strip.
  useEffect(() => {
    const node = graph.nodes.find((n) => n.id === selectedNode);
    const cat = node && nodeCategory(node);
    if (cat && collapsed.includes(cat)) setCollapsed(collapsed.filter((c) => c !== cat));
    // Only when the selection changes, not on every graph update.
  }, [selectedNode]); // eslint-disable-line react-hooks/exhaustive-deps
  useRevealNode(scrollRef, selectedNode);

  const portsByNode = useMemo(() => {
    const m = new Map<number, PwPort[]>();
    for (const p of graph.ports) m.set(p.nodeId, [...(m.get(p.nodeId) ?? []), p]);
    return m;
  }, [graph.ports]);

  const groups = useMemo(() => {
    const q = filter.trim().toLowerCase();
    const g = new Map<NodeCategory, PwNode[]>();
    for (const n of graph.nodes) {
      if (!n.volume) continue;
      if (q && !`${n.description} ${n.name} ${n.appName ?? ""}`.toLowerCase().includes(q)) continue;
      const c = nodeCategory(n);
      g.set(c, [...(g.get(c) ?? []), n]);
    }
    for (const list of g.values()) list.sort((a, b) => a.description.localeCompare(b.description));
    return CATEGORY_ORDER.filter((c) => g.has(c)).map((c) => [c, g.get(c)!] as const);
  }, [graph.nodes, filter]);

  return (
    <>
      {toolsSlot &&
        createPortal(
          <div className="header-field">
            <SearchField value={filter} onChange={setFilter} placeholder={t("volumes.filter")} />
          </div>,
          toolsSlot,
        )}
      <div className="scroll bottom-scroll volumes-scroll" ref={scrollRef}>
        {groups.length ? (
          <div className="volume-groups">
            {groups.map(([cat, nodes]) => (
              <VolumeGroup
                key={cat}
                category={cat}
                count={nodes.length}
                open={filtering || !collapsed.includes(cat)}
                onToggle={() => toggle(cat)}
              >
                {nodes.map((n) => (
                  <VolumeStrip
                    key={n.id}
                    node={n}
                    ports={portsByNode.get(n.id) ?? []}
                    selected={selectedNode === n.id}
                    onSelect={() => setSelectedNode(selectedNode === n.id ? null : n.id)}
                  />
                ))}
              </VolumeGroup>
            ))}
          </div>
        ) : (
          <EmptyState>{t("volumes.empty")}</EmptyState>
        )}
      </div>
    </>
  );
}
