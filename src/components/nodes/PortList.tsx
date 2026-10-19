import { PwPort } from "../../pipewire"
import { cx } from "../ui/cx"
import { PortButton } from "./PortButton"
import { Patching } from "./usePatching"

/** Inputs column then outputs column; a single column when the node only has one direction. */
export function PortList({ ins, outs, patch }: { ins: PwPort[]; outs: PwPort[]; patch: Patching }) {
  return (
    <div className={cx("ports", !(ins.length && outs.length) && "single")}>
      {[ins, outs].map(
        (list, i) =>
          list.length > 0 && (
            <div key={i} className={cx("port-col", i ? "outs" : "ins")}>
              {list.map(p => (
                <PortButton key={p.id} port={p} patch={patch} />
              ))}
            </div>
          )
      )}
    </div>
  )
}
