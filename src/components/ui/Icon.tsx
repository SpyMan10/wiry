import { cx } from "./cx";

/** A Bootstrap Icons glyph; `name` is the icon name without the `bi-` prefix. */
export function Icon({ name, title, className }: { name: string; title?: string; className?: string }) {
  return <i className={cx("bi", `bi-${name}`, className)} title={title} />;
}
