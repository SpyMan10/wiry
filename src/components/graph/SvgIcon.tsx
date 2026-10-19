import biCodepoints from "bootstrap-icons/font/bootstrap-icons.json"

/** Bootstrap Icons glyph for SVG, where the `.bi-*` CSS classes can't be used. */
export function SvgIcon({
  name,
  x,
  y,
  title,
  className
}: {
  name: string
  x: number
  y: number
  title?: string
  className?: string
}) {
  const code = (biCodepoints as Record<string, number>)[name] ?? 0x3f
  return (
    <text className={className} x={x} y={y} textAnchor="end">
      {title && <title>{title}</title>}
      {String.fromCodePoint(code)}
    </text>
  )
}
