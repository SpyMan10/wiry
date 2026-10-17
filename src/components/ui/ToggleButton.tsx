import { ReactNode } from "react";
import { Button } from "./Button";
import { cx } from "./cx";

/** Toolbar on/off button. */
export function ToggleButton({
  pressed,
  onToggle,
  icon,
  title,
  children,
}: {
  pressed: boolean;
  onToggle: (pressed: boolean) => void;
  icon: string;
  title?: string;
  children: ReactNode;
}) {
  return (
    <Button
      className={cx("toolbar-toggle", pressed && "on")}
      icon={icon}
      title={title}
      aria-pressed={pressed}
      onClick={() => onToggle(!pressed)}
    >
      {children}
    </Button>
  );
}
