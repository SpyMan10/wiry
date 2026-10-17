import { useEffect, useRef } from "react";

const WINDOW_MS = 500;

/**
 * Frames actually rendered by the webview (requestAnimationFrame rate), with the worst frame
 * time of the last half second. Updates the DOM directly so it doesn't disturb what it measures.
 */
export function FpsMeter() {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let frame = 0;
    let frames = 0;
    let windowStart = performance.now();
    let last = windowStart;
    let worst = 0;

    const tick = (now: number) => {
      frames++;
      worst = Math.max(worst, now - last);
      last = now;
      if (now - windowStart >= WINDOW_MS && ref.current) {
        const fps = (frames * 1000) / (now - windowStart);
        ref.current.textContent = `${fps.toFixed(0)} FPS · max ${worst.toFixed(1)} ms`;
        frames = 0;
        worst = 0;
        windowStart = now;
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, []);

  return (
    <div className="fps-meter" ref={ref}>
      … FPS
    </div>
  );
}
