import { nodeCategory } from "../../pipewire"
import { useUiState } from "../../state"
import { cx } from "../ui/cx"
import { GraphPort } from "./GraphPort"
import { Box, NODE_INSET, NODE_PAD, NODE_PAD_Y, NODE_TITLE_Y } from "./layout"
import { SvgIcon } from "./SvgIcon"

const ICON_STEP = 16
const CHAR_W = 7

function truncate(s: string, n: number) {
  return s.length > n ? s.slice(0, Math.max(1, n - 1)) + "…" : s
}

/** A node box of the graph: title, type icons and linked ports. Click selects it. */
export function GraphNode({
  box,
  side,
  x,
  width,
  dim
}: {
  box: Box
  side: "left" | "right"
  x: number
  width: number
  dim: boolean
}) {
  const { selectedNode, setSelectedNode } = useUiState()
  const { node, icons } = box
  const selected = selectedNode === node.id

  return (
    <g
      className={cx("g-node", `cat-${nodeCategory(node)}`, selected && "selected", dim && "dim")}
      transform={`translate(${x},${box.y})`}
      onClick={() => setSelectedNode(selected ? null : node.id)}
    >
      <rect className="g-node-bg" width={width} height={box.h} rx={8} />
      <rect
        className="g-node-stripe"
        x={6}
        y={NODE_PAD_Y}
        width={3}
        height={box.h - 2 * NODE_PAD_Y}
        rx={1.5}
      />
      <text className="g-node-title" x={NODE_INSET} y={NODE_TITLE_Y}>
        <title>{node.description}</title>
        {truncate(
          node.description,
          Math.floor((width - NODE_INSET - NODE_PAD - icons.length * ICON_STEP) / CHAR_W)
        )}
      </text>
      {icons.map(({ icon, title }, i) => (
        <SvgIcon
          key={icon}
          className="g-node-icon"
          name={icon}
          title={title}
          x={width - NODE_PAD - (icons.length - 1 - i) * ICON_STEP}
          y={NODE_TITLE_Y + 1}
        />
      ))}
      {box.ports.map((p, i) => (
        <GraphPort key={p.id} port={p} index={i} side={side} boxWidth={width} />
      ))}
    </g>
  )
}
