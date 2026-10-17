import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type MixerDirection = "playback" | "capture";

export interface AlsaVolume {
  /** Per-channel volume on alsamixer's mapped scale (0..1). */
  channels: number[];
  channelNames: string[];
  db: number | null;
}

/** One direction of a native ALSA mixer element (what alsamixer shows). */
export interface AlsaControl {
  name: string;
  index: number;
  direction: MixerDirection;
  volume: AlsaVolume | null;
  /** On/off switch (`true` = on). */
  switch: boolean | null;
  enumerated: { items: string[]; current: number } | null;
}

export interface AlsaCard {
  index: number;
  name: string;
  longName: string;
  controls: AlsaControl[];
}

const id = (card: number, c: AlsaControl) => ({ card, name: c.name, index: c.index, direction: c.direction });

export const alsa = {
  setVolume: (card: number, c: AlsaControl, value: number) =>
    invoke<void>("alsa_set_volume", { control: id(card, c), value }),
  setSwitch: (card: number, c: AlsaControl, on: boolean) => invoke<void>("alsa_set_switch", { control: id(card, c), on }),
  setEnum: (card: number, c: AlsaControl, item: number) => invoke<void>("alsa_set_enum", { control: id(card, c), item }),
};

const OFF = /^(disabled?|off|no|false)$/i;
const ON = /^(enabled?|on|yes|true)$/i;

/**
 * For enumerated controls that are really booleans (two items such as "Disabled"/"Enabled"):
 * the indices of the off and on items, in whatever order the driver lists them.
 */
export function booleanEnum(c: AlsaControl): { off: number; on: number } | null {
  const items = c.enumerated?.items;
  if (items?.length !== 2) return null;
  const off = items.findIndex((i) => OFF.test(i.trim()));
  const on = items.findIndex((i) => ON.test(i.trim()));
  return off >= 0 && on >= 0 && off !== on ? { off, on } : null;
}

export const controlLabel = (c: AlsaControl) => (c.index > 0 ? `${c.name} ${c.index}` : c.name);

/** Live state of every sound card's mixer (`alsa-mixers` event). */
export function useAlsaCards() {
  const [cards, setCards] = useState<AlsaCard[] | null>(null);
  useEffect(() => {
    let alive = true;
    const unlisten = listen<AlsaCard[]>("alsa-mixers", (e) => setCards(e.payload));
    invoke<AlsaCard[]>("alsa_mixers").then((c) => alive && setCards(c));
    return () => {
      alive = false;
      unlisten.then((f) => f());
    };
  }, []);
  return cards;
}
