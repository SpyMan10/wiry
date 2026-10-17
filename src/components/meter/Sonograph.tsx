import { useEffect, useRef } from "react";
import { useElementSize } from "../../hooks/useElementSize";
import { subscribeLevels } from "./levels";
import { SONOGRAPH_STEP, drawSonograph, sonographColors } from "./drawSonograph";

/** Scrolling mirrored level history of a node's audio; fills its container (size set in CSS). */
export function Sonograph({ nodeId }: { nodeId: number }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { width, height } = useElementSize(canvasRef);

  useEffect(() => {
    if (width <= 0 || height <= 0) return;
    const canvas = canvasRef.current!;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    const ctx = canvas.getContext("2d")!;
    ctx.scale(dpr, dpr);

    const colors = sonographColors(canvas);
    const history = new Array<number>(Math.floor(width / SONOGRAPH_STEP)).fill(0);
    let peakHold = 0;
    const draw = () => drawSonograph(ctx, width, height, history, peakHold, colors);

    draw();
    return subscribeLevels((frame) => {
      const peak = frame[nodeId] ?? 0;
      history.shift();
      history.push(peak);
      peakHold = Math.max(peak, peakHold * 0.9);
      draw();
    });
  }, [nodeId, width, height]);

  return <canvas ref={canvasRef} className="sonograph" />;
}
