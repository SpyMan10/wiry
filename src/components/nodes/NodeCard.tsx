import { PwNode, PwPort, hasMeter, nodeCategory } from "../../pipewire";
import { useUiState } from "../../state";
import { Sonograph } from "../meter/Sonograph";
import { cx } from "../ui/cx";
import { VolumeControl } from "../volume/VolumeControl";
import { NodeHeader } from "./NodeHeader";
import { PortList } from "./PortList";
import { Patching } from "./usePatching";

/** A node of the list: header, then level meter + ports, then volume. */
export function NodeCard({ node, ports, patch }: { node: PwNode; ports: PwPort[]; patch: Patching }) {
  const { selectedNode, setSelectedNode } = useUiState();
  const ins = ports.filter((p) => p.direction === "in");
  const outs = ports.filter((p) => p.direction === "out");
  const metered = hasMeter(node);

  return (
    <div
      data-node-id={node.id}
      className={cx(
        "node-card",
        `cat-${nodeCategory(node)}`,
        selectedNode === node.id && "selected",
        patch.armedNode === node.id && "armed",
      )}
    >
      <NodeHeader
        node={node}
        ports={ports}
        ins={ins}
        outs={outs}
        patch={patch}
        onSelect={() => setSelectedNode(selectedNode === node.id ? null : node.id)}
      />
      <div className={cx("node-body", metered && "has-meter")}>
        {metered && (
          <div className="node-meter">
            <Sonograph nodeId={node.id} />
          </div>
        )}
        <PortList ins={ins} outs={outs} patch={patch} />
      </div>
      {node.volume && <VolumeControl node={node} />}
    </div>
  );
}
