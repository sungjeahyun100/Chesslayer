import { RelicIcon } from "./ContentArt";
import { bossNames, floorNames } from "./progress";
import type { Character, Progress } from "./types";

const relicNames: Record<string, string> = {
  haste: "민첩성", self_defense: "호신용품", carnival_flare: "카니발의 신호탄",
  fire_control: "사격 통제", recon_scope: "정찰 조준경",
};
const tactics: Record<string, string> = {
  guardian: "기물 강화 · 가호 · 고유 행마",
  vanguard: "부대 행동 2회 · 민첩 · 희생",
  heretic: "식재료 · 시체 · 황충의 축복",
  artillery: "창고 2개 · 탄약 · 원거리 포격",
  special: "소수 정예 · 사격 · 기동",
};

export function SquadSelection({ choices, character, onCharacter, floor, onFloor, progress, onBack, onStart, ready }: {
  choices: Character[]; character: string; onCharacter: (id: string) => void;
  floor: number; onFloor: (floor: number) => void; progress: Progress;
  onBack: () => void; onStart: () => void; ready: boolean;
}) {
  const selected = choices.find((choice) => choice.id === character);
  return <main className="selection-page">
    <button className="text-button" onClick={onBack}>← 처음으로</button>
    <p className="eyebrow">CHOOSE YOUR EXPEDITION</p>
    <h1>원정에 나설 부대</h1>
    <p className="muted">각 층의 보스를 격파하면 다음 층과 새로운 부대가 해금됩니다.</p>
    <div className="characters">
      {choices.map((choice) => {
        const unlocked = progress.unlocked_squads.includes(choice.id);
        return <button className={`character ${character === choice.id ? "chosen" : ""} ${unlocked ? "" : "locked"}`}
          key={choice.id} disabled={!unlocked} onClick={() => onCharacter(choice.id)} aria-pressed={character === choice.id}>
          <RelicIcon id={choice.relic} name={relicNames[choice.relic] ?? choice.name} className="character-icon" />
          <span className="eyebrow">{choice.title}</span><h2>{choice.name}</h2>
          <p>{choice.description}</p><small className="squad-tactics">{tactics[choice.id]}</small>
          <span className="character-radio">{!unlocked ? `${choice.unlock_floor}층 보스 격파 시 해금` : character === choice.id ? "● 선택됨" : "○ 선택 가능"}</span>
        </button>;
      })}
    </div>
    <div className="floor-selection" role="group" aria-label="출발 층 선택">
      {[1, 2, 3, 4].map((value) => <button key={value} className={floor === value ? "selected" : ""}
        disabled={value > progress.highest_floor} aria-pressed={floor === value} onClick={() => onFloor(value)}>
        <strong>{value}층 · {floorNames[value]}</strong><span>{value > progress.highest_floor ? `${value - 1}층 격파 후 해금` : `15단계 · 보스 ${bossNames[value]}`}</span>
      </button>)}
      {progress.final_unlocked && <button className={`final-destination ${floor === 0 ? "selected" : ""}`} aria-pressed={floor === 0} onClick={() => onFloor(0)}>
        <strong>최종 결전 · 부족의 독재자</strong><span>{progress.final_defeated ? "격파 완료 · 재도전 가능" : "별도 결전 · 보스 전투 1회"}</span>
      </button>}
    </div>
    <div className="launch"><span className="muted">{selected?.name} · {floor === 0 ? "최종 결전" : `${floor}층 원정`}<br />해금 기록은 새 원정에도 유지됩니다.</span>
      <button className="primary large" onClick={onStart} disabled={!ready || !progress.unlocked_squads.includes(character)}>런 시작 <span>→</span></button>
    </div>
  </main>;
}
