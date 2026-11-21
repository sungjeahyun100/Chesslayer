import type { CompendiumSquad } from "../Compendium";

export const SIMULATOR_SETTINGS_KEY = "chesslayer.simulator.settings.v1";

export type SimulatorSettings = {
  squad: string;
  encounter: number;
  deck: string[];
  relics: string[];
  upgrades: Record<number, number>;
};

type Squad = { id: string; deck: string[]; relics: string[]; pieces: { id: number; kind: number }[] };
type Content = { squads: Squad[]; encounters: { index: number }[] };

function reachableKinds(entry: CompendiumSquad, root: number): Set<number> {
  const reachable = new Set([root]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const piece of entry.pieces) for (const path of piece.upgrades) {
      if (reachable.has(path.from) && !reachable.has(path.to)) {
        reachable.add(path.to);
        changed = true;
      }
    }
  }
  return reachable;
}

export function restoreSettings(content: Content, entries: CompendiumSquad[]): SimulatorSettings | null {
  const first = content.squads[0];
  if (!first) return null;
  let saved: Record<string, unknown> = {};
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(SIMULATOR_SETTINGS_KEY) ?? "null");
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) saved = parsed as Record<string, unknown>;
  } catch { /* Malformed or unavailable browser storage: use current content defaults. */ }
  const savedSquad = content.squads.find((item) => item.id === saved.squad);
  const squad = savedSquad ?? first;
  const entry = entries.find((item) => item.id === squad.id);
  const allowedCards = new Set(entry?.cards.filter((card) => card.available).map((card) => card.definition.kind));
  const allowedRelics = new Set(entry?.relics.map((relic) => relic.id));
  const deck = savedSquad && Array.isArray(saved.deck)
    ? saved.deck.filter((kind): kind is string => typeof kind === "string" && allowedCards.has(kind))
    : [...squad.deck];
  const relics = savedSquad && Array.isArray(saved.relics)
    ? [...new Set(saved.relics.filter((id): id is string => typeof id === "string" && allowedRelics.has(id)))]
    : [...squad.relics];
  const upgrades: Record<number, number> = {};
  if (savedSquad && saved.upgrades && typeof saved.upgrades === "object" && !Array.isArray(saved.upgrades) && entry) {
    for (const piece of squad.pieces) {
      const target = (saved.upgrades as Record<string, unknown>)[piece.id];
      if (typeof target === "number" && Number.isInteger(target) && reachableKinds(entry, piece.kind).has(target))
        upgrades[piece.id] = target;
    }
  }
  return {
    squad: squad.id,
    encounter: content.encounters.some((item) => item.index === saved.encounter)
      ? saved.encounter as number : content.encounters[0]?.index ?? 0,
    deck, relics, upgrades,
  };
}

export function saveSettings(settings: SimulatorSettings): void {
  try { localStorage.setItem(SIMULATOR_SETTINGS_KEY, JSON.stringify(settings)); }
  catch { /* The simulator remains usable when storage is unavailable. */ }
}
