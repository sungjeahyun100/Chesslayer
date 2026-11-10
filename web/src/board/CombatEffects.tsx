import { useEffect, useState, type CSSProperties } from "react";
import { PieceImage } from "../PieceImage";
import { combatTiming, type CombatPlan } from "../game/combatPlan";

const point = (cell: number) => ({ x: cell % 8 + .5, y: Math.floor(cell / 8) + .5 });
const clamp = (value: number) => Math.max(0, Math.min(1, value));
const friction = (t: number) => (1 - Math.exp(-5 * t)) / (1 - Math.exp(-5));
const smooth = (t: number) => t * t * (3 - 2 * t);

function Impact({ cell, result }: { cell: number; result: CombatPlan["result"] }) {
  if (result === "none") return null;
  const offset = result === "miss" ? { x: .28, y: -.22 } : { x: 0, y: 0 };
  const style = { left: `${(cell % 8 + .5 + offset.x) * 12.5}%`, top: `${(Math.floor(cell / 8) + .5 + offset.y) * 12.5}%` };
  if (result === "ward") return <span className="combat-ward" style={style} aria-hidden="true"><span>✝</span></span>;
  return <span className={`combat-impact combat-impact-${result}`} style={style} aria-hidden="true" />;
}

function ShotgunEffects({ plan, elapsed }: { plan: CombatPlan; elapsed: number }) {
  const shot = plan.shotgun;
  if (!shot) return null;
  const origin = point(shot.from);
  const toward = point(shot.toward);
  const angle = Math.atan2(toward.y - origin.y, toward.x - origin.x);
  const muzzle = { x: origin.x + Math.cos(angle) * .22, y: origin.y + Math.sin(angle) * .22 };
  const age = elapsed - combatTiming.shotgunReady;
  const flight = clamp(age / combatTiming.shotgunFlight);
  const impacting = elapsed >= plan.impact && elapsed < plan.impact + combatTiming.impactFlash;
  return <>
    {shot.area.map((cell) => <span key={cell} className="combat-shotgun-area" data-cell={cell}
      style={{ left: `${cell % 8 * 12.5}%`, top: `${Math.floor(cell / 8) * 12.5}%`,
        animationDuration: `${plan.duration}ms` }} />)}
    <svg viewBox="0 0 8 8" preserveAspectRatio="none" className="combat-projectiles combat-shotgun">
      {shot.rays.flatMap((ray, rayIndex) => {
        const lastCell = ray.cells.at(-1);
        if (lastCell === undefined || age < 0 || age > combatTiming.shotgunFlight) return [];
        const endpoint = point(lastCell);
        if (shot.hits.some((hit) => hit.cell === lastCell && hit.result === "miss")) {
          endpoint.x += .28;
          endpoint.y -= .22;
        }
        const dx = endpoint.x - origin.x;
        const dy = endpoint.y - origin.y;
        const length = Math.hypot(dx, dy);
        // The engine supplies each ray's endpoint, including its first blocker.
        // Small offsets stay within that endpoint's square. Every pellet starts
        // at the same muzzle, so the spread grows visibly with travel distance.
        return [-.21, -.07, .07, .21].map((spread, pellet) => {
          const end = { x: endpoint.x - dy / length * spread, y: endpoint.y + dx / length * spread };
          const back = Math.max(0, flight - .12);
          return <g className="combat-shotgun-pellet" data-ray={rayIndex} key={`${rayIndex}-${pellet}`}>
            <line x1={muzzle.x + (end.x - muzzle.x) * back} y1={muzzle.y + (end.y - muzzle.y) * back}
              x2={muzzle.x + (end.x - muzzle.x) * flight} y2={muzzle.y + (end.y - muzzle.y) * flight} />
            <circle cx={muzzle.x + (end.x - muzzle.x) * flight} cy={muzzle.y + (end.y - muzzle.y) * flight} r=".035" />
          </g>;
        });
      })}
      {age >= 0 && age < 100 && <g className="combat-shotgun-muzzle"
        transform={`translate(${muzzle.x} ${muzzle.y}) rotate(${angle * 180 / Math.PI})`}>
        <ellipse className="combat-shotgun-flash" cx=".04" cy="0" rx=".36" ry=".31" />
        <path className="combat-shotgun-flame" d="M -.11 -.11 L .13 -.15 L .29 -.30 L .28 -.10 L .66 0 L .28 .10 L .29 .30 L .13 .15 L -.11 .11 Z" />
        <ellipse className="combat-shotgun-core" cx=".12" cy="0" rx=".22" ry=".075" />
      </g>}
    </svg>
    {impacting && shot.hits.map((hit) => <Impact key={hit.piece_id} cell={hit.cell} result={hit.result} />)}
    {impacting && plan.victims?.map((victim) => <span key={victim.id} className="combat-vanish-piece"
      style={{ left: `${victim.cell % 8 * 12.5}%`, top: `${Math.floor(victim.cell / 8) * 12.5}%` }}>
      <PieceImage pieceKey={victim.key} owner={victim.owner} />
    </span>)}
  </>;
}

function ChainEffects({ plan, elapsed }: { plan: CombatPlan; elapsed: number }) {
  const chain = plan.chain;
  if (!chain) return null;
  const boss = point(plan.origin);
  const from = point(chain.from);
  const to = point(chain.to);
  const extension = smooth(clamp(elapsed / combatTiming.chainExtend));
  const pull = smooth(clamp((elapsed - combatTiming.chainExtend - combatTiming.chainCatch) / combatTiming.chainPull));
  const moving = { x: from.x + (to.x - from.x) * pull, y: from.y + (to.y - from.y) * pull };
  const end = { x: boss.x + (moving.x - boss.x) * extension, y: boss.y + (moving.y - boss.y) * extension };
  const dx = end.x - boss.x;
  const dy = end.y - boss.y;
  const length = Math.hypot(dx, dy);
  const angle = Math.atan2(dy, dx) * 180 / Math.PI;
  const count = Math.max(1, Math.ceil(length / .15));
  const caught = elapsed >= combatTiming.chainExtend;
  const fade = 1 - clamp((elapsed - plan.impact) / combatTiming.chainFade);
  return <>
    {!plan.impactDone && <span className="combat-moving-piece combat-chain-piece"
      data-piece-id={chain.piece.id} data-from={chain.from} data-to={chain.to}
      style={{ left: `${(moving.x - .5) * 12.5}%`, top: `${(moving.y - .5) * 12.5}%` }}>
      <PieceImage pieceKey={chain.piece.key} owner={chain.piece.owner} />
    </span>}
    <svg viewBox="0 0 8 8" preserveAspectRatio="none" className="combat-projectiles combat-chain"
      data-from={plan.origin} data-target={chain.from} data-to={chain.to} style={{ opacity: fade }}>
      {Array.from({ length: count }, (_, index) => {
        const t = (index + .5) / count;
        return <ellipse key={index} className={`combat-chain-link${index % 2 ? " edge" : ""}`}
          cx="0" cy="0" rx=".105" ry={index % 2 ? ".033" : ".065"}
          transform={`translate(${boss.x + dx * t} ${boss.y + dy * t}) rotate(${angle})`} />;
      })}
      <path className="combat-chain-hook" transform={`translate(${end.x} ${end.y}) rotate(${angle})`}
        d="M -.1 -.1 L .08 -.1 Q .22 0 .08 .13 Q -.03 .18 -.08 .08" />
      {caught && <ellipse className="combat-chain-catch" cx={moving.x} cy={moving.y} rx=".30" ry=".36" />}
    </svg>
  </>;
}

export function CombatEffects({ plan }: { plan: CombatPlan | null }) {
  const [elapsed, setElapsed] = useState(0);
  useEffect(() => {
    if (!plan) return;
    let frame = 0;
    const start = performance.now();
    const tick = (now: number) => {
      setElapsed(now - start);
      if (now - start < plan.duration) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [plan?.sequence]);
  if (!plan) return null;
  const a = point(plan.origin);
  const b = point(plan.target);
  const moving = plan.kind === "move" || plan.kind === "return";
  let motion: { x: number; y: number } | null = null;
  if (moving && (plan.kind === "return" || elapsed < plan.impact)) {
    const t = elapsed < plan.impact ? clamp(elapsed / combatTiming.move)
      : 1 - friction(clamp((elapsed - plan.impact) / combatTiming.return));
    motion = { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t };
  }
  const projectiles = plan.kind === "tracer" ? Array.from({ length: plan.count }, (_, index) => {
    const age = elapsed - index * plan.interval;
    if (age < 0 || age > combatTiming.projectile) return null;
    const t = age / combatTiming.projectile;
    const tx = plan.result === "miss" ? b.x + .28 : b.x;
    const ty = plan.result === "miss" ? b.y - .22 : b.y;
    const x = a.x + (tx - a.x) * t;
    const y = a.y + (ty - a.y) * t;
    const back = Math.max(0, t - .13);
    return <g key={index} className="combat-tracer">
      <line x1={a.x + (tx - a.x) * back} y1={a.y + (ty - a.y) * back} x2={x} y2={y} />
      <circle cx={x} cy={y} r=".055" />
    </g>;
  }) : null;
  const muzzle = plan.kind === "tracer" && Array.from({ length: plan.count }, (_, i) => i)
    .some((i) => elapsed >= i * plan.interval && elapsed < i * plan.interval + 55);
  const mortarT = clamp(elapsed / combatTiming.mortar);
  const mortarY = a.y + (b.y - a.y) * mortarT - 4 * 1.35 * mortarT * (1 - mortarT);
  const impactVisible = elapsed >= plan.impact && elapsed < plan.impact + combatTiming.impactFlash;
  return <div className="combat-effects" aria-hidden="true">
    {plan.kind === "shotgun" && <ShotgunEffects plan={plan} elapsed={elapsed} />}
    {plan.kind === "chain" && <ChainEffects plan={plan} elapsed={elapsed} />}
    {motion && <span className="combat-moving-piece" style={{ left: `${(motion.x - .5) * 12.5}%`, top: `${(motion.y - .5) * 12.5}%` }}>
      <PieceImage pieceKey={plan.actor.key} owner={plan.actor.owner} />
    </span>}
    {(plan.kind === "tracer" || plan.kind === "mortar") && <svg viewBox="0 0 8 8" preserveAspectRatio="none" className="combat-projectiles">
      {projectiles}
      {muzzle && <circle className="combat-muzzle" cx={a.x} cy={a.y} r=".17" />}
      {plan.kind === "mortar" && elapsed <= combatTiming.mortar && <g className="combat-mortar-shell">
        <circle cx={a.x + (b.x - a.x) * mortarT} cy={mortarY} r=".105" />
        <circle cx={a.x + (b.x - a.x) * mortarT} cy={mortarY} r=".19" opacity=".25" />
      </g>}
    </svg>}
    {impactVisible && !["blast", "nuke"].includes(plan.kind) && <Impact cell={plan.target} result={plan.result} />}
    {impactVisible && plan.victim && <span className="combat-vanish-piece" style={{ left: `${(b.x - .5) * 12.5}%`, top: `${(b.y - .5) * 12.5}%` }}>
      <PieceImage pieceKey={plan.victim.key} owner={plan.victim.owner} />
    </span>}
    {impactVisible && plan.kind === "mortar" && <span className="combat-mortar-impact" style={{ left: `${b.x * 12.5}%`, top: `${b.y * 12.5}%` }} />}
    {(plan.kind === "blast" || plan.kind === "nuke") && <>
      {plan.blast.map((cell) => {
        const distance = Math.max(Math.abs(cell % 8 - plan.origin % 8),
          Math.abs(Math.floor(cell / 8) - Math.floor(plan.origin / 8)));
        return <span key={cell} className="combat-blast-cell" style={{
          left: `${cell % 8 * 12.5}%`, top: `${Math.floor(cell / 8) * 12.5}%`,
          animationDelay: `${distance * 35}ms`, animationDuration: plan.kind === "nuke" ? "400ms" : "260ms",
        }} />;
      })}
      <span className={`combat-explosion ${plan.kind}`} style={{ "--center-x": `${(a.x) * 12.5}%`, "--center-y": `${(a.y) * 12.5}%` } as CSSProperties}>
        <span className="combat-explosion-flash" />
        <span className="combat-shockwave" />
        {plan.kind === "nuke" && <span className="combat-mushroom"><span className="combat-mushroom-stem" /><span className="combat-mushroom-cap" /></span>}
      </span>
    </>}
  </div>;
}
