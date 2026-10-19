/** PipeWire id of a node, shown discreetly next to its name. */
export function NodeId({ id }: { id: number }) {
  return <span className="node-id">#{id}</span>
}
