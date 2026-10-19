import { peakToUnit } from "./levels"

const BAR = 2 // px per sample
export const SONOGRAPH_STEP = 3 // bar + gap

export interface SonographColors {
  normal: string
  hot: string
  clip: string
  base: string
}

export function sonographColors(el: Element): SonographColors {
  const style = getComputedStyle(el)
  const v = (name: string, fallback: string) => style.getPropertyValue(name).trim() || fallback
  return {
    normal: v("--audio", "#6aa7ff"),
    hot: v("--video", "#ffb35c"),
    clip: v("--danger", "#ff6b6b"),
    base: v("--border", "#2f343e")
  }
}

/** Draws the mirrored level history (oldest left, newest right) plus a peak-hold tick. */
export function drawSonograph(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  history: number[],
  peakHold: number,
  colors: SonographColors
) {
  const mid = height / 2
  const barHeight = (peak: number) => peakToUnit(peak) * (height - 2)

  ctx.clearRect(0, 0, width, height)
  ctx.fillStyle = colors.base
  ctx.fillRect(0, mid - 0.5, width, 1)

  history.forEach((peak, i) => {
    const u = peakToUnit(peak)
    if (u <= 0) return
    const h = Math.max(1, barHeight(peak))
    ctx.fillStyle = peak >= 1 ? colors.clip : u > 0.85 ? colors.hot : colors.normal
    ctx.globalAlpha = 0.35 + 0.65 * (i / history.length)
    ctx.fillRect(i * SONOGRAPH_STEP, mid - h / 2, BAR, h)
  })
  ctx.globalAlpha = 1

  if (peakHold > 0) {
    ctx.fillStyle = peakHold >= 1 ? colors.clip : colors.normal
    ctx.fillRect(width - 1, mid - barHeight(peakHold) / 2, 1, barHeight(peakHold))
  }
}
