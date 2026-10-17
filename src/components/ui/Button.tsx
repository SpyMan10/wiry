import { ButtonHTMLAttributes } from "react";
import { Icon } from "./Icon";
import { cx } from "./cx";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Leading Bootstrap Icons name. */
  icon?: string;
  accent?: boolean;
}

export function Button({ icon, accent, className, children, type = "button", ...rest }: Props) {
  return (
    // Icon-only buttons (close, mute, fold…) are square.
    <button type={type} className={cx(accent && "btn-accent", icon && !children && "btn-icon", className)} {...rest}>
      {icon && <Icon name={icon} />}
      {icon && children ? " " : null}
      {children}
    </button>
  );
}
