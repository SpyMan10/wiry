import { useMemo, useRef } from "react";
import { t } from "../../i18n";
import { GraphSnapshot, pw } from "../../pipewire";
import { useElementSize } from "../../hooks/useElementSize";
import { useUiState } from "../../state";
import { RefreshButton } from "../layout/RefreshButton";
import { Button } from "../ui/Button";
import { EmptyState } from "../ui/EmptyState";
import { PanelHeader } from "../ui/PanelHeader";
import { GraphNode } from "./GraphNode";
import { Box, MARGIN, computeLayout, portY } from "./layout";
import { Wire } from "./Wire";

/** Minimum horizontal room (px) for the wires between the two columns. */
const MIN_WIRE_SPAN = 120;

/** Centre panel: the wired part of the graph, sources on the left and destinations on the right. */
export function PatchGraph({ graph }: { graph: GraphSnapshot }) {
  const { selectedNode, setSelectedNode } = useUiState();
  const wrapRef = useRef<HTMLDivElement>(null);
  const { width } = useElementSize(wrapRef, { width: 600, height: 0 });
  const layout = useMemo(() => computeLayout(graph), [graph]);

  // The selected node and its wires stand out; everything else is dimmed.
  const focus = selectedNode;
  const isActive = (outNode: number, inNode: number) => focus === null || focus === outNode || focus === inNode;

  const nodeW = Math.max(140, Math.min(240, width * 0.34));
  // Below this the two columns would overlap: lay out at this width and scroll horizontally.
  const graphW = Math.max(width, 2 * (nodeW + MARGIN) + MIN_WIRE_SPAN);
  const leftX = MARGIN;
  const rightX = graphW - MARGIN - nodeW;



  const isDim = (box: Box, side: "left" | "right") =>
    focus !== null &&
    focus !== box.node.id &&
    !layout.links.some(
      (l) => isActive(l.outputNode, l.inputNode) && (side === "left" ? l.outputNode : l.inputNode) === box.node.id,
    );

  return (
    <section className="panel graph">
      <PanelHeader icon="bezier2" title={t("panels.connections")} count={layout.links.length}>
        {selectedNode !== null && (
          <Button className="link-btn" icon="eye" onClick={() => setSelectedNode(null)}>
            {t("graph.showAll")}
          </Button>
        )}
        <RefreshButton graph={graph} />
      </PanelHeader>
      <div className="scroll graph-scroll" ref={wrapRef}>
        {layout.links.length === 0 ? (
          <EmptyState>{t("graph.empty")}</EmptyState>
        ) : (
          <svg width={graphW} height={layout.height} className="graph-svg">
            <g className="wires">
              {layout.links.map((l) => {
                const a = layout.leftByNode.get(l.outputNode);
                const b = layout.rightByNode.get(l.inputNode);
                if (!a || !b) return null;
                const active = isActive(l.outputNode, l.inputNode);
                return (
                  <Wire
                    key={l.id}
                    x1={leftX + nodeW}
                    y1={portY(a, l.outputPort)}
                    x2={rightX}
                    y2={portY(b, l.inputPort)}
                    mediaType={layout.ports.get(l.outputPort)?.mediaType ?? "other"}
                    state={!active ? "dim" : focus !== null ? "focus" : "normal"}
                    levelNode={l.outputNode}
                    onDisconnect={() => pw.unlink(l.id)}
                  />
                );
              })}
            </g>
            {layout.left.map((b) => (
              <GraphNode key={`left-${b.node.id}`} box={b} side="left" x={leftX} width={nodeW} dim={isDim(b, "left")} />
            ))}
            {layout.right.map((b) => (
              <GraphNode key={`right-${b.node.id}`} box={b} side="right" x={rightX} width={nodeW} dim={isDim(b, "right")} />
            ))}
          </svg>
        )}
      </div>
    </section>
  );
}
