import { t } from "../../i18n"
import { Button } from "./Button"
import { Icon } from "./Icon"

/** Borderless search input with a clear button; Escape clears it. */
export function SearchField({
  value,
  onChange,
  placeholder
}: {
  value: string
  onChange: (v: string) => void
  placeholder: string
}) {
  return (
    <label className="search-box">
      <Icon name="search" />
      <input
        className="search"
        placeholder={placeholder}
        value={value}
        onChange={e => onChange(e.currentTarget.value)}
        onKeyDown={e => e.key === "Escape" && onChange("")}
      />
      {value && (
        <Button
          className="search-clear"
          icon="x-circle-fill"
          title={t("nodeList.clear")}
          onClick={() => onChange("")}
        />
      )}
    </label>
  )
}
