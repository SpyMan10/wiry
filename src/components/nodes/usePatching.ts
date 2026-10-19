import { useMemo } from "react"
import { GraphSnapshot, PwPort, matchPorts, pw } from "../../pipewire"
import { useUiState } from "../../state"

/** Click-to-connect logic of the node list: arming a port/node and wiring it to targets. */
export function usePatching(graph: GraphSnapshot, portsByNode: Map<number, PwPort[]>) {
  const { armed, setArmed } = useUiState()
  const armedPort = armed?.kind === "port" ? armed.port : null
  const armedNode = armed?.kind === "node" ? armed.nodeId : null

  const linkByPair = useMemo(() => {
    const m = new Map<string, number>()
    for (const l of graph.links) m.set(`${l.outputPort}:${l.inputPort}`, l.id)
    return m
  }, [graph.links])

  const linkCounts = useMemo(() => {
    const m = new Map<number, number>()
    for (const l of graph.links) {
      m.set(l.outputPort, (m.get(l.outputPort) ?? 0) + 1)
      m.set(l.inputPort, (m.get(l.inputPort) ?? 0) + 1)
    }
    return m
  }, [graph.links])

  const linkBetween = (a: PwPort, b: PwPort) => {
    const [out, inp] = a.direction === "out" ? [a, b] : [b, a]
    return linkByPair.get(`${out.id}:${inp.id}`)
  }

  /** Whether `p` can be wired to the armed port. */
  const isCompatible = (p: PwPort) =>
    !!armedPort &&
    armedPort.direction !== p.direction &&
    armedPort.mediaType === p.mediaType &&
    armedPort.nodeId !== p.nodeId

  /** Wires the armed node's outputs to `targets`, matching channels. */
  const connectArmedNodeTo = (targets: PwPort[]) => {
    if (armedNode === null) return
    const outs = (portsByNode.get(armedNode) ?? []).filter(p => p.direction === "out" && !p.monitor)
    for (const [o, i] of matchPorts(outs, targets)) {
      if (!linkByPair.has(`${o.id}:${i.id}`)) pw.link(o.id, i.id)
    }
  }

  /** Arms a port, or toggles its link with the armed port/node. Shift keeps the source armed. */
  const clickPort = (p: PwPort, keepArmed: boolean) => {
    if (armedPort?.id === p.id) return setArmed(null)
    if (armedPort && isCompatible(p)) {
      const existing = linkBetween(armedPort, p)
      if (existing !== undefined) pw.unlink(existing)
      else if (p.direction === "in") pw.link(armedPort.id, p.id)
      else pw.link(p.id, armedPort.id)
      if (!keepArmed) setArmed(null)
      return
    }
    if (armedNode !== null && p.direction === "in" && p.nodeId !== armedNode) {
      connectArmedNodeTo([p])
      if (!keepArmed) setArmed(null)
      return
    }
    setArmed({ kind: "port", port: p })
  }

  return {
    armed,
    armedPort,
    armedNode,
    linkCount: (portId: number) => linkCounts.get(portId) ?? 0,
    isCompatible,
    isLinkedToArmed: (p: PwPort) =>
      !!armedPort && isCompatible(p) && linkBetween(armedPort, p) !== undefined,
    clickPort,
    connectArmedNodeTo,
    toggleArmNode: (nodeId: number) =>
      setArmed(armedNode === nodeId ? null : { kind: "node", nodeId }),
    cancel: () => setArmed(null)
  }
}

export type Patching = ReturnType<typeof usePatching>
