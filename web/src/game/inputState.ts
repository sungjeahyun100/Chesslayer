import type { Ability, Card, EnemyInspection, Promotion, Snapshot } from "../types";
import type { RestChoice } from "../RestPanel";

export type UpgradeSelection = { id: number; target: number };
export type InputState =
  | { mode: "idle" }
  | { mode: "piece"; cell: number }
  | { mode: "enemy"; inspection: EnemyInspection }
  | { mode: "ability"; cell: number; ability: Ability; targets: number[]; hovered: number | null }
  | { mode: "card"; card: Card; targets: number[]; upgrades: UpgradeSelection[]; focus: number | null }
  | { mode: "promotion"; from: number; to: number; options: Promotion[] }
  | { mode: "rest"; choice: RestChoice };

export type InputAction =
  | { type: "reset" }
  | { type: "select-piece"; cell: number }
  | { type: "inspect-enemy"; inspection: EnemyInspection }
  | { type: "select-card"; card: Card }
  | { type: "select-ability"; cell: number; ability: Ability }
  | { type: "cancel-ability" }
  | { type: "toggle-ability-target"; cell: number; limit: number }
  | { type: "hover-ability"; cell: number | null }
  | { type: "toggle-card-target"; id: number }
  | { type: "single-card-target"; id: number }
  | { type: "select-dead-pawn"; id: number; deadIds: number[] }
  | { type: "focus-upgrade"; id: number | null }
  | { type: "stage-upgrade"; target: number }
  | { type: "remove-upgrade"; id: number }
  | { type: "select-rest"; choice: RestChoice }
  | { type: "promote"; from: number; to: number; options: Promotion[] }
  | { type: "cancel-promotion" };

export const idleInput: InputState = { mode: "idle" };
const toggle = (values: number[], value: number) => values.includes(value)
  ? values.filter((item) => item !== value) : [...values, value];

// The reducer only stores UI choices. Legal targets and actions come from WASM.
export function inputReducer(state: InputState, action: InputAction): InputState {
  switch (action.type) {
    case "reset": return idleInput;
    case "select-piece": return { mode: "piece", cell: action.cell };
    case "inspect-enemy": return { mode: "enemy", inspection: action.inspection };
    case "select-card": return { mode: "card", card: action.card, targets: [], upgrades: [], focus: null };
    case "select-ability": return { mode: "ability", cell: action.cell, ability: action.ability, targets: [], hovered: null };
    case "cancel-ability": return state.mode === "ability" ? { mode: "piece", cell: state.cell } : state;
    case "toggle-ability-target": return state.mode === "ability" && (state.targets.includes(action.cell) || state.targets.length < action.limit)
      ? { ...state, targets: toggle(state.targets, action.cell) } : state;
    case "hover-ability": return state.mode === "ability" ? { ...state, hovered: action.cell } : state;
    case "toggle-card-target": return state.mode === "card" ? { ...state, targets: toggle(state.targets, action.id) } : state;
    case "single-card-target": return state.mode === "card" ? { ...state, targets: [action.id] } : state;
    case "select-dead-pawn": return state.mode === "card" ? { ...state, targets: state.targets.includes(action.id)
      ? state.targets.filter((id) => id !== action.id)
      : [...state.targets.filter((id) => !action.deadIds.includes(id)), action.id] } : state;
    case "focus-upgrade": return state.mode === "card" ? { ...state, focus: action.id } : state;
    case "stage-upgrade": return state.mode === "card" && state.focus !== null
      ? { ...state, upgrades: [...state.upgrades.filter((choice) => choice.id !== state.focus), { id: state.focus, target: action.target }], focus: null } : state;
    case "remove-upgrade": return state.mode === "card" ? { ...state, upgrades: state.upgrades.filter((choice) => choice.id !== action.id) } : state;
    case "select-rest": return { mode: "rest", choice: action.choice };
    case "promote": return { mode: "promotion", from: action.from, to: action.to, options: action.options };
    case "cancel-promotion": return state.mode === "promotion" ? { mode: "piece", cell: state.from } : state;
  }
}

// Read-only projections keep renderers independent of reducer actions.
export function inputView(input: InputState) {
  return {
    selected: input.mode === "piece" || input.mode === "ability" ? input.cell : input.mode === "promotion" ? input.from : null,
    enemyInspection: input.mode === "enemy" ? input.inspection : null,
    activeAbility: input.mode === "ability" ? input.ability : null,
    abilityTargets: input.mode === "ability" ? input.targets : [],
    hoveredAbilityTarget: input.mode === "ability" ? input.hovered : null,
    targetCard: input.mode === "card" ? input.card : null,
    cardTargets: input.mode === "card" ? input.targets : [],
    cardUpgrades: input.mode === "card" ? input.upgrades : [],
    upgradeFocus: input.mode === "card" ? input.focus : null,
    restChoice: input.mode === "rest" ? input.choice : null,
    promotion: input.mode === "promotion" ? input : null,
  };
}

export interface InputGate {
  modalOpen: boolean;
  presentationBusy: boolean;
}

// Future animation playback can use presentationBusy without changing handlers.
// Enemy inspection remains available during the enemy turn, as before.
export function inputPolicy(state: Pick<Snapshot, "phase" | "turn" | "effect_choice">, input: InputState, gate: InputGate) {
  const blocked = gate.modalOpen || gate.presentationBusy;
  const board = !blocked && input.mode !== "promotion";
  return {
    blocked,
    board,
    act: board && state.phase === "combat" && state.turn === 0 && !state.effect_choice,
    inspect: board && state.phase === "combat" && !state.effect_choice,
    effect: board && state.phase === "combat" && !!state.effect_choice,
  };
}
