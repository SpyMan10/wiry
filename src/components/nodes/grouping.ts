import { CATEGORY_ORDER, NodeCategory, PwNode, PwPort, nodeCategory } from "../../pipewire";

/** Ports per node id, optionally without monitor ports. */
export function portsByNodeOf(ports: PwPort[], withMonitors: boolean) {
  const m = new Map<number, PwPort[]>();
  for (const p of ports) {
    if (!withMonitors && p.monitor) continue;
    if (!m.has(p.nodeId)) m.set(p.nodeId, []);
    m.get(p.nodeId)!.push(p);
  }
  return m;
}

/** Connectable nodes matching `query`, grouped by category in display order and sorted by name. */
export function groupNodes(nodes: PwNode[], portsByNode: Map<number, PwPort[]>, query: string) {
  const q = query.trim().toLowerCase();
  const groups = new Map<NodeCategory, PwNode[]>();
  for (const n of nodes) {
    if (!portsByNode.get(n.id)?.length) continue;
    if (q && !`${n.description} ${n.name} ${n.appName ?? ""}`.toLowerCase().includes(q)) continue;
    const c = nodeCategory(n);
    if (!groups.has(c)) groups.set(c, []);
    groups.get(c)!.push(n);
  }
  for (const list of groups.values()) list.sort((a, b) => a.description.localeCompare(b.description));
  return CATEGORY_ORDER.filter((c) => groups.has(c)).map((c) => [c, groups.get(c)!] as const);
}
