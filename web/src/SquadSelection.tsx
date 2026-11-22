import { RelicIcon } from "./ContentArt";
import { bossNames, floorNames } from "./progress";
import type { Character, Progress, RunMode } from "./types";

export function SquadSelection({ choices, character, onCharacter, floor, onFloor, progress, mode, developerMode, onBack, onStart, onToggleDeveloperMode, ready }: {
  choices: Character[]; character: string | null; onCharacter: (id: string) => void;
  floor: number; onFloor: (floor: number) => void; progress: Progress;
  mode: RunMode; developerMode: boolean; onBack: () => void; onStart: () => void;
  onToggleDeveloperMode: () => void; ready: boolean;
}) {
  const selected = choices.find((choice) => choice.id === character);
  return <main className="selection-page">
    <button className="text-button" onClick={onBack}>← 처음으로</button>
    <p className="eyebrow">CHOOSE YOUR EXPEDITION</p>
    <h1>원정에 나설 부대</h1>
    <p className="muted">{mode === "endless" ? "무한 모드 · 패배할 때까지 원정이 이어집니다." : "복수귀 부대를 선택하고 원정을 시작하세요."}</p>
    <div className="characters">
      {choices.filter((choice) => choice.id === "avenger").map((choice) => <button key={choice.id}
        type="button" className={`character ${character === choice.id ? "chosen" : ""}`}
        data-squad-id={choice.id} aria-pressed={character === choice.id} onClick={() => onCharacter(choice.id)}>
        <RelicIcon id={choice.relic} name={choice.name} className="character-icon" />
        <span className="eyebrow">{choice.title}</span><h2>{choice.name}</h2>
        <p>{choice.description}</p><small className="squad-tactics">카드 행마권 · 연속 이동 · 수동 턴 종료</small>
        <span className="character-radio">{character === choice.id ? "● 선택됨" : "○ 선택하기"}</span>
      </button>)}
    </div>
    <div className="developer-access">
      <button className={`developer-unlock ${developerMode ? "active" : ""}`} onClick={onToggleDeveloperMode}
        disabled={!ready} aria-pressed={developerMode}>개발자 모드 {developerMode ? "끄기" : "켜기"}</button>
    </div>
    {developerMode && mode === "campaign" && <div className="floor-selection" role="group" aria-label="개발자 출발 층 선택">
      {[1, 2, 3, 4].map((value) => <button key={value} type="button" className={floor === value ? "selected" : ""}
        aria-pressed={floor === value} onClick={() => onFloor(value)}>
        <strong>{value}층 · {floorNames[value]}</strong><span>보스 {bossNames[value]}</span>
      </button>)}
      {progress.final_unlocked && <button type="button" className={`final-destination ${floor === 0 ? "selected" : ""}`}
        aria-pressed={floor === 0} onClick={() => onFloor(0)}><strong>최종 결전 · 부족의 독재자</strong></button>}
    </div>}
    <div className="launch"><span className="muted">{selected?.name ?? "부대를 선택하세요"} · {mode === "endless" ? "무한 원정 · 1층 출발" : floor === 0 ? "최종 결전" : `${floor}층 원정`}</span>
      <button className="primary large" onClick={onStart} disabled={!ready || character !== "avenger"}>런 시작 <span>→</span></button>
    </div>
  </main>;
}
