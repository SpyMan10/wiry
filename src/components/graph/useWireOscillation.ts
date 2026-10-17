import { RefObject, useEffect } from "react";
import { peakToUnit, subscribeLevels } from "../meter/levels";
import { WireCurve, wavyWirePath, wirePath } from "./wireGeometry";

/** Phase speed of the travelling wave (radians per second). */
const SPEED = 12.6;
/** Keep oscillating this long after the last sound, so short gaps don't stop the animation. */
const HOLD_MS = 600;

const reducedMotion = () => window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/**
 * Makes a wire oscillate while audio flows through it. Purely visual: the amplitude is always
 * the same; the level of `levelNode` (the wire's source) only tells whether something is playing
 * (the sonographs show the actual level). Animates the path directly at display rate, outside
 * React, and goes back to the plain curve once silent for HOLD_MS.
 */
export function useWireOscillation(pathRef: RefObject<SVGPathElement | null>, curve: WireCurve, levelNode: number) {
  const { x1, y1, x2, y2 } = curve;

  useEffect(() => {
    const path = pathRef.current;
    if (!path || reducedMotion()) return;
    const c = { x1, y1, x2, y2 };
    let lastSound = -Infinity;
    let phase = 0;
    let last = 0;
    let frame: number | null = null;

    const tick = (now: number) => {
      if (now - lastSound > HOLD_MS) {
        path.setAttribute("d", wirePath(c));
        frame = null;
        last = 0;
        return;
      }
      phase += (SPEED * (last ? Math.min(now - last, 100) : 16)) / 1000;
      last = now;
      path.setAttribute("d", wavyWirePath(c, 1, phase));
      frame = requestAnimationFrame(tick);
    };

    const unsubscribe = subscribeLevels((levels) => {
      // Anything above the meters' floor (-60 dB) counts as flowing.
      if (peakToUnit(levels[levelNode] ?? 0) === 0) return;
      lastSound = performance.now();
      if (frame === null) frame = requestAnimationFrame(tick);
    });

    return () => {
      unsubscribe();
      if (frame !== null) cancelAnimationFrame(frame);
      path.setAttribute("d", wirePath(c));
    };
  }, [pathRef, x1, y1, x2, y2, levelNode]);
}
