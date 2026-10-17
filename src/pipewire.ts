import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { t } from "./i18n";

export type MediaType = "audio" | "midi" | "video" | "other";

export interface PwVolume {
  /** Linear per-channel volumes. */
  channels: number[];
  channelMap: string[];
  mute: boolean;
}

export interface PwNode {
  id: number;
  name: string;
  description: string;
  mediaClass: string;
  appName: string | null;
  iconName: string | null;
  volume: PwVolume | null;
  /** ALSA card backing this node (hardware nodes only). */
  alsaCard: number | null;
}

export interface PwPort {
  id: number;
  nodeId: number;
  index: number;
  name: string;
  alias: string | null;
  direction: "in" | "out";
  mediaType: MediaType;
  channel: string | null;
  monitor: boolean;
  physical: boolean;
}

export interface PwLink {
  id: number;
  outputNode: number;
  outputPort: number;
  inputNode: number;
  inputPort: number;
}

export interface GraphSnapshot {
  connected: boolean;
  error: string | null;
  nodes: PwNode[];
  ports: PwPort[];
  links: PwLink[];
}

const EMPTY: GraphSnapshot = { connected: false, error: null, nodes: [], ports: [], links: [] };

export function usePipewireGraph(): GraphSnapshot {
  const [graph, setGraph] = useState<GraphSnapshot>(EMPTY);

  useEffect(() => {
    let alive = true;
    const unlisten = listen<GraphSnapshot>("pw-graph", (e) => setGraph(e.payload));
    invoke<GraphSnapshot>("pw_graph").then((g) => alive && setGraph(g));
    return () => {
      alive = false;
      unlisten.then((f) => f());
    };
  }, []);

  return graph;
}

export const pw = {
  nodeProps: (nodeId: number) => invoke<Record<string, string> | null>("pw_node_props", { nodeId }),
  link: (outputPort: number, inputPort: number) => invoke<void>("pw_link", { outputPort, inputPort }),
  unlink: (linkId: number) => invoke<void>("pw_unlink", { linkId }),
  setVolume: (nodeId: number, channels: number[]) => invoke<void>("pw_set_volume", { nodeId, channels }),
  setMute: (nodeId: number, mute: boolean) => invoke<void>("pw_set_mute", { nodeId, mute }),
};

export type NodeCategory = "sink" | "source" | "playback" | "recording" | "midi" | "video" | "other";

export const categoryLabel = (c: NodeCategory) => t(`categories.${c}`);

/** Bootstrap Icons name for each category. */
export const CATEGORY_ICONS: Record<NodeCategory, string> = {
  sink: "speaker",
  source: "mic",
  playback: "play-circle",
  recording: "record-circle",
  midi: "music-note-beamed",
  video: "camera-video",
  other: "diagram-3",
};

export interface NodeIcon {
  /** Bootstrap Icons name (without the `bi-` prefix). */
  icon: string;
  title: string;
}

/** Icons describing what a node is, shown next to its name. */
export function nodeIcons(node: PwNode, ports: PwPort[]): NodeIcon[] {
  const c = node.mediaClass;
  const icons: NodeIcon[] = [];
  if (c.startsWith("Stream/")) icons.push({ icon: "window", title: `${t("nodeTypes.application")}${node.appName ? ` · ${node.appName}` : ""}` });
  else if (ports.some((p) => p.physical)) icons.push({ icon: "cpu", title: t("nodeTypes.hardware") });
  if (c.includes("Virtual")) icons.push({ icon: "layers", title: t("nodeTypes.virtual") });

  if (c.includes("Midi")) icons.push({ icon: "music-note-beamed", title: t("nodeTypes.midi") });
  else if (c.includes("Video")) icons.push({ icon: "camera-video", title: t("nodeTypes.video") });
  else {
    const input = c.startsWith("Audio/Source") || c.startsWith("Stream/Input") || c.startsWith("Audio/Duplex");
    const output = c.startsWith("Audio/Sink") || c.startsWith("Stream/Output") || c.startsWith("Audio/Duplex");
    if (input) icons.push({ icon: "mic", title: t("nodeTypes.input") });
    if (output) icons.push({ icon: "speaker", title: t("nodeTypes.output") });
  }
  return icons.map((i) => ({ ...i, title: `${i.title} — ${c || "?"}` }));
}

/** Nodes for which the backend runs a level meter (mirrors `meter_target` in pw.rs). */
export function hasMeter(node: PwNode) {
  return /^(Audio\/(Sink|Duplex|Source)|Stream\/Output\/Audio)/.test(node.mediaClass);
}

export const CATEGORY_ORDER: NodeCategory[] = ["playback", "recording", "sink", "source", "midi", "video", "other"];

export function nodeCategory(node: PwNode): NodeCategory {
  const c = node.mediaClass;
  if (c.startsWith("Stream/Output/Audio")) return "playback";
  if (c.startsWith("Stream/Input/Audio")) return "recording";
  if (c.startsWith("Audio/Sink") || c.startsWith("Audio/Duplex")) return "sink";
  if (c.startsWith("Audio/Source")) return "source";
  if (c.includes("Midi")) return "midi";
  if (c.includes("Video")) return "video";
  return "other";
}

/** Human label for a port: channel if known, otherwise the port name. */
export function portLabel(port: PwPort): string {
  if (port.channel && port.channel !== "UNK") {
    return port.monitor ? t("ports.monitor", { channel: port.channel }) : port.channel;
  }
  return port.name;
}

export function sortPorts(ports: PwPort[]): PwPort[] {
  return [...ports].sort((a, b) => a.index - b.index || a.id - b.id);
}

/**
 * Pairs the output ports of one node with the input ports of another
 * (same media type), matching channel names when possible.
 */
export function matchPorts(outs: PwPort[], ins: PwPort[]): [PwPort, PwPort][] {
  const pairs: [PwPort, PwPort][] = [];
  const outputs = sortPorts(outs);
  for (const type of new Set(outputs.map((p) => p.mediaType))) {
    const o = outputs.filter((p) => p.mediaType === type);
    const i = sortPorts(ins.filter((p) => p.mediaType === type));
    if (!o.length || !i.length) continue;

    if (o.length === 1) {
      i.forEach((p) => pairs.push([o[0], p]));
    } else if (i.length === 1) {
      o.forEach((p) => pairs.push([p, i[0]]));
    } else {
      const byChannel = o.every((p) => p.channel && i.some((q) => q.channel === p.channel));
      o.forEach((p, idx) => {
        const target = byChannel ? i.find((q) => q.channel === p.channel) : i[idx];
        if (target) pairs.push([p, target]);
      });
    }
  }
  return pairs;
}

// Volumes: PipeWire stores linear amplitude; UIs (wpctl, pavucontrol) show a cubic scale.
export const linearToUi = (v: number) => Math.cbrt(Math.max(0, v));
export const uiToLinear = (v: number) => Math.max(0, v) ** 3;
