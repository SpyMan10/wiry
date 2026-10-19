import { NodeCategory, PwNode, PwPort, sortPorts } from "../../pipewire"
import { cx } from "../ui/cx"
import { GroupHeader } from "./GroupHeader"
import { NodeCard } from "./NodeCard"
import { Patching } from "./usePatching"

export function NodeGroup({
  category,
  nodes,
  open,
  onToggle,
  portsByNode,
  patch
}: {
  category: NodeCategory
  nodes: PwNode[]
  open: boolean
  onToggle: () => void
  portsByNode: Map<number, PwPort[]>
  patch: Patching
}) {
  return (
    <div className={cx("group", `cat-${category}`, !open && "collapsed")}>
      <GroupHeader category={category} count={nodes.length} open={open} onToggle={onToggle} />
      {open &&
        nodes.map(n => (
          <NodeCard
            key={n.id}
            node={n}
            ports={sortPorts(portsByNode.get(n.id) ?? [])}
            patch={patch}
          />
        ))}
    </div>
  )
}
