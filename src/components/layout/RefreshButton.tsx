import { useEffect, useRef, useState } from "react"
import { t } from "../../i18n"
import { GraphSnapshot, pw } from "../../pipewire"
import { Button } from "../ui/Button"
import { cx } from "../ui/cx"

/** Gives up waiting for the new graph after this long (the icon stops spinning). */
const TIMEOUT_MS = 4000

/**
 * Forces a full resync with PipeWire (reconnect + re-read of the registry), for the rare cases
 * where the graph or the node list got out of date. Spins until the rebuilt graph arrives.
 */
export function RefreshButton({ graph }: { graph: GraphSnapshot }) {
  const [busy, setBusy] = useState(false)
  const before = useRef<GraphSnapshot | null>(null)

  // Any snapshot received after the request means the graph was rebuilt.
  useEffect(() => {
    if (busy && graph !== before.current && graph.connected) setBusy(false)
  }, [graph, busy])

  useEffect(() => {
    if (!busy) return
    const timer = window.setTimeout(() => setBusy(false), TIMEOUT_MS)
    return () => window.clearTimeout(timer)
  }, [busy])

  return (
    <Button
      className={cx("refresh-btn", busy && "busy")}
      icon="arrow-clockwise"
      title={t("graph.refresh")}
      disabled={busy}
      onClick={() => {
        before.current = graph
        setBusy(true)
        pw.resync().catch(() => setBusy(false))
      }}
    />
  )
}
