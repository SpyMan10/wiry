/** Cubic bezier of a wire, leaving and entering its ports horizontally. */
export interface WireCurve {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

const handle = ({ x1, x2 }: WireCurve) => Math.max(40, (x2 - x1) * 0.5);

export function wirePath(c: WireCurve) {
  const dx = handle(c);
  return `M${c.x1},${c.y1} C${c.x1 + dx},${c.y1} ${c.x2 - dx},${c.y2} ${c.x2},${c.y2}`;
}

const SAMPLES = 48;
/** Sideways displacement (px) of the wave. */
const AMPLITUDE = 4;
/** Wavelength (px) of the oscillation along the wire. */
const WAVELENGTH = 20;

/**
 * The wire with a travelling wave along it: `amplitude` (0..1) scales the displacement,
 * `phase` (radians) makes it flow from source to destination. Ends stay pinned to the ports.
 */
export function wavyWirePath(c: WireCurve, amplitude: number, phase: number) {
  const dx = handle(c);
  const p0 = [c.x1, c.y1];
  const p1 = [c.x1 + dx, c.y1];
  const p2 = [c.x2 - dx, c.y2];
  const p3 = [c.x2, c.y2];
  const length = Math.hypot(c.x2 - c.x1, c.y2 - c.y1) + dx; // rough arc length, enough for the wavelength
  const cycles = Math.max(1, length / WAVELENGTH);

  let d = "";
  for (let i = 0; i <= SAMPLES; i++) {
    const t = i / SAMPLES;
    const u = 1 - t;
    // Point and tangent of the cubic bezier.
    const b = (k: number) => u * u * u * p0[k] + 3 * u * u * t * p1[k] + 3 * u * t * t * p2[k] + t * t * t * p3[k];
    const db = (k: number) => 3 * u * u * (p1[k] - p0[k]) + 6 * u * t * (p2[k] - p1[k]) + 3 * t * t * (p3[k] - p2[k]);
    const tx = db(0);
    const ty = db(1);
    const norm = Math.hypot(tx, ty) || 1;
    // Envelope keeps both ends on their ports.
    const offset = AMPLITUDE * amplitude * Math.sin(Math.PI * t) * Math.sin(2 * Math.PI * cycles * t - phase);
    const x = b(0) - (ty / norm) * offset;
    const y = b(1) + (tx / norm) * offset;
    d += `${i ? "L" : "M"}${x.toFixed(1)},${y.toFixed(1)}`;
  }
  return d;
}
