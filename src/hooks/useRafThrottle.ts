import { useEffect, useRef } from "react";

/** Throttles calls to one per animation frame, always sending the latest value. */
export function useRafThrottle<T>(send: (v: T) => void) {
  const pending = useRef<{ v: T } | null>(null);
  const frame = useRef<number | null>(null);
  useEffect(() => () => void (frame.current !== null && cancelAnimationFrame(frame.current)), []);
  return (v: T) => {
    pending.current = { v };
    if (frame.current !== null) return;
    frame.current = requestAnimationFrame(() => {
      frame.current = null;
      if (pending.current) send(pending.current.v);
      pending.current = null;
    });
  };
}
