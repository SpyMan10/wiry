import { listen } from "@tauri-apps/api/event";

/** Peak (linear) of every metered node for one frame, keyed by node id. */
export type LevelFrame = Record<string, number>;

// One global `pw-levels` subscription shared by every meter. Each frame (~30 fps) carries the
// peak of every metered node; meters draw straight to their canvas so levels never re-render React.
const subscribers = new Set<(frame: LevelFrame) => void>();
let unlisten: Promise<() => void> | null = null;

export function subscribeLevels(cb: (frame: LevelFrame) => void) {
  subscribers.add(cb);
  unlisten ??= listen<LevelFrame>("pw-levels", (e) => subscribers.forEach((s) => s(e.payload)));
  return () => {
    subscribers.delete(cb);
    if (!subscribers.size && unlisten) {
      unlisten.then((f) => f());
      unlisten = null;
    }
  };
}

const FLOOR_DB = -60;

/** Peak (linear) -> 0..1 on a dB scale. */
export const peakToUnit = (peak: number) =>
  peak <= 0 ? 0 : Math.min(1, Math.max(0, (20 * Math.log10(peak) - FLOOR_DB) / -FLOOR_DB));
