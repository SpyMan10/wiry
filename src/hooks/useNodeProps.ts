import { useEffect, useState } from "react"
import { GraphSnapshot, pw } from "../pipewire"

/** Full PipeWire properties of a node, refetched whenever the graph changes. */
export function useNodeProps(nodeId: number | null, graph: GraphSnapshot) {
  const [props, setProps] = useState<Record<string, string> | null>(null)
  useEffect(() => {
    if (nodeId === null) return setProps(null)
    let alive = true
    pw.nodeProps(nodeId).then(p => alive && setProps(p))
    return () => {
      alive = false
    }
  }, [nodeId, graph])
  return props
}
