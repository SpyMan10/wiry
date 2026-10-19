import { PwPort, portLabel } from "../../pipewire"
import { NODE_HEADER_H, NODE_INSET, NODE_PAD, PORT_ROW_H } from "./layout"

/** Port row of a graph box: label plus a dot on the box edge facing the wires. */
export function GraphPort({
  port,
  index,
  side,
  boxWidth
}: {
  port: PwPort
  index: number
  side: "left" | "right"
  boxWidth: number
}) {
  const y = NODE_HEADER_H + index * PORT_ROW_H + PORT_ROW_H / 2
  const onLeft = side === "left"
  return (
    <g>
      <text
        className="g-port-label"
        x={onLeft ? boxWidth - NODE_PAD : NODE_INSET}
        y={y + 4}
        textAnchor={onLeft ? "end" : "start"}
      >
        {portLabel(port)}
      </text>
      <polygon
        className={`g-port mt-${port.mediaType}`}
        points={flowTriangle(onLeft ? boxWidth : 0, y)}
      />
    </g>
  )
}

const TRI_HALF_H = 5
const TRI_BACK = 4
const TRI_TIP = 5

/**
 * Triangle centred on the box edge, pointing in the direction of the signal flow
 * (left to right: out of a source box, into a destination box).
 */
function flowTriangle(x: number, y: number) {
  return `${x - TRI_BACK},${y - TRI_HALF_H} ${x + TRI_TIP},${y} ${x - TRI_BACK},${y + TRI_HALF_H}`
}
