export type Phase =
  "combat" | "reward" | "intermission" | "rest" | "victory" | "defeat";
export interface Character {
  id: string;
  name: string;
  title: string;
  description: string;
  relic: "haste" | "whirlwind" | "self_defense";
}
export interface Piece {
  id: number;
  kind: number;
  name: string;
  owner: number;
  cell: number;
  square: string;
  royal: boolean;
  ward: boolean;
}
export interface CardDefinition {
  kind: string;
  name: string;
  cost: number;
  description: string;
  targeted: boolean;
  exhaust: boolean;
  retain: boolean;
  innate: boolean;
  ethereal: boolean;
}
export interface Card {
  id: number;
  definition: CardDefinition;
  targets: number[];
  disabled_reason: string | null;
}
export interface Snapshot {
  phase: Phase;
  node: number;
  encounter: string;
  character: string;
  turn: number;
  round: number;
  actions_left: number;
  mana: number;
  pieces: Piece[];
  moves: { from: number; to: number }[];
  hand: Card[];
  draw_count: number;
  discard_count: number;
  exhausted_count: number;
  deck_count: number;
  relics: { name: string; description: string }[];
  log: string[];
  rewards: CardDefinition[];
  rest_options: {
    id: number;
    name: string;
    square: string;
    can_upgrade: boolean;
    can_revive: boolean;
    dead: boolean;
  }[];
  royal_name: string;
  seed: number;
}
export type Command =
  | { type: "move"; from: number; to: number }
  | { type: "play_card"; id: number; target?: number }
  | { type: "end_turn" | "enemy_step" | "continue" | "rest_skip" }
  | { type: "choose_reward"; index: number }
  | { type: "rest_upgrade" | "rest_revive"; id: number };
