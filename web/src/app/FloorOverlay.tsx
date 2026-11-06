import type { FloorTransition } from "../game/useGamePresentation";
import { floorLabel, floorNames } from "../progress";

export function FloorOverlay({ floorTransition, skipFloorTransition }: { floorTransition: FloorTransition | null; skipFloorTransition: () => void }) {
  return <>
    {floorTransition && <div className={`floor-transition stage-${floorTransition.stage} act-${floorTransition.next.floor}`} role="status" aria-label={`${floorLabel(floorTransition.next.floor, floorTransition.next.final_battle)} 진입`}>
      <div className="floor-transition-title"><small>CHESSLAYER · EXPEDITION BEGINS</small><strong>{floorTransition.next.final_battle ? "FINALE" : `ACT ${["", "I", "II", "III", "IV"][floorTransition.next.floor]}`}</strong><span>{floorTransition.next.final_battle ? "부족의 독재자" : floorNames[floorTransition.next.floor]}</span><small>{floorTransition.next.character}</small></div>
      <button onClick={skipFloorTransition} aria-label="층 전환 연출 건너뛰기">건너뛰기 ↗</button>
    </div>}
  </>;
}
