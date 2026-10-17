import { useEffect } from "react";

/**
 * Disables the webview's right-click menu (Reload, Back, Inspect…), which makes no sense in a
 * desktop app. Kept when `enabled` is false, e.g. in debug mode to reach the inspector.
 */
export function useNoContextMenu(enabled: boolean) {
  useEffect(() => {
    if (!enabled) return;
    const block = (e: MouseEvent) => e.preventDefault();
    window.addEventListener("contextmenu", block);
    return () => window.removeEventListener("contextmenu", block);
  }, [enabled]);
}
