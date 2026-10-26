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
  self_defense: boolean;
  temporary_upgrade: boolean;
  limiter: { active: boolean; moves_left: number } | null;
  absorbed_moves: string[];
}
export interface EnemyInspection {
  piece_id: number;
  cell: number;
  moves: { to: number; captures: number[]; route: number[] | null }[];
}
export interface CardDefinition {
  kind: string;
  name: string;
  cost: number;
  description: string;
  targeted: boolean;
  selection_mode: "none" | "single" | "multiple" | "upgrade";
  cost_per_target: boolean;
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
export type RelicId = Character["relic"] | "kings_indian" | "queens_indian" | "hedgehog" | "initiative" | "ice_cream";
export interface Relic {
  id: RelicId;
  name: string;
  description: string;
  status: string | null;
}
export type Reward =
  | { type: "card"; definition: CardDefinition }
  | { type: "relic"; definition: Relic };
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
  node_reward: Relic | null;
  encounter: string;
  character: string;
  turn: number;
  round: number;
  actions_left: number;
  mana: number;
  mana_rule: string;
  initiative_turns_left: number;
  enemy_initiative_turns_left: number;
  pieces: Piece[];
  moves: { from: number; to: number; promotion: Promotion | null }[];
  hand: Card[];
  draw_count: number;
  discard_count: number;
  exhausted_count: number;
  deck_count: number;
  deck: CardDefinition[];
  next_battle_pieces: Piece[];
  relics: Relic[];
  log: string[];
  reward_choice: number | null;
  upgrade_claimed: boolean;
  rewards: Reward[];
  rest_options: RestOption[];
  upgrade_paths: UpgradePath[];
  royal_name: string;
  seed: number;
  floor: number;
}
export type Command =
  | { type: "select_node"; id: number }
  | { type: "move"; from: number; to: number; promotion?: number }
  | { type: "play_card"; id: number; target?: number }
  | { type: "play_card_selection"; id: number; targets?: number[]; upgrades?: { id: number; target: number }[] }
  | { type: "end_turn" | "enemy_step" | "claim_treasure" | "rest_skip" }
  | { type: "choose_reward"; index: number }
  | { type: "claim_reward"; index: number }
  | { type: "continue_reward" | "advance_floor" }
  | { type: "claim_upgrade"; id: number; target: number }
  | { type: "rest_upgrade"; id: number; target: number }
  | { type: "rest_revive"; id: number };

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
  reward_revisit: boolean;
  unclaimed_reward: boolean;
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
  key: string;
  from: number;
  from_key: string;
  from_name: string;
  to: number;
  to_key: string;
  to_name: string;
  effect: string;
  requirement: string;
  required_pieces: { kind: number; name: string; owned: boolean }[];
  unlocked: boolean;
}

export interface Promotion {
  kind: number;
  key: string;
  name: string;
  description: string;
}
