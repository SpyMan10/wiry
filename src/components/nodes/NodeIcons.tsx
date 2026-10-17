import { NodeIcon } from "../../pipewire";
import { Icon } from "../ui/Icon";

export function NodeIcons({ icons }: { icons: NodeIcon[] }) {
  return (
    <span className="node-icons">
      {icons.map(({ icon, title }) => (
        <Icon key={icon} name={icon} title={title} />
      ))}
    </span>
  );
}
