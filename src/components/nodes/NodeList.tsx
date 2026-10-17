import { useMemo, useState } from "react";
import { t } from "../../i18n";
import { GraphSnapshot, NodeCategory } from "../../pipewire";
import { usePersistentState } from "../../hooks/usePersistentState";
import { FoldButton } from "../layout/FoldButton";
import { EmptyState } from "../ui/EmptyState";
import { PanelHeader } from "../ui/PanelHeader";
import { ScrollArea } from "../ui/ScrollArea";
import { NodeGroup } from "./NodeGroup";
import { NodeToolbar } from "./NodeToolbar";
import { groupNodes, portsByNodeOf } from "./grouping";
import { usePatching } from "./usePatching";

/** Left panel: connectable nodes grouped by category, with ports, meters and volumes. */
export function NodeList({ graph, onFold }: { graph: GraphSnapshot; onFold: () => void }) {
  const [query, setQuery] = useState("");
  const [showMonitors, setShowMonitors] = useState(false);
  const [collapsed, setCollapsed] = usePersistentState<NodeCategory[]>("wiry.collapsedGroups", []);

  const portsByNode = useMemo(() => portsByNodeOf(graph.ports, showMonitors), [graph.ports, showMonitors]);
  const groups = useMemo(() => groupNodes(graph.nodes, portsByNode, query), [graph.nodes, portsByNode, query]);
  const patch = usePatching(graph, portsByNode);
  // While filtering, every group is shown open so no match is hidden.
  const filtering = query.trim() !== "";

  const toggleGroup = (cat: NodeCategory) =>
    setCollapsed(collapsed.includes(cat) ? collapsed.filter((c) => c !== cat) : [...collapsed, cat]);


  return (
    <section className="panel node-list">
      <PanelHeader icon="hdd-network" title={t("panels.nodes")} count={groups.reduce((n, [, l]) => n + l.length, 0)}>
        <FoldButton side="left" onFold={onFold} />
      </PanelHeader>
      <NodeToolbar
        query={query}
        onQueryChange={setQuery}
        showMonitors={showMonitors}
        onShowMonitorsChange={setShowMonitors}
      />
      <ScrollArea>
        {groups.map(([cat, nodes]) => (
          <NodeGroup
            key={cat}
            category={cat}
            nodes={nodes}
            open={filtering || !collapsed.includes(cat)}
            onToggle={() => toggleGroup(cat)}
            portsByNode={portsByNode}
            patch={patch}
          />
        ))}
        {!groups.length && <EmptyState>{t("nodeList.empty")}</EmptyState>}
      </ScrollArea>
    </section>
  );
}
