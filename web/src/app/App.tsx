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

export function App() {
  const [screen, setScreen] = useState<AppScreen>("home");
  const [character, setCharacter] = useState("guardian");
  const [startFloor, setStartFloor] = useState(1);
  const enterGame = useCallback(() => setScreen("game"), []);
  const session = useGameSession(screen === "game", enterGame);
  const { ready, choices, entries, snapshot, error, setError, storage, presentation } = session;
  const { progress, developerMode, hasSave, saveNotice } = storage;
  const { displayState, floorTransition, skipFloorTransition } = presentation;
  function toggleDeveloperMode() {
    if (developerMode) {
      if (!progress.unlocked_squads.includes(character)) setCharacter("guardian");
      setStartFloor(1);
    }
    storage.toggleDeveloperMode();
  }
  return <>
    <FloorOverlay floorTransition={floorTransition} skipFloorTransition={skipFloorTransition} />
    <AppChrome screen={screen} state={displayState} onHome={() => setScreen("home")} />
    {error && <div className="error" role="alert">{error}<button className="text-button" onClick={() => setError("")}>닫기</button></div>}
    {saveNotice && <p className="save-notice" role="status">{saveNotice}</p>}
    {screen === "home" && <HomeScreen ready={ready} hasSave={hasSave} resume={session.resume} onNavigate={setScreen} />}
    {screen === "choose" && <SquadSelection choices={choices} character={character} onCharacter={setCharacter}
      floor={startFloor} onFloor={setStartFloor} progress={developerMode ? developerProgress(progress, choices.map((choice) => choice.id)) : progress}
      developerMode={developerMode} onBack={() => setScreen("home")} onStart={() => session.start(character, startFloor)}
      onToggleDeveloperMode={toggleDeveloperMode} ready={ready} />}
    {screen === "compendium" && <Compendium choices={choices} entries={entries} onBack={() => setScreen("home")} />}
    {screen === "sandbox" && <PieceSandbox choices={choices} entries={entries} onBack={() => setScreen("home")} />}
    {screen === "game" && snapshot && displayState && <GameScreen key={session.runId} snapshot={snapshot} displayState={displayState}
      send={session.send} inspectEnemy={session.inspectEnemy} setError={setError} developerMode={developerMode}
      upgradeHistory={storage.upgradeHistory} rewardUpgradeHistory={storage.rewardUpgradeHistory}
      presentationBusy={presentation.presentationBusy} lastEnemyMove={presentation.lastEnemyMove} boardRevision={presentation.boardRevision}
      onRestart={() => { setScreen("choose"); setError(""); }} />}
    <footer>CHESSLAYER <span>원정과 부대 해금은 이 브라우저에 저장됩니다.</span></footer>
  </>;
}
