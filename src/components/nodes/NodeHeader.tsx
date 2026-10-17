import { PwNode, PwPort, nodeIcons } from "../../pipewire";
import { NodeActions } from "./NodeActions";
import { NodeIcons } from "./NodeIcons";
import { Patching } from "./usePatching";

/** Card title row: name, type icons, and the link/connect action. Clicking selects the node. */
export function NodeHeader({
  node,
  ports,
  ins,
  outs,
  patch,
  onSelect,
}: {
  node: PwNode;
  ports: PwPort[];
  ins: PwPort[];
  outs: PwPort[];
  patch: Patching;
  onSelect: () => void;
}) {
  return (
    <div className="node-head" onClick={onSelect}>
      <div className="node-title" title={`${node.name} · #${node.id}`}>
        {node.description}
      </div>
      <NodeIcons icons={nodeIcons(node, ports)} />
      <NodeActions nodeId={node.id} ins={ins} outs={outs} patch={patch} />
    </div>
  );
}
