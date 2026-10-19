import { AlsaCard } from "../../alsa"
import { t } from "../../i18n"
import { Icon } from "../ui/Icon"
import { Select } from "../ui/Select"

export function AlsaCardPicker({
  cards,
  value,
  onChange
}: {
  cards: AlsaCard[]
  value: number
  onChange: (index: number) => void
}) {
  return (
    <label className="alsa-card-picker" title={t("alsa.card")}>
      <Icon name="cpu" />
      <Select
        value={value}
        onChange={onChange}
        options={cards.map(c => ({ value: c.index, label: `${c.name} (hw:${c.index})` }))}
      />
    </label>
  )
}
