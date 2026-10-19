import { AlsaControl, alsa, booleanEnum, controlLabel } from "../../alsa"
import { t } from "../../i18n"
import { useDraft } from "../../hooks/useDraft"
import { Button } from "../ui/Button"
import { Select } from "../ui/Select"
import { cx } from "../ui/cx"
import { MuteButton } from "../volume/MuteButton"
import { Percent } from "../volume/Percent"
import { VolumeSlider } from "../volume/VolumeSlider"

/** One mixer control, laid out on the section grid: label · switch · value · % · dB. */
export function AlsaControlRow({ card, control }: { card: number; control: AlsaControl }) {
  const label = controlLabel(control)
  return (
    <div className="alsa-row">
      <span className="alsa-name" title={label}>
        {label}
      </span>
      {control.enumerated ? (
        booleanEnum(control) ? (
          <BooleanEnumValue card={card} control={control} />
        ) : (
          <EnumValue card={card} control={control} />
        )
      ) : control.volume ? (
        <VolumeValue card={card} control={control} />
      ) : (
        <SwitchValue card={card} control={control} />
      )}
    </div>
  )
}

function VolumeValue({ card, control }: { card: number; control: AlsaControl }) {
  const vol = control.volume!
  const level = vol.channels.length ? Math.max(...vol.channels) : 0
  const draft = useDraft(level, (v: number) => alsa.setVolume(card, control, v))
  return (
    <>
      {control.switch !== null ? (
        <MuteButton
          muted={!control.switch}
          onToggle={() => alsa.setSwitch(card, control, !control.switch)}
        />
      ) : (
        <span />
      )}
      <div className={cx("alsa-slider", control.switch === false && "muted")}>
        {/* No middle-click reset: never jump a hardware gain to 100 % by accident. */}
        <VolumeSlider
          value={draft.value}
          onChange={draft.set}
          onEnd={draft.end}
          max={1}
          resetValue={null}
        />
      </div>
      <Percent
        value={draft.value}
        max={1}
        onCommit={v => {
          draft.set(v)
          draft.end()
        }}
      />
      <span className="alsa-db">{vol.db !== null ? `${vol.db.toFixed(1)} dB` : ""}</span>
    </>
  )
}

function SwitchValue({ card, control }: { card: number; control: AlsaControl }) {
  const on = !!control.switch
  return (
    <Toggle
      on={on}
      label={on ? t("alsa.switchOn") : t("alsa.switchOff")}
      onToggle={() => alsa.setSwitch(card, control, !on)}
    />
  )
}

/** A two-item enum ("Disabled"/"Enabled") shown as a switch, labelled with the driver's item. */
function BooleanEnumValue({ card, control }: { card: number; control: AlsaControl }) {
  const { off, on: onItem } = booleanEnum(control)!
  const e = control.enumerated!
  const on = e.current === onItem
  return (
    <Toggle
      on={on}
      label={e.items[e.current]}
      onToggle={() => alsa.setEnum(card, control, on ? off : onItem)}
    />
  )
}

function Toggle({ on, label, onToggle }: { on: boolean; label: string; onToggle: () => void }) {
  return (
    <>
      <Button
        className={cx("alsa-switch", on && "on")}
        icon={on ? "toggle-on" : "toggle-off"}
        title={label}
        aria-pressed={on}
        onClick={onToggle}
      />
      <span className="alsa-switch-label">{label}</span>
    </>
  )
}

function EnumValue({ card, control }: { card: number; control: AlsaControl }) {
  const e = control.enumerated!
  return (
    <>
      <span />
      <div className="alsa-enum">
        <Select
          value={e.current}
          onChange={item => alsa.setEnum(card, control, item)}
          options={e.items.map((label, i) => ({ value: i, label }))}
        />
      </div>
    </>
  )
}
