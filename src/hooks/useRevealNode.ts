import { RefObject, useEffect } from "react"

/**
 * Scrolls the element of `nodeId` (marked with `data-node-id`) into view inside `root`
 * whenever the selected node changes, wherever it was selected from. Waits two frames so a
 * group just unfolded to show it has been rendered.
 */
export function useRevealNode(root: RefObject<HTMLElement | null>, nodeId: number | null) {
  useEffect(() => {
    if (nodeId === null) return
    let frame = requestAnimationFrame(() => {
      frame = requestAnimationFrame(() => {
        root.current
          ?.querySelector(`[data-node-id="${nodeId}"]`)
          ?.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "smooth" })
      })
    })
    return () => cancelAnimationFrame(frame)
  }, [root, nodeId])
}
