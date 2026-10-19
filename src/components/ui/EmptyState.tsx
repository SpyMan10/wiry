import { ReactNode } from "react"

export function EmptyState({ children }: { children: ReactNode }) {
  return <p className="empty-msg">{children}</p>
}
