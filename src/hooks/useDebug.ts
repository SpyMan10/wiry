import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

/** Whether the app was started with `WIRY_DEBUG=1`. */
export function useDebug() {
  const [debug, setDebug] = useState(false)
  useEffect(() => {
    invoke<boolean>("debug_enabled").then(setDebug, () => setDebug(false))
  }, [])
  return debug
}
