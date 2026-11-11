import type { ReactNode } from "react";
import type { Snapshot } from "../types";
import type { GameInput } from "../game/useGameInput";
import { BattleHeading } from "../components/BattleHeading";
import { ActionHint } from "../components/ActionHint";
import { CardHand } from "../components/CardHand";
import { PieceInspector } from "../components/PieceInspector";
import { RelicList } from "../components/RelicList";

export function BattleScreen({ state, cardState, cardMovement, input, board }: { state: Snapshot; cardState: Snapshot;
  cardMovement: { sequence: number; before: Snapshot; next: Snapshot } | null; input: GameInput; board: ReactNode }) {
  const { enemyInspection, selected, activeAbility, targetCard } = input;
  const inspectedEnemy = state.phase === "combat" ? state.pieces.find((piece) => piece.id === enemyInspection?.piece_id && piece.cell === enemyInspection?.cell) : undefined;
  const selectedPiece = inspectedEnemy || state.pieces.find((piece) => piece.cell === selected);
  const selectedAbilities = state.abilities?.filter((ability) => ability.piece_id === selectedPiece?.id) ?? [];
  const initiativeRemaining = selectedPiece?.owner === 1 ? state.enemy_initiative_turns_left ?? 0 : state.initiative_turns_left ?? 0;
  return <>
    <div className="battle-column">
      <section className="battle-area">
        <BattleHeading state={state} />
        {board}
        <ActionHint state={state} input={input} />
      </section>
      {state.phase === "combat" && <CardHand state={cardState} movement={cardMovement} selectedCardId={targetCard?.id} selectedCount={input.stagedCount}
        onChoose={input.chooseCard} disabled={!input.policy.act} />}
    </div>
    <aside className="sidebar">
      {selectedPiece && <PieceInspector piece={selectedPiece} enemy={!!inspectedEnemy} abilities={selectedAbilities}
        activeAbility={activeAbility} canUseAbility={state.turn === 0} inputEnabled={input.policy.act} onAbility={input.chooseAbility}
        boss={state.boss} initiativeRemaining={initiativeRemaining} />}
      <div className="combat-resource" aria-label={state.ammo !== null ? `탄약 ${state.ammo}` : `마나 ${state.mana}`}>
        <span className="mana-orb">{state.ammo ?? state.mana}</span>
        <span>{state.ammo !== null ? `탄약 / ${state.ammo_capacity}` : "마나"}</span>
      </div>
      <RelicList relics={state.relics} />
    </aside>
  </>;
}
