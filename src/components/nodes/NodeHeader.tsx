import { PwNode, PwPort, nodeIcons } from "../../pipewire"
import { NodeActions } from "./NodeActions"
import { NodeIcons } from "./NodeIcons"
import { NodeId } from "./NodeId"
import { Patching } from "./usePatching"

/** Card title row: name and type icons (click selects the node, as in the mixer), then the link/connect action. */
export function NodeHeader({
  node,
  ports,
  ins,
  outs,
  patch,
  onSelect
}: {
  node: PwNode
  ports: PwPort[]
  ins: PwPort[]
  outs: PwPort[]
  patch: Patching
  onSelect: () => void
}) {
  return (
    <div className="node-head">
      <button
        type="button"
        className="node-title-btn"
        title={`${node.name} · #${node.id}`}
        onClick={onSelect}
      >
        <span className="node-title">{node.description}</span>
        <NodeId id={node.id} />
        <NodeIcons icons={nodeIcons(node, ports)} />
      </button>
      <NodeActions nodeId={node.id} ins={ins} outs={outs} patch={patch} />
    </div>
  )
}
