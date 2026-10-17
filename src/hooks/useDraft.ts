import { useState } from "react";
import { useRafThrottle } from "./useRafThrottle";

/**
 * Local value for a control driven by a remote one (e.g. a PipeWire volume): while the user drags,
 * the local draft wins so server echoes don't make the control jump; changes are sent at
 * most once per frame.
 */
export function useDraft<T>(remote: T, send: (v: T) => void) {
  const [draft, setDraft] = useState<T | null>(null);
  const throttled = useRafThrottle(send);
  return {
    value: draft ?? remote,
    set: (v: T) => {
      setDraft(v);
      throttled(v);
    },
    // Give the backend a moment to echo the final value before dropping the draft.
    end: () => setTimeout(() => setDraft(null), 200),
  };
}
