import { ReactNode, createContext, useContext, useEffect, useState } from "react"
import { GraphSnapshot, PwPort } from "./pipewire"

/** Port or node picked as the source of a connection being made. */
export type Armed = { kind: "port"; port: PwPort } | { kind: "node"; nodeId: number } | null

interface UiState {
  selectedNode: number | null
  setSelectedNode: (id: number | null) => void
  armed: Armed
  setArmed: (a: Armed) => void
}

const UiStateContext = createContext<UiState | null>(null)

/** Interaction state shared by the node list and the graph (selection, pending connection). */
export function UiStateProvider({
  graph,
  children
}: {
  graph: GraphSnapshot
  children: ReactNode
}) {
  const [selectedNode, setSelectedNode] = useState<number | null>(null)
  const [armed, setArmed] = useState<Armed>(null)

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setArmed(null)
        setSelectedNode(null)
      }
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [])

  // Drop selections pointing to objects that disappeared.
  useEffect(() => {
    if (armed?.kind === "port" && !graph.ports.some(p => p.id === armed.port.id)) setArmed(null)
    if (armed?.kind === "node" && !graph.nodes.some(n => n.id === armed.nodeId)) setArmed(null)
    if (selectedNode !== null && !graph.nodes.some(n => n.id === selectedNode))
      setSelectedNode(null)
  }, [graph, armed, selectedNode])

  return (
    <UiStateContext.Provider value={{ selectedNode, setSelectedNode, armed, setArmed }}>
      {children}
    </UiStateContext.Provider>
  )
}

export function useUiState(): UiState {
  const ctx = useContext(UiStateContext)
  if (!ctx) throw new Error("useUiState must be used inside <UiStateProvider>")
  return ctx
}
