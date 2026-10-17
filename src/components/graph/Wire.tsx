import { useRef, useState } from "react";
import { t } from "../../i18n";
import { MediaType } from "../../pipewire";
import { cx } from "../ui/cx";
import { useWireOscillation } from "./useWireOscillation";
import { wirePath } from "./wireGeometry";

/**
 * Bezier link between two port positions; oscillates while audio flows from its source.
 * Hover turns it red, click disconnects.
 */
export function Wire({
  x1,
  y1,
  x2,
  y2,
  mediaType,
  state,
  levelNode,
  onDisconnect,
}: {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  mediaType: MediaType;
  /** "focus" when touching the focused node, "dim" when another node is focused. */
  state: "normal" | "focus" | "dim";
  /** Node whose audio level drives the oscillation (the wire's source). */
  levelNode: number;
  onDisconnect: () => void;
}) {
  const [hovered, setHovered] = useState(false);
  const pathRef = useRef<SVGPathElement>(null);
  const curve = { x1, y1, x2, y2 };
  const d = wirePath(curve);
  useWireOscillation(pathRef, curve, levelNode);
  return (
    <g onMouseEnter={() => setHovered(true)} onMouseLeave={() => setHovered(false)} onClick={onDisconnect}>
      <path className="wire-hit" d={d} />
      <path
        ref={pathRef}
        className={cx("wire", `mt-${mediaType}`, hovered && "hover", state !== "normal" && state)}
        d={d}
      />
      <title>{t("graph.wireTitle")}</title>
    </g>
  );
}
