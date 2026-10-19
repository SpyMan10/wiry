import {
  CATEGORY_ORDER,
  GraphSnapshot,
  NodeIcon,
  PwLink,
  PwNode,
  PwPort,
  nodeCategory,
  nodeIcons,
  sortPorts
} from "../../pipewire"

/** Horizontal inner spacing of a node box (same as the node cards). */
export const NODE_PAD = 12
/**
 * Vertical inner spacing. Smaller than NODE_PAD because the title and port rows already add
 * ~4px around their text: the visible margin ends up the same on all four sides.
 */
export const NODE_PAD_Y = 8
/** Left inset of the content, clearing the category stripe. */
export const NODE_INSET = NODE_PAD + 8
/** Vertical centre of the title row (16px high): title, id and icons are centred on it. */
export const NODE_TITLE_Y = NODE_PAD_Y + 8
/** Top of the first port row: padding + title + gap. */
export const NODE_HEADER_H = NODE_PAD_Y + 16 + 4
export const PORT_ROW_H = 20
const GAP = 14
export const MARGIN = 16

/** A node box in one of the two columns, with only its linked ports. */
export interface Box {
  node: PwNode
  ports: PwPort[]
  icons: NodeIcon[]
  y: number
  h: number
}

export interface GraphLayout {
  /** Sources (nodes with linked outputs). */
  left: Box[]
  /** Destinations (nodes with linked inputs). */
  right: Box[]
  leftByNode: Map<number, Box>
  rightByNode: Map<number, Box>
  links: PwLink[]
  ports: Map<number, PwPort>
  height: number
}

/** Vertical centre of a port inside its box, in graph coordinates. */
export const portY = (box: Box, portId: number) =>
  box.y + NODE_HEADER_H + box.ports.findIndex(p => p.id === portId) * PORT_ROW_H + PORT_ROW_H / 2

function stack(items: Omit<Box, "y" | "h">[]): Box[] {
  let y = MARGIN
  return items.map(item => {
    const h = NODE_HEADER_H + item.ports.length * PORT_ROW_H + NODE_PAD_Y
    const box = { ...item, y, h }
    y += h + GAP
    return box
  })
}

/** Two-column layout of the linked part of the graph: sources left, destinations right. */
export function computeLayout(graph: GraphSnapshot): GraphLayout {
  const nodes = new Map(graph.nodes.map(n => [n.id, n]))
  const ports = new Map(graph.ports.map(p => [p.id, p]))
  const links = graph.links.filter(l => ports.has(l.outputPort) && ports.has(l.inputPort))

  // Icons are computed from all of a node's ports (e.g. "physical"), not only the linked ones.
  const allPortsByNode = new Map<number, PwPort[]>()
  for (const p of graph.ports)
    allPortsByNode.set(p.nodeId, [...(allPortsByNode.get(p.nodeId) ?? []), p])

  const collect = (dir: "out" | "in") => {
    const byNode = new Map<number, PwPort[]>()
    for (const l of links) {
      const port = ports.get(dir === "out" ? l.outputPort : l.inputPort)!
      const list = byNode.get(port.nodeId) ?? []
      if (!list.includes(port)) list.push(port)
      byNode.set(port.nodeId, list)
    }
    return [...byNode]
      .filter(([id]) => nodes.has(id))
      .map(([id, ps]) => {
        const node = nodes.get(id)!
        return { node, ports: sortPorts(ps), icons: nodeIcons(node, allPortsByNode.get(id) ?? []) }
      })
  }

  const catRank = (n: PwNode) => CATEGORY_ORDER.indexOf(nodeCategory(n))
  const left = stack(
    collect("out").sort(
      (a, b) =>
        catRank(a.node) - catRank(b.node) || a.node.description.localeCompare(b.node.description)
    )
  )
  const leftByNode = new Map(left.map(b => [b.node.id, b]))

  // Order destinations by the mean height of their sources to limit crossings.
  const barycenter = (nodeId: number) => {
    const ys = links
      .filter(l => l.inputNode === nodeId && leftByNode.has(l.outputNode))
      .map(l => portY(leftByNode.get(l.outputNode)!, l.outputPort))
    return ys.length ? ys.reduce((a, b) => a + b, 0) / ys.length : 0
  }
  const right = stack(collect("in").sort((a, b) => barycenter(a.node.id) - barycenter(b.node.id)))
  const rightByNode = new Map(right.map(b => [b.node.id, b]))

  const height = Math.max(...[...left, ...right].map(b => b.y + b.h + MARGIN), 200)
  return { left, right, leftByNode, rightByNode, links, ports, height }
}
