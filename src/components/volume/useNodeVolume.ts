import { PwNode, linearToUi, pw, uiToLinear } from "../../pipewire"
import { useDraft } from "../../hooks/useDraft"

/**
 * Volume of a node on the UI scale (1 = 100 %): master (loudest channel) and per channel.
 * Changes are drafted locally while dragging and sent at most once per frame.
 */
export function useNodeVolume(node: PwNode) {
  const vol = node.volume!
  const draft = useDraft(vol.channels, (c: number[]) => pw.setVolume(node.id, c))
  const channels = draft.value

  const setMaster = (ui: number) => {
    const target = uiToLinear(ui)
    const maxLin = Math.max(...channels, 0)
    // Keep the balance between channels when they differ.
    draft.set(maxLin > 0 ? channels.map(c => (c / maxLin) * target) : channels.map(() => target))
  }

  return {
    master: channels.length ? Math.max(...channels.map(linearToUi)) : 0,
    channels: channels.map(linearToUi),
    channelMap: vol.channelMap,
    mute: vol.mute,
    setMaster,
    setChannel: (idx: number, ui: number) =>
      draft.set(channels.map((c, i) => (i === idx ? uiToLinear(ui) : c))),
    /** End of a gesture (drag, typed value): lets the backend's value take over again. */
    end: draft.end,
    toggleMute: () => pw.setMute(node.id, !vol.mute)
  }
}
