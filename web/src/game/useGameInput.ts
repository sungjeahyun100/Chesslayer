import { useEffect, useReducer } from "react";
import type { Ability, Card, Command, EnemyInspection, Snapshot } from "../types";
import type { RestChoice } from "../RestPanel";
import { idleInput, inputPolicy, inputReducer, inputView, type InputGate } from "./inputState";

interface Options {
  state: Snapshot;
  gate: InputGate;
  rewardUpgradeOpen: boolean;
  send: (command: Command) => boolean;
  inspectEnemy: (cell: number) => EnemyInspection | null;
  setError: (message: string) => void;
  closeRewardUpgrade: () => void;
  returnToBoard: () => void;
}

export function useGameInput({ state, gate, rewardUpgradeOpen, send, inspectEnemy, setError, closeRewardUpgrade, returnToBoard }: Options) {
  const [input, dispatch] = useReducer(inputReducer, idleInput);
  const view = inputView(input);
  const { selected, enemyInspection, activeAbility, abilityTargets, targetCard, cardTargets, cardUpgrades, upgradeFocus } = view;
  const policy = inputPolicy(state, input, gate);
  const boardPieces = rewardUpgradeOpen ? state.next_battle_pieces : state.pieces;
  const focusedUpgrade = state.pieces.find((piece) => piece.id === upgradeFocus);
  const stagedCount = targetCard?.definition.selection_mode === "upgrade" ? cardUpgrades.length : cardTargets.length;
  const stagedCost = targetCard?.definition.cost_per_target ? stagedCount : targetCard?.definition.cost ?? 0;

  // Includes worker turns and developer commands that arrive outside this controller.
  useEffect(() => { dispatch({ type: "reset" }); }, [state]);
  useEffect(() => {
    const cancel = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || policy.blocked) return;
      if (rewardUpgradeOpen) closeRewardUpgrade();
      dispatch(input.mode === "card" && input.focus !== null ? { type: "focus-upgrade", id: null } : { type: "reset" });
      setError("");
    };
    window.addEventListener("keydown", cancel);
    return () => window.removeEventListener("keydown", cancel);
  }, [policy.blocked, rewardUpgradeOpen, closeRewardUpgrade, setError, input]);

  function execute(command: Command) {
    if (policy.blocked) return;
    if (send(command)) dispatch({ type: "reset" });
  }
  function cancelCard() {
    if (policy.blocked) return;
    dispatch({ type: "reset" });
    setError("");
  }
  function chooseCard(card: Card) {
    if (!policy.act) return;
    if (card.disabled_reason) { setError(card.disabled_reason); return; }
    setError("");
    if (card.definition.targeted) dispatch({ type: "select-card", card });
    else execute({ type: "play_card", id: card.id });
  }
  function chooseAbility(ability: Ability) {
    if (!policy.act || !ability.targets.length) return;
    const actor = state.pieces.find((piece) => piece.id === ability.piece_id);
    if (!actor) return;
    dispatch(activeAbility === ability ? { type: "cancel-ability" } : { type: "select-ability", cell: actor.cell, ability });
  }
  function clickSquare(cell: number) {
    if (!policy.board) return;
    if (rewardUpgradeOpen) {
      const boardPiece = boardPieces.find((piece) => piece.cell === cell);
      const ally = state.rest_options.find((piece) => piece.can_upgrade && piece.id === boardPiece?.id);
      if (ally) dispatch({ type: "select-rest", choice: { cell, id: ally.id } });
      return;
    }
    if (state.phase === "rest") {
      const ally = state.rest_options.find((piece) => !piece.dead && piece.cell === cell);
      const occupant = state.pieces.find((piece) => piece.cell === cell);
      const candidate = !occupant && state.rest_options.find((piece) => piece.can_revive && piece.origin === cell);
      const option = ally || candidate;
      if (option) dispatch({ type: "select-rest", choice: { cell, id: option.id } });
      return;
    }
    if (state.phase !== "combat") return;
    const piece = state.pieces.find((piece) => piece.cell === cell);
    const targetId = piece?.id ?? 10000 + cell;
    if (state.effect_choice) {
      if (policy.effect && state.effect_choice.targets.includes(targetId)) execute({ type: "resolve_effect", target: targetId });
      return;
    }
    if (targetCard) {
      if (!policy.act || !targetCard.targets.includes(targetId)) return;
      if (targetCard.definition.selection_mode === "upgrade") {
        if (!piece) return;
        if (!cardUpgrades.some((choice) => choice.id === piece.id) && cardUpgrades.length >= state.mana) {
          setError("남은 마나보다 많은 기물을 선택할 수 없습니다.");
          return;
        }
        dispatch({ type: "focus-upgrade", id: piece.id });
      } else if (targetCard.definition.selection_mode === "multiple") {
        dispatch({ type: "toggle-card-target", id: targetId });
      } else if (["limiter_release", "sacred_domain"].includes(targetCard.definition.kind)) {
        dispatch({ type: "single-card-target", id: targetId });
      } else execute({ type: "play_card", id: targetCard.id, target: targetId });
      return;
    }
    if (activeAbility) {
      if (!policy.act) return;
      const actor = state.pieces.find((candidate) => candidate.id === activeAbility.piece_id);
      if (actor && activeAbility.targets.some((target) => target.cell === cell)) {
        if (activeAbility.kind === "king_reposition") {
          dispatch({ type: "toggle-ability-target", cell, limit: actor.key === "tempest_king" ? 2 : 6 });
        } else execute({ type: "special_action", from: actor.cell, to: cell });
        return;
      }
      dispatch({ type: "cancel-ability" });
    }
    if (policy.act && selected !== null && state.moves.some((move) => move.from === selected && move.to === cell)) {
      const options = state.moves.filter((move) => move.from === selected && move.to === cell).flatMap((move) => move.promotion ? [move.promotion] : []);
      if (options.length) dispatch({ type: "promote", from: selected, to: cell, options });
      else execute({ type: "move", from: selected, to: cell });
      return;
    }
    if (piece?.owner === 1 && policy.inspect) {
      const inspection = enemyInspection?.piece_id === piece.id ? null : inspectEnemy(cell);
      dispatch(inspection ? { type: "inspect-enemy", inspection } : { type: "reset" });
      return;
    }
    dispatch(policy.act && piece?.owner === 0 && selected !== cell ? { type: "select-piece", cell } : { type: "reset" });
  }
  function confirmCard() {
    if (!policy.act || !targetCard || stagedCount < 1 || stagedCost > state.mana) return;
    if (["multiple", "upgrade"].includes(targetCard.definition.selection_mode)) {
      execute({ type: "play_card_selection", id: targetCard.id,
        targets: targetCard.definition.selection_mode === "multiple" ? cardTargets : [],
        upgrades: targetCard.definition.selection_mode === "upgrade" ? cardUpgrades : [] });
    } else execute({ type: "play_card", id: targetCard.id, target: cardTargets[0] });
  }
  function confirmAbility() {
    if (!policy.act || !activeAbility || abilityTargets.length < 2) return;
    const actor = state.pieces.find((piece) => piece.id === activeAbility.piece_id);
    if (actor) execute({ type: "king_reposition", from: actor.cell, targets: abilityTargets });
  }
  function confirmPromotion(kind: number) {
    if (policy.blocked || input.mode !== "promotion") return;
    execute({ type: "move", from: input.from, to: input.to, promotion: kind });
  }
  function confirmCardUpgrade(target: number) {
    if (policy.blocked || !targetCard || !focusedUpgrade) return;
    dispatch({ type: "stage-upgrade", target });
    returnToBoard();
  }
  function closeCardUpgrade() {
    if (policy.blocked) return;
    dispatch({ type: "focus-upgrade", id: null });
    returnToBoard();
  }
  function chooseDeadPawn(id: number) {
    if (policy.act) dispatch({ type: "select-dead-pawn", id, deadIds: state.rest_options.filter((piece) => piece.dead).map((piece) => piece.id) });
  }
  function removeCardUpgrade(id: number) {
    if (policy.act) dispatch({ type: "remove-upgrade", id });
  }
  function chooseRest(choice: RestChoice | null) {
    if (policy.blocked) return;
    dispatch(choice ? { type: "select-rest", choice } : { type: "reset" });
  }
  function cancelAbility() {
    if (!policy.blocked) dispatch({ type: "cancel-ability" });
  }
  function cancelPromotion() {
    if (!policy.blocked) dispatch({ type: "cancel-promotion" });
  }
  function hoverAbility(cell: number | null) {
    if (!policy.blocked) dispatch({ type: "hover-ability", cell });
  }
  return { input, ...view, policy, focusedUpgrade, stagedCount, stagedCost, clickSquare, chooseCard, chooseAbility,
    confirmCard, cancelCard, confirmAbility, cancelAbility, confirmPromotion, cancelPromotion,
    confirmCardUpgrade, closeCardUpgrade, chooseDeadPawn, removeCardUpgrade, chooseRest, hoverAbility,
    send: execute, reset: () => dispatch({ type: "reset" }) };
}

export type GameInput = ReturnType<typeof useGameInput>;
