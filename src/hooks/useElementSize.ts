import { RefObject, useLayoutEffect, useState } from "react"

/** Content-box size of an element, kept up to date with a ResizeObserver. */
export function useElementSize(ref: RefObject<Element | null>, initial = { width: 0, height: 0 }) {
  const [size, setSize] = useState(initial)
  useLayoutEffect(() => {
    const el = ref.current
    if (!el) return
    const ro = new ResizeObserver(([e]) =>
      setSize({ width: Math.floor(e.contentRect.width), height: Math.floor(e.contentRect.height) })
    )
    ro.observe(el)
    return () => ro.disconnect()
  }, [ref])
  return size
}
