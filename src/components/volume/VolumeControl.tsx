import { useState } from "react";
import { t } from "../../i18n";
import { PwNode, linearToUi, pw, uiToLinear } from "../../pipewire";
import { useDraft } from "../../hooks/useDraft";
import { cx } from "../ui/cx";
import { ChannelsToggle } from "./ChannelsToggle";
import { MuteButton } from "./MuteButton";
import { Percent } from "./Percent";
import { VolumeSlider } from "./VolumeSlider";

/** Mute + master volume slider, with optional per-channel sliders, for a node that has a volume. */
export function VolumeControl({ node }: { node: PwNode }) {
  const vol = node.volume!;
  const [expanded, setExpanded] = useState(false);
  const draft = useDraft(vol.channels, (c: number[]) => pw.setVolume(node.id, c));
  const channels = draft.value;

  const master = channels.length ? Math.max(...channels.map(linearToUi)) : 0;

  const setMaster = (ui: number) => {
    const target = uiToLinear(ui);
    const maxLin = Math.max(...channels, 0);
    // Keep the balance between channels when they differ.
    draft.set(maxLin > 0 ? channels.map((c) => (c / maxLin) * target) : channels.map(() => target));
  };

  const setChannel = (idx: number, ui: number) => draft.set(channels.map((c, i) => (i === idx ? uiToLinear(ui) : c)));

  return (
    <div className={cx("vol", vol.mute && "muted")}>
      <div className="vol-row">
        <MuteButton muted={vol.mute} onToggle={() => pw.setMute(node.id, !vol.mute)} />
        <VolumeSlider value={master} onChange={setMaster} onEnd={draft.end} />
        <Percent
          value={master}
          onCommit={(v) => {
            setMaster(v);
            draft.end();
          }}
        />
        <ChannelsToggle visible={channels.length > 1} expanded={expanded} onToggle={() => setExpanded(!expanded)} />
      </div>
      {expanded &&
        channels.map((c, i) => (
          <div key={i} className="vol-row channel">
            <span className="ch-name">{vol.channelMap[i] ?? t("volume.channelFallback", { n: i + 1 })}</span>
            <VolumeSlider value={linearToUi(c)} onChange={(v) => setChannel(i, v)} onEnd={draft.end} />
            <Percent
              value={linearToUi(c)}
              small
              onCommit={(v) => {
                setChannel(i, v);
                draft.end();
              }}
            />
            <ChannelsToggle visible={false} expanded={false} onToggle={() => {}} />
          </div>
        ))}
    </div>
  );
}
