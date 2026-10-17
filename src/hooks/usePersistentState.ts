import { useCallback, useState } from "react";

/** `useState` mirrored to localStorage (JSON). Falls back silently when storage is unavailable. */
export function usePersistentState<T>(key: string, fallback: T) {
  const [value, setValue] = useState<T>(() => {
    try {
      const raw = localStorage.getItem(key);
      return raw === null ? fallback : (JSON.parse(raw) as T);
    } catch {
      return fallback;
    }
  });

  const set = useCallback(
    (next: T) => {
      setValue(next);
      try {
        localStorage.setItem(key, JSON.stringify(next));
      } catch {
        // Storage unavailable: the value just won't be remembered.
      }
    },
    [key],
  );

  return [value, set] as const;
}
