/** Native select, themed. */
export function Select<T extends string | number>({
  value,
  options,
  onChange,
  title
}: {
  value: T
  options: { value: T; label: string }[]
  onChange: (v: T) => void
  title?: string
}) {
  return (
    <select
      className="select"
      title={title}
      value={String(value)}
      onChange={e => {
        const picked = options.find(o => String(o.value) === e.currentTarget.value)
        if (picked) onChange(picked.value)
      }}
    >
      {options.map(o => (
        <option key={String(o.value)} value={String(o.value)}>
          {o.label}
        </option>
      ))}
    </select>
  )
}
