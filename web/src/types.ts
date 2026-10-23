export type Phase =
  "map" | "combat" | "reward" | "treasure" | "rest" | "victory" | "defeat";
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
  key: string;
  description: string;
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
  map: {
    rows: number;
    lanes: number;
    current: number | null;
    path: number[];
    nodes: MapNode[];
  };
  node_kind: NodeKind | null;
  node_reward: { name: string; description: string } | null;
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
  rest_options: RestOption[];
  upgrade_paths: UpgradePath[];
  royal_name: string;
  seed: number;
}
export type Command =
  | { type: "select_node"; id: number }
  | { type: "move"; from: number; to: number }
  | { type: "play_card"; id: number; target?: number }
  | { type: "end_turn" | "enemy_step" | "claim_treasure" | "rest_skip" }
  | { type: "choose_reward"; index: number }
  | { type: "rest_upgrade" | "rest_revive"; id: number };

export type NodeKind = "combat" | "elite" | "rest" | "treasure" | "boss";
export interface MapNode {
  id: number;
  row: number;
  lane: number;
  kind: NodeKind;
  name: string;
  description: string;
  next: number[];
  available: boolean;
  visited: boolean;
  current: boolean;
  skipped: boolean;
}

export interface RestOption {
  id: number;
  kind: number;
  key: string;
  name: string;
  description: string;
  cell: number | null;
  origin: number | null;
  square: string;
  upgrades: string[];
  ward: boolean;
  can_upgrade: boolean;
  can_revive: boolean;
  dead: boolean;
}
export interface UpgradePath {
  from: number;
  from_key: string;
  from_name: string;
  to: number;
  to_key: string;
  to_name: string;
  effect: string;
  requirement: string;
}
