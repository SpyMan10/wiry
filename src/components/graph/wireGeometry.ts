/** Cubic bezier of a wire, leaving and entering its ports horizontally. */
export interface WireCurve {
  x1: number
  y1: number
  x2: number
  y2: number
}

const handle = ({ x1, x2 }: WireCurve) => Math.max(40, (x2 - x1) * 0.5)

export function wirePath(c: WireCurve) {
  const dx = handle(c)
  return `M${c.x1},${c.y1} C${c.x1 + dx},${c.y1} ${c.x2 - dx},${c.y2} ${c.x2},${c.y2}`
}

/** Sideways displacement (px) of the wave. */
const AMPLITUDE = 4
/** Wavelength (px) of the oscillation, measured along the wire. */
const WAVELENGTH = 20
/** Distance (px) between two points of the drawn wave. */
const STEP_PX = 2
/** Length (px) over which the wave fades in/out at each end, so the wire stays on its ports. */
const TAPER_PX = WAVELENGTH
/** Resolution of the arc-length table used to walk the bezier at constant speed. */
const ARC_SAMPLES = 256

type Point = [number, number]

function bezier(c: WireCurve) {
  const dx = handle(c)
  const p: Point[] = [
    [c.x1, c.y1],
    [c.x1 + dx, c.y1],
    [c.x2 - dx, c.y2],
    [c.x2, c.y2]
  ]
  const at = (t: number): Point => {
    const u = 1 - t
    const w = [u * u * u, 3 * u * u * t, 3 * u * t * t, t * t * t]
    return [
      w[0] * p[0][0] + w[1] * p[1][0] + w[2] * p[2][0] + w[3] * p[3][0],
      w[0] * p[0][1] + w[1] * p[1][1] + w[2] * p[2][1] + w[3] * p[3][1]
    ]
  }
  const tangent = (t: number): Point => {
    const u = 1 - t
    const k = (i: number) =>
      3 * u * u * (p[1][i] - p[0][i]) +
      6 * u * t * (p[2][i] - p[1][i]) +
      3 * t * t * (p[3][i] - p[2][i])
    return [k(0), k(1)]
  }
  return { at, tangent }
}

const smoothstep = (x: number) => (x <= 0 ? 0 : x >= 1 ? 1 : x * x * (3 - 2 * x))

/**
 * Precomputes a wire's geometry for animation and returns a function drawing it with a
 * travelling sine wave: `phase` (radians) makes the wave flow from source to destination,
 * `amplitude` (0..1) scales it.
 *
 * The wave follows the arc length, so it is a true sinusoid with a constant wavelength and a
 * constant amplitude all along the wire; only the first/last TAPER_PX blend into the ports.
 */
export function wireWave(c: WireCurve) {
  const { at, tangent } = bezier(c)

  // Cumulative arc length, to map a distance along the wire back to the curve parameter.
  const ts: number[] = []
  const lengths: number[] = []
  let total = 0
  let prev = at(0)
  for (let i = 0; i <= ARC_SAMPLES; i++) {
    const t = i / ARC_SAMPLES
    const pt = at(t)
    total += Math.hypot(pt[0] - prev[0], pt[1] - prev[1])
    ts.push(t)
    lengths.push(total)
    prev = pt
  }
  const tAt = (s: number) => {
    let lo = 0
    let hi = lengths.length - 1
    while (hi - lo > 1) {
      const mid = (lo + hi) >> 1
      if (lengths[mid] < s) lo = mid
      else hi = mid
    }
    const span = lengths[hi] - lengths[lo] || 1
    return ts[lo] + ((s - lengths[lo]) / span) * (ts[hi] - ts[lo])
  }

  // Evenly spaced points along the wire, with their unit normal and end taper.
  const count = Math.max(2, Math.ceil(total / STEP_PX))
  const points = Array.from({ length: count + 1 }, (_, i) => {
    const s = (i / count) * total
    const t = tAt(s)
    const [x, y] = at(t)
    const [tx, ty] = tangent(t)
    const n = Math.hypot(tx, ty) || 1
    const taper = smoothstep(Math.min(s, total - s) / TAPER_PX)
    return { s, x, y, nx: -ty / n, ny: tx / n, taper }
  })

  const k = (2 * Math.PI) / WAVELENGTH
  return (amplitude: number, phase: number) => {
    let d = ""
    for (let i = 0; i < points.length; i++) {
      const p = points[i]
      const offset = AMPLITUDE * amplitude * p.taper * Math.sin(k * p.s - phase)
      d += `${i ? "L" : "M"}${(p.x + p.nx * offset).toFixed(2)},${(p.y + p.ny * offset).toFixed(2)}`
    }
    return d
  }
}
