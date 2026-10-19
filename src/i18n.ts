import en from "./locales/en.json"

/** Dotted paths to every string of the locale file, e.g. "nodeList.filter". */
type Leaves<T, P extends string = ""> = {
  [K in keyof T & string]: T[K] extends string ? `${P}${K}` : Leaves<T[K], `${P}${K}.`>
}[keyof T & string]

export type TKey = Leaves<typeof en>

/** Looks up a UI string and fills its `{placeholders}`. */
export function t(key: TKey, vars?: Record<string, string | number>): string {
  let node: unknown = en
  for (const part of key.split(".")) node = (node as Record<string, unknown>)[part]
  const text = typeof node === "string" ? node : key
  return vars
    ? text.replace(/\{(\w+)\}/g, (m, name) => (name in vars ? String(vars[name]) : m))
    : text
}
