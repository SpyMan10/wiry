import { t } from "../../i18n";
import { EmptyState } from "../ui/EmptyState";
import { CellContent } from "./CellContent";

/** Key/value table of PipeWire properties, filtered on keys and values. */
export function PropertyTable({ props, filter }: { props: Record<string, string>; filter: string }) {
  const q = filter.trim().toLowerCase();
  const rows = Object.entries(props)
    .filter(([k, v]) => !q || k.toLowerCase().includes(q) || v.toLowerCase().includes(q))
    .sort(([a], [b]) => a.localeCompare(b));

  if (!rows.length) return <EmptyState>{t("properties.noMatch")}</EmptyState>;
  return (
    <table className="props-table">
      <tbody>
        {rows.map(([key, value]) => (
          <tr key={key}>
            <th scope="row">
              <CellContent>{key}</CellContent>
            </th>
            <td>
              <CellContent>{value}</CellContent>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
