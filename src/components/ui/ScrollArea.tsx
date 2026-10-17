import { ReactNode, useLayoutEffect, useRef, useState } from "react";
import { cx } from "./cx";

/**
 * Vertical scroller with fading shadows on the edges that have hidden content: top once
 * scrolled down, bottom while there is more below.
 */
export function ScrollArea({ children }: { children: ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  const [edges, setEdges] = useState({ top: false, bottom: false });

  const update = () => {
    const el = ref.current;
    if (!el) return;
    const top = el.scrollTop > 0;
    const bottom = el.scrollTop + el.clientHeight < el.scrollHeight - 1;
    setEdges((prev) => (prev.top === top && prev.bottom === bottom ? prev : { top, bottom }));
  };

  // Content and size changes (new nodes, folded groups, resized window) move the bottom edge too.
  useLayoutEffect(update);
  useLayoutEffect(() => {
    const ro = new ResizeObserver(update);
    ro.observe(ref.current!);
    return () => ro.disconnect();
  }, []);

  return (
    <div className="scroll-area">
      <div className={cx("scroll-shadow top", edges.top && "visible")} />
      <div className="scroll" ref={ref} onScroll={update}>
        {children}
      </div>
      <div className={cx("scroll-shadow bottom", edges.bottom && "visible")} />
    </div>
  );
}
