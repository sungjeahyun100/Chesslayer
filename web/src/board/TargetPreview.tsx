import type { Ability, Snapshot } from "../types";

export function TargetPreview({ legal, occupied, enemyMove, enemyCapture, bossTarget, hookTarget, abilityTarget, fallout, staged, selectionNumber }: {
  legal: boolean; occupied: boolean; enemyMove: boolean; enemyCapture: boolean; bossTarget: boolean; hookTarget: boolean;
  abilityTarget: Ability["targets"][number] | undefined; fallout: Snapshot["fallout"][number] | undefined; staged: boolean; selectionNumber: number | string;
}) {
  return <>
                          {legal && (
                            <span
                              className={occupied ? "capture-dot" : "move-dot"}
                            />
                          )}
                          {enemyMove && <span className="enemy-move-dot" />}
                          {enemyCapture && <span className="enemy-capture-ring" />}
                          {bossTarget && <span className="boss-target-mark" title="예고된 행동의 현재 대상">!</span>}
                          {hookTarget && <span className="boss-hook-mark" title="갈고리 반격 대상">⚓</span>}
                          {abilityTarget && <span className="aim-target">{abilityTarget.chance === null ? "✦" : `${abilityTarget.chance}%`}</span>}
                          {fallout && <span className="fallout-badge" title={`방사성 낙진 · 남은 ${fallout.remaining}라운드`}>{fallout.remaining}</span>}
                          {staged && <span className="selection-number">{selectionNumber}</span>}
  </>;
}
