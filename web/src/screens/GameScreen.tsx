import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { Command, EnemyInspection, Snapshot } from "../types";
import type { BoardMove } from "../game/useGamePresentation";
import { useGameInput } from "../game/useGameInput";
import { RunBoard } from "../board/RunBoard";
import { ActMap, nodeTypes } from "../ActMap";
import { BattleRewards } from "../BattleRewards";
import { ExpeditionStatus } from "../ExpeditionStatus";
import { floorLabel } from "../progress";
import { BattleScreen } from "./BattleScreen";
import { RestScreen } from "./RestScreen";
import { OutcomeScreen } from "./OutcomeScreen";

interface Props {
  snapshot: Snapshot;
  displayState: Snapshot;
  send: (command: Command) => boolean;
  inspectEnemy: (cell: number) => EnemyInspection | null;
  setError: (message: string) => void;
  onRestart: () => void;
  developerMode: boolean;
  upgradeHistory: { id: number; target: number }[];
  rewardUpgradeHistory: { id: number; target: number }[];
  presentationBusy: boolean;
  lastEnemyMove: BoardMove | null;
  boardRevision: number;
}

export function GameScreen({ snapshot, displayState: state, send: dispatchCommand, inspectEnemy, setError, onRestart,
  developerMode, upgradeHistory, rewardUpgradeHistory, presentationBusy, lastEnemyMove, boardRevision }: Props) {
  const [statusOpen, setStatusOpen] = useState(false);
  const [rewardUpgradeOpen, setRewardUpgradeOpen] = useState(false);
  const closeStatus = useCallback(() => setStatusOpen(false), []);
  const closeRewardUpgrade = useCallback(() => setRewardUpgradeOpen(false), []);
  const boardRef = useRef<HTMLDivElement>(null);
  const outcomeRef = useRef<HTMLElement>(null);
  const previousPhase = useRef<string | undefined>(undefined);
  const rewardOpen = state.phase === "reward" && !presentationBusy && !rewardUpgradeOpen;
  function send(command: Command) {
    if (statusOpen || presentationBusy) return false;
    const accepted = dispatchCommand(command);
    if (accepted && command.type === "claim_upgrade") setRewardUpgradeOpen(false);
    return accepted;
  }
  const input = useGameInput({ state: snapshot, gate: { modalOpen: statusOpen || rewardOpen, presentationBusy },
    rewardUpgradeOpen, send, inspectEnemy, setError, closeRewardUpgrade,
    returnToBoard: () => {
      if (window.matchMedia("(max-width: 1000px)").matches) boardRef.current?.scrollIntoView({ behavior: "smooth", block: "center" });
    },
  });
  useEffect(() => {
    if (state.phase !== previousPhase.current) {
      if (state.phase !== "combat" && state.phase !== "rest") outcomeRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
      else if (previousPhase.current) boardRef.current?.scrollIntoView({ behavior: "smooth", block: "center" });
    }
    previousPhase.current = state.phase;
  }, [state.phase]);
  useEffect(() => {
    if (input.targetCard) boardRef.current?.scrollIntoView({ behavior: "smooth", block: "center" });
  }, [input.targetCard]);
  const rewardRevisit = state.phase === "reward" && state.map.nodes.some((node) => node.current && node.visited);
  const showMap = (state.phase === "map" || rewardRevisit) && !rewardUpgradeOpen;
  const restView = state.phase === "rest" || rewardUpgradeOpen;
  const restPiece = state.rest_options.find((piece) => piece.id === input.restChoice?.id);
  const upgradeOpen = restView && !!restPiece && !restPiece.dead;
  const board = <RunBoard state={state} input={input} rewardUpgradeOpen={rewardUpgradeOpen}
    lastEnemyMove={lastEnemyMove} boardRevision={boardRevision} boardRef={boardRef} />;
  return <>
    {!presentationBusy && createPortal(<button className="primary expedition-status-trigger" onClick={() => setStatusOpen(true)} aria-haspopup="dialog" aria-expanded={statusOpen}>원정 현황</button>, document.body)}
    {showMap ? <ActMap state={state} send={send} coveredByReward={rewardRevisit} /> : (
      <main className="game-layout" data-phase={state.phase} data-upgrade-open={upgradeOpen || undefined} data-card-upgrade-open={!!input.focusedUpgrade || undefined}>
        <nav className="node-navigation" aria-label="원정 위치">
          <span>{floorLabel(state.floor, state.final_battle)} / {(state.map.nodes.find((node) => node.current)?.row ?? 0) + 1}번째 지점
            · {state.node_kind && nodeTypes[state.node_kind].label}</span>
        </nav>
        {!["treasure", "event", "shop", "victory", "defeat"].includes(state.phase) && (restView
          ? <RestScreen state={state} input={input} board={board} reward={rewardUpgradeOpen}
              history={rewardUpgradeOpen ? rewardUpgradeHistory : upgradeHistory}
              onClose={() => { input.chooseRest(null); if (rewardUpgradeOpen) closeRewardUpgrade(); }} />
          : <BattleScreen state={state} input={input} board={board} />)}
        {!["combat", "rest", "reward"].includes(state.phase) && <OutcomeScreen state={state} send={send}
          onRestart={() => { if (!statusOpen && !presentationBusy) onRestart(); }} outcomeRef={outcomeRef} />}
      </main>
    )}
    {statusOpen && <ExpeditionStatus state={state} onClose={closeStatus} developerMode={developerMode}
      onUpgrade={(id, target) => { if (developerMode && !presentationBusy) dispatchCommand({ type: "developer_set_upgrade", id, target }); }} />}
    {rewardOpen && <BattleRewards state={state} send={send} revisit={!!rewardRevisit}
      onUpgrade={() => { input.reset(); setRewardUpgradeOpen(true); }} />}
  </>;
}
