import { useState } from "react";
import { createPortal } from "react-dom";
import { t } from "../../i18n";
import { GraphSnapshot } from "../../pipewire";
import { useNodeProps } from "../../hooks/useNodeProps";
import { useUiState } from "../../state";
import { EmptyState } from "../ui/EmptyState";
import { SearchField } from "../ui/SearchField";
import { PropertyTable } from "./PropertyTable";

/** Every PipeWire property of the selected node, with a filter in the panel header. */
export function PropertiesTab({ graph, toolsSlot }: { graph: GraphSnapshot; toolsSlot: HTMLElement | null }) {
  const { selectedNode } = useUiState();
  const [filter, setFilter] = useState("");
  const props = useNodeProps(selectedNode, graph);
  const node = graph.nodes.find((n) => n.id === selectedNode);

  return (
    <>
      {toolsSlot &&
        node &&
        createPortal(
          <>
            <span className="props-node" title={node.name}>
              {node.description} · #{node.id}
              {props && <span className="count"> · {Object.keys(props).length}</span>}
            </span>
            <div className="header-field">
              <SearchField value={filter} onChange={setFilter} placeholder={t("properties.filter")} />
            </div>
          </>,
          toolsSlot,
        )}
      <div className="scroll bottom-scroll">
        {!node ? (
          <EmptyState>{t("properties.empty")}</EmptyState>
        ) : (
          props && <PropertyTable props={props} filter={filter} />
        )}
      </div>
    </>
  );
}
