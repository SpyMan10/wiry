import { AlsaControl } from "../../alsa";
import { Icon } from "../ui/Icon";
import { AlsaControlRow } from "./AlsaControlRow";

/** A titled group of controls (Playback / Capture). */
export function AlsaSection({
  title,
  icon,
  card,
  controls,
}: {
  title: string;
  icon: string;
  card: number;
  controls: AlsaControl[];
}) {
  if (!controls.length) return null;
  return (
    <section className="alsa-section">
      <h3>
        <Icon name={icon} /> {title}
      </h3>
      <div className="alsa-rows">
        {controls.map((c) => (
          <AlsaControlRow key={`${c.direction}-${c.name}-${c.index}`} card={card} control={c} />
        ))}
      </div>
    </section>
  );
}
