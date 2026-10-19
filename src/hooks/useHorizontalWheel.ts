import { RefObject, useEffect } from "react"

/**
 * Scrolls a horizontally scrolling element with the (vertical) mouse wheel. Horizontal wheel /
 * trackpad gestures keep working natively; nothing happens when the content fits, and the
 * wheel scrolls vertically as usual while the element also overflows vertically.
 */
export function useHorizontalWheel(ref: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const el = ref.current
    if (!el) return
    const onWheel = (e: WheelEvent) => {
      if (e.deltaY === 0 || Math.abs(e.deltaX) > Math.abs(e.deltaY)) return
      if (el.scrollWidth <= el.clientWidth || el.scrollHeight > el.clientHeight) return
      e.preventDefault()
      const lines = e.deltaMode === WheelEvent.DOM_DELTA_LINE ? 40 : 1
      el.scrollLeft += e.deltaY * lines
    }
    // Non-passive so the page doesn't also try to scroll.
    el.addEventListener("wheel", onWheel, { passive: false })
    return () => el.removeEventListener("wheel", onWheel)
  }, [ref])
}
