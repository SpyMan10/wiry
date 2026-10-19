import { CSSProperties } from "react"
import { GraphSnapshot, usePipewireGraph } from "./pipewire"
import { t } from "./i18n"
import { UiStateProvider, useUiState } from "./state"
import { usePersistentState } from "./hooks/usePersistentState"
import { ArmedHint } from "./components/layout/ArmedHint"
import { ConnectionBanner } from "./components/layout/ConnectionBanner"
import { FpsMeter } from "./components/layout/FpsMeter"
import { useDebug } from "./hooks/useDebug"
import { useNoContextMenu } from "./hooks/useNoContextMenu"
import { FoldRail } from "./components/layout/FoldRail"
import { NodeList } from "./components/nodes/NodeList"
import { PatchGraph } from "./components/graph/PatchGraph"
import { BottomPanel } from "./components/bottom/BottomPanel"
import { Splitter } from "./components/ui/Splitter"
import { cx } from "./components/ui/cx"
import "./App.css"

function App() {
  const graph = usePipewireGraph()
  const debug = useDebug()
  // WIRY_DEBUG=1 keeps the menu, for "Inspect Element".
  useNoContextMenu(!debug)
  return (
    <UiStateProvider graph={graph}>
      <div className="app">
        <ConnectionBanner graph={graph} />
        <Workspace graph={graph} />
        <BottomPanel graph={graph} />
        <ArmedHint />
        {debug && <FpsMeter />}
      </div>
    </UiStateProvider>
  )
}

const LEFT_DEFAULT = 360
const LEFT_MIN = 280

/** Two columns: foldable, resizable node list, then the patch graph. */
function Workspace({ graph }: { graph: GraphSnapshot }) {
  const { setArmed } = useUiState()
  const [folded, setFolded] = usePersistentState("wiry.folded", { left: false })
  const [leftWidth, setLeftWidth] = usePersistentState("wiry.leftWidth", LEFT_DEFAULT)

  const setLeftFolded = (left: boolean) => {
    setFolded({ ...folded, left })
    // Ports are picked in the list: a pending connection makes no sense once it is folded.
    if (left) setArmed(null)
  }

  return (
    <main
      className={cx("columns", folded.left && "fold-left")}
      style={folded.left ? undefined : ({ "--left-col": `${leftWidth}px` } as CSSProperties)}
    >
      {/* The folded panel stays mounted (hidden) so its filters and state survive. */}
      <div className={cx("side", folded.left && "folded")}>
        <FoldRail
          side="left"
          icon="hdd-network"
          label={t("panels.nodes")}
          onUnfold={() => setLeftFolded(false)}
        />
        <NodeList graph={graph} onFold={() => setLeftFolded(true)} />
        {!folded.left && (
          <Splitter
            axis="x"
            className="side-splitter"
            value={leftWidth}
            onChange={setLeftWidth}
            min={LEFT_MIN}
            max={() => window.innerWidth * 0.6}
            defaultValue={LEFT_DEFAULT}
          />
        )}
      </div>
      <PatchGraph graph={graph} />
    </main>
  )
}

export default App
