import { useEffect, useState } from "react"
import { createPortal } from "react-dom"
import { useAlsaCards } from "../../alsa"
import { t } from "../../i18n"
import { GraphSnapshot } from "../../pipewire"
import { useUiState } from "../../state"
import { EmptyState } from "../ui/EmptyState"
import { AlsaCardPicker } from "./AlsaCardPicker"
import { AlsaSection } from "./AlsaSection"

/**
 * alsamixer-like view of one sound card: playback and capture controls side by side.
 * Follows the card of the selected node; any card can be picked in the header.
 */
export function AlsaMixerTab({
  graph,
  toolsSlot
}: {
  graph: GraphSnapshot
  toolsSlot: HTMLElement | null
}) {
  const cards = useAlsaCards()
  const { selectedNode } = useUiState()
  const nodeCard = graph.nodes.find(n => n.id === selectedNode)?.alsaCard ?? null
  const [picked, setPicked] = useState<number | null>(nodeCard)

  // Selecting a hardware node shows its card.
  useEffect(() => {
    if (nodeCard !== null) setPicked(nodeCard)
  }, [nodeCard])

  if (!cards) return null
  if (!cards.length) return <EmptyState>{t("alsa.noCards")}</EmptyState>

  const card =
    cards.find(c => c.index === picked) ??
    cards.find(c => c.controls.some(ctl => ctl.volume)) ??
    cards[0]
  const playback = card.controls.filter(c => c.direction === "playback")
  const capture = card.controls.filter(c => c.direction === "capture")

  return (
    <>
      {toolsSlot &&
        createPortal(
          <>
            <span className="props-node" title={card.longName}>
              {card.longName}
            </span>
            <AlsaCardPicker cards={cards} value={card.index} onChange={setPicked} />
          </>,
          toolsSlot
        )}
      <div className="scroll bottom-scroll alsa-scroll">
        {card.controls.length ? (
          <div className="alsa-sections">
            <AlsaSection
              title={t("alsa.playback")}
              icon="speaker"
              card={card.index}
              controls={playback}
            />
            <AlsaSection
              title={t("alsa.capture")}
              icon="mic"
              card={card.index}
              controls={capture}
            />
          </div>
        ) : (
          <EmptyState>{t("alsa.noControls")}</EmptyState>
        )}
      </div>
    </>
  )
}
