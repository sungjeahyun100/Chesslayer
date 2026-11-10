import type { FloorTransition } from "../game/useGamePresentation";
import { floorLabel, floorNames } from "../progress";

export function FloorOverlay({ floorTransition, skipFloorTransition }: { floorTransition: FloorTransition | null; skipFloorTransition: () => void }) {
  const next = floorTransition?.next;
  return <>
    {floorTransition && <div className={`floor-transition stage-${floorTransition.stage} act-${floorTransition.next.floor}`} role="status" aria-label={`${floorLabel(floorTransition.next.floor, floorTransition.next.final_battle)} 진입`}>
      {next && <div className="floor-transition-title"><small>CHESSLAYER · {next.mode === "endless" ? "ENDLESS EXPEDITION" : "EXPEDITION BEGINS"}</small><strong>{next.mode === "endless" ? `${next.floor}층` : next.final_battle ? "FINALE" : `ACT ${["", "I", "II", "III", "IV"][next.floor]}`}</strong><span>{next.mode === "endless" ? "무한 원정" : next.final_battle ? "부족의 독재자" : floorNames[next.floor]}</span><small>{next.character}</small></div>}
      <button onClick={skipFloorTransition} aria-label="층 전환 연출 건너뛰기">건너뛰기 ↗</button>
    </div>}
  </>;
}
