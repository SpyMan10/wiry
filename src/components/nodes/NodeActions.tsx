import { t } from "../../i18n";
import { PwPort } from "../../pipewire";
import { Button } from "../ui/Button";
import { Patching } from "./usePatching";

/** "Link" (arm this node's outputs) or, while another node is armed, "Connect here". */
export function NodeActions({
  nodeId,
  ins,
  outs,
  patch,
}: {
  nodeId: number;
  ins: PwPort[];
  outs: PwPort[];
  patch: Patching;
}) {
  const isTarget = patch.armedNode !== null && patch.armedNode !== nodeId && ins.length > 0;

  return (
    <div className="node-actions">
      {isTarget ? (
        <Button
          accent
          icon="plug-fill"
          onClick={() => {
            patch.connectArmedNodeTo(ins);
            patch.cancel();
          }}
        >
          {t("nodeList.connectHere")}
        </Button>
      ) : (
        outs.length > 0 && (
          <Button
            accent={patch.armedNode === nodeId}
            icon="link-45deg"
            title={t("nodeList.linkTitle")}
            onClick={() => patch.toggleArmNode(nodeId)}
          >
            {t("nodeList.link")}
          </Button>
        )
      )}
    </div>
  );
}
