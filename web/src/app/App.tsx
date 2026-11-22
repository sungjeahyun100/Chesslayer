import { useCallback, useState } from "react";
import { useGameSession } from "../game/useGameSession";
import { developerProgress } from "../progress";
import { SquadSelection } from "../SquadSelection";
import { Compendium } from "../Compendium";
import { PieceSandbox } from "../PieceSandbox";
import { HomeScreen } from "../screens/HomeScreen";
import { GameScreen } from "../screens/GameScreen";
import { AppChrome, type AppScreen } from "./AppChrome";
import { FloorOverlay } from "./FloorOverlay";
import type { RunMode } from "../types";

export function App() {
  const [screen, setScreen] = useState<AppScreen>("home");
  const [character, setCharacter] = useState<string | null>(null);
  const [startFloor, setStartFloor] = useState(1);
  const [mode, setMode] = useState<RunMode>("campaign");
  const enterGame = useCallback(() => setScreen("game"), []);
  const session = useGameSession(screen === "game", enterGame);
  const { ready, choices, entries, snapshot, error, setError, storage, presentation } = session;
  const { progress, developerMode, hasSave, saveNotice } = storage;
  const { displayState, floorTransition, skipFloorTransition } = presentation;
  function choose(nextMode: RunMode) {
    setMode(nextMode);
    setCharacter(null);
    setStartFloor(1);
    setScreen("choose");
  }
  return <>
    <FloorOverlay floorTransition={floorTransition} skipFloorTransition={skipFloorTransition} />
    <AppChrome screen={screen} state={displayState} onHome={() => setScreen("home")} />
    {error && <div className="error" role="alert">{error}<button className="text-button" onClick={() => setError("")}>닫기</button></div>}
    {saveNotice && <p className="save-notice" role="status">{saveNotice}</p>}
    {screen === "home" && <HomeScreen ready={ready} hasSave={hasSave} hasEndlessSave={storage.hasEndlessSave}
      endlessBest={storage.endlessBest} resume={session.resume} onNavigate={setScreen}
      onStart={choose} />}
    {screen === "choose" && <SquadSelection choices={choices} character={character} onCharacter={setCharacter}
      floor={startFloor} onFloor={setStartFloor} progress={developerMode ? developerProgress(progress, ["avenger"]) : progress}
      mode={mode} developerMode={developerMode} onBack={() => setScreen("home")}
      onStart={() => { if (character === "avenger") session.start(character, startFloor, mode); }}
      onToggleDeveloperMode={() => { if (developerMode) setStartFloor(1); storage.toggleDeveloperMode(); }} ready={ready} />}
    {screen === "compendium" && <Compendium choices={choices} entries={entries} onBack={() => setScreen("home")} />}
    {screen === "sandbox" && <PieceSandbox entries={entries} onBack={() => setScreen("home")} />}
    {screen === "game" && snapshot && displayState && <GameScreen key={session.runId} snapshot={snapshot} displayState={displayState}
      cardMovement={session.cardMovement}
      send={session.send} inspectEnemy={session.inspectEnemy} setError={setError} developerMode={developerMode}
      upgradeHistory={storage.upgradeHistory} rewardUpgradeHistory={storage.rewardUpgradeHistory}
      endlessBest={storage.endlessBest} endlessRunBest={storage.endlessRunBest}
      presentationBusy={presentation.presentationBusy} lastEnemyMove={presentation.lastEnemyMove} boardRevision={presentation.boardRevision}
      animation={presentation.animation}
      statusEffects={presentation.statusEffects}
      onRestart={() => { setError(""); choose(snapshot.mode); }} />}
    <footer>CHESSLAYER <span>원정 진행은 이 브라우저에 저장됩니다.</span></footer>
  </>;
}
