import { ReactNode } from "react"

/**
 * The actual text of a table cell. Only this inline wrapper is selectable (the app disables
 * selection elsewhere), so a selection can't spill over the cell's padding or other cells.
 */
export function CellContent({ children }: { children: ReactNode }) {
  return <span className="cell-content">{children}</span>
}
