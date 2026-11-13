import type { StatusChange } from "../game/statusVisuals";

export function StatusEffects({ effects }: { effects: StatusChange[] }) {
  if (!effects.length) return null;
  return <div className="status-effect-layer" aria-hidden="true">
    {effects.map((effect) => <span key={effect.id}
      className={`status-effect status-effect-${effect.kind} status-effect-${effect.event}`}
      data-status-effect={effect.kind} data-status-event={effect.event} data-cell={effect.cell}
      style={{ left: `${(effect.cell % 8 + .5) * 12.5}%`, top: `${(Math.floor(effect.cell / 8) + .5) * 12.5}%` }}>
      <i className="status-effect-ring" /><i className="status-effect-core" />
      <i className="status-effect-spark one" /><i className="status-effect-spark two" />
    </span>)}
  </div>;
}
