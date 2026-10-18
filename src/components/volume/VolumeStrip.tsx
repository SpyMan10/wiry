import { t } from "../../i18n";
import { PwNode, PwPort, nodeCategory, nodeIcons } from "../../pipewire";
import { NodeIcons } from "../nodes/NodeIcons";
import { cx } from "../ui/cx";
import { Fader } from "./Fader";
import { MuteButton } from "./MuteButton";
import { useNodeVolume } from "./useNodeVolume";

/**
 * A mixer strip for one node: type icons and name written vertically along the strip (click
 * selects it), then the master fader (highlighted, with its value and mute button) and one fader
 * per channel (L/R…) when there are several.
 */
export function VolumeStrip({
  node,
  ports,
  selected,
  onSelect,
}: {
  node: PwNode;
  ports: PwPort[];
  selected: boolean;
  onSelect: () => void;
}) {
  const v = useNodeVolume(node);

  return (
    <div
      data-node-id={node.id}
      className={cx("volume-strip", `cat-${nodeCategory(node)}`, selected && "selected", v.mute && "muted")}
    >
      <button type="button" className="strip-name" title={`${node.description} · ${node.name} · #${node.id}`} onClick={onSelect}>
        <NodeIcons icons={nodeIcons(node, ports)} />
        <span className="strip-name-text">{node.description}</span>
      </button>
      <div className="strip-main">
        <div className="strip-faders">
          <Fader
            value={v.master}
            onChange={v.setMaster}
            onEnd={v.end}
            footer={<MuteButton muted={v.mute} onToggle={v.toggleMute} />}
          />
          {v.channels.length > 1 &&
            v.channels.map((c, i) => (
              <Fader
                key={i}
                variant="channel"
                value={c}
                onChange={(value) => v.setChannel(i, value)}
                onEnd={v.end}
                footer={<span className="fader-label">{v.channelMap[i] ?? t("volume.channelFallback", { n: i + 1 })}</span>}
              />
            ))}
        </div>
      </div>
    </div>
  );
}
