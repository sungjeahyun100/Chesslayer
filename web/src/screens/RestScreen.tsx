import type { ReactNode } from "react";
import type { Snapshot } from "../types";
import type { GameInput } from "../game/useGameInput";
import { BattleHeading } from "../components/BattleHeading";
import { RestPanel } from "../RestPanel";

export function RestScreen({ state, input, board, reward, history, onClose }: {
  state: Snapshot; input: GameInput; board: ReactNode; reward: boolean;
  history: { id: number; target: number }[]; onClose: () => void;
}) {
  return <>
    <div className="battle-column">
      <section className="battle-area">
        <BattleHeading state={state} rewardUpgradeOpen={reward} />
        {board}
      </section>
    </div>
    <aside className="sidebar">
      <RestPanel state={state} choice={input.restChoice} onChoose={input.chooseRest}
        onCancel={onClose} send={input.send} history={history} onContinue={onClose} reward={reward} />
    </aside>
  </>;
}
