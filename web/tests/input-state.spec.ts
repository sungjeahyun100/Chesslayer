import { expect, test } from "@playwright/test";
import { idleInput, inputPolicy, inputReducer, inputView, type InputState } from "../src/game/inputState";
import type { Ability, Card } from "../src/types";

const card: Card = { id: 1, targets: [7, 8, 10024], disabled_reason: null, definition: {
  kind: "frontline", name: "전선", description: "", cost: 1, targeted: true, selection_mode: "multiple",
  cost_per_target: false, exhaust: false, retain: false, innate: false, ethereal: false,
} };
const ability: Ability = { piece_id: 9, kind: "king_reposition", name: "재배치", description: "", targets: [] };

test("switching input mode discards incompatible card, inspection and ability data", () => {
  let input: InputState = inputReducer(idleInput, { type: "select-card", card });
  input = inputReducer(input, { type: "toggle-card-target", id: 7 });
  input = inputReducer(input, { type: "select-card", card: { ...card, id: 2 } });
  expect(inputView(input).cardTargets).toEqual([]);
  input = inputReducer(input, { type: "select-ability", cell: 60, ability });
  expect(inputView(input).targetCard).toBeNull();
  input = inputReducer(input, { type: "toggle-ability-target", cell: 4, limit: 2 });
  input = inputReducer(input, { type: "hover-ability", cell: 4 });
  input = inputReducer(input, { type: "cancel-ability" });
  expect(input).toEqual({ mode: "piece", cell: 60 });
  expect(inputView(input).abilityTargets).toEqual([]);
  expect(inputView(input).hoveredAbilityTarget).toBeNull();
  input = inputReducer(input, { type: "inspect-enemy", inspection: { piece_id: 20, cell: 4, moves: [] } });
  input = inputReducer(input, { type: "select-piece", cell: 52 });
  expect(inputView(input).enemyInspection).toBeNull();
});

test("multiple targets keep selection order, toggle off, and replace only a dead pawn", () => {
  let input = inputReducer(idleInput, { type: "select-card", card });
  for (const id of [7, 10024, 8, 7]) input = inputReducer(input, { type: "toggle-card-target", id });
  expect(inputView(input).cardTargets).toEqual([10024, 8]);
  input = inputReducer(input, { type: "select-dead-pawn", id: 7, deadIds: [7, 8] });
  expect(inputView(input).cardTargets).toEqual([10024, 7]);
  input = inputReducer(input, { type: "select-dead-pawn", id: 7, deadIds: [7, 8] });
  expect(inputView(input).cardTargets).toEqual([10024]);
});

test("temporary upgrades preserve staged paths while changing or cancelling focus", () => {
  let input = inputReducer(idleInput, { type: "select-card", card });
  input = inputReducer(input, { type: "focus-upgrade", id: 7 });
  input = inputReducer(input, { type: "stage-upgrade", target: 8 });
  input = inputReducer(input, { type: "focus-upgrade", id: 9 });
  input = inputReducer(input, { type: "focus-upgrade", id: null });
  expect(inputView(input).cardUpgrades).toEqual([{ id: 7, target: 8 }]);
  input = inputReducer(input, { type: "focus-upgrade", id: 7 });
  input = inputReducer(input, { type: "stage-upgrade", target: 10 });
  expect(inputView(input).cardUpgrades).toEqual([{ id: 7, target: 10 }]);
  input = inputReducer(input, { type: "remove-upgrade", id: 7 });
  expect(inputView(input).cardUpgrades).toEqual([]);
  expect(inputReducer(input, { type: "reset" })).toEqual(idleInput);
});

test("king target limits allow deselection and a fresh ability clears hover and targets", () => {
  let input = inputReducer(idleInput, { type: "select-ability", cell: 60, ability });
  for (const cell of [1, 2, 3]) input = inputReducer(input, { type: "toggle-ability-target", cell, limit: 2 });
  expect(inputView(input).abilityTargets).toEqual([1, 2]);
  input = inputReducer(input, { type: "toggle-ability-target", cell: 1, limit: 2 });
  input = inputReducer(input, { type: "toggle-ability-target", cell: 3, limit: 2 });
  expect(inputView(input).abilityTargets).toEqual([2, 3]);
  input = inputReducer(input, { type: "select-ability", cell: 60, ability });
  expect(inputView(input).abilityTargets).toEqual([]);
});

test("promotion, enemy turns, engine choices and overlays share one input gate", () => {
  const combat = { phase: "combat", turn: 0, effect_choice: null } as const;
  const gate = { modalOpen: false, presentationBusy: false };
  const promotion = inputReducer(idleInput, { type: "promote", from: 8, to: 0, options: [] });
  expect(inputPolicy(combat, promotion, gate).act).toBe(false);
  expect(inputPolicy(combat, promotion, gate).board).toBe(false);
  expect(inputReducer(promotion, { type: "cancel-promotion" })).toEqual({ mode: "piece", cell: 8 });
  expect(inputReducer(promotion, { type: "reset" })).toEqual(idleInput);
  const enemy = inputPolicy({ ...combat, turn: 1 }, idleInput, gate);
  expect(enemy.act).toBe(false);
  expect(enemy.inspect).toBe(true);
  const choice = inputPolicy({ ...combat, effect_choice: { name: "배치", targets: [1], can_finish: true } }, idleInput, gate);
  expect(choice.act).toBe(false);
  expect(choice.inspect).toBe(false);
  expect(choice.effect).toBe(true);
  for (const lock of [{ ...gate, modalOpen: true }, { ...gate, presentationBusy: true }]) {
    const policy = inputPolicy(combat, idleInput, lock);
    expect(policy.board || policy.act || policy.inspect || policy.effect).toBe(false);
  }
});
