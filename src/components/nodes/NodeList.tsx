import { useEffect, useMemo, useRef, useState } from "react"
import { t } from "../../i18n"
import { GraphSnapshot, NodeCategory, nodeCategory } from "../../pipewire"
import { usePersistentState } from "../../hooks/usePersistentState"
import { useRevealNode } from "../../hooks/useRevealNode"
import { useUiState } from "../../state"
import { FoldButton } from "../layout/FoldButton"
import { EmptyState } from "../ui/EmptyState"
import { PanelHeader } from "../ui/PanelHeader"
import { ScrollArea } from "../ui/ScrollArea"
import { NodeGroup } from "./NodeGroup"
import { NodeToolbar } from "./NodeToolbar"
import { groupNodes, portsByNodeOf } from "./grouping"
import { usePatching } from "./usePatching"

/** Left panel: connectable nodes grouped by category, with their ports and level meters. */
export function NodeList({ graph, onFold }: { graph: GraphSnapshot; onFold: () => void }) {
  const [query, setQuery] = useState("")
  const [showMonitors, setShowMonitors] = useState(false)
  const [collapsed, setCollapsed] = usePersistentState<NodeCategory[]>("wiry.collapsedGroups", [])

  const portsByNode = useMemo(
    () => portsByNodeOf(graph.ports, showMonitors),
    [graph.ports, showMonitors]
  )
  const groups = useMemo(
    () => groupNodes(graph.nodes, portsByNode, query),
    [graph.nodes, portsByNode, query]
  )
  const patch = usePatching(graph, portsByNode)
  // While filtering, every group is shown open so no match is hidden.
  const filtering = query.trim() !== ""

  const toggleGroup = (cat: NodeCategory) =>
    setCollapsed(collapsed.includes(cat) ? collapsed.filter(c => c !== cat) : [...collapsed, cat])

  // Selecting a node anywhere (graph, Volumes tab…) unfolds its group and reveals its card.
  const { selectedNode } = useUiState()
  const rootRef = useRef<HTMLElement>(null)
  useEffect(() => {
    const node = graph.nodes.find(n => n.id === selectedNode)
    const cat = node && nodeCategory(node)
    if (cat && collapsed.includes(cat)) setCollapsed(collapsed.filter(c => c !== cat))
    // Only when the selection changes, not on every graph update.
  }, [selectedNode]) // eslint-disable-line react-hooks/exhaustive-deps
  useRevealNode(rootRef, selectedNode)

  return (
    <section className="panel node-list" ref={rootRef}>
      <PanelHeader
        icon="hdd-network"
        title={t("panels.nodes")}
        count={groups.reduce((n, [, l]) => n + l.length, 0)}
      >
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
  )
}
