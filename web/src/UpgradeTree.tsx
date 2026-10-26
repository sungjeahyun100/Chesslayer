import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { PieceImage } from "./PieceImage";
import { buildUpgradeTree } from "./upgradeTree";
import type { RestOption, UpgradePath } from "./types";

export function UpgradeTree({ piece, paths, acquired, onSelect, onConfirm, completed, selectionOnly = false, inlineConfirm = false }: {
  piece: RestOption;
  paths: UpgradePath[];
  acquired: number[];
  onSelect: (kind: number | undefined) => void;
  onConfirm: (target: number) => void;
  completed: boolean;
  selectionOnly?: boolean;
  inlineConfirm?: boolean;
}) {
  const tree = useMemo(() => buildUpgradeTree(paths, piece), [paths, piece]);
  const viewport = useRef<HTMLDivElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const [openKind, setOpenKind] = useState<number | null>(null);
  const confirming = useRef(false);
  const dragged = useRef(false);
  const active = tree.nodes.find((n) => n.kind === openKind) ?? tree.nodes.find((n) => n.kind === piece.kind)!;
  const drag = useRef<{ x: number; y: number; left: number; top: number } | null>(null);
  const [zoom, setZoom] = useState<number | null>(null);
  const [fit, setFit] = useState(1);
  const scale = zoom ?? fit;
  useEffect(() => { if (completed) { setOpenKind(null); confirming.current = false; } }, [completed]);
  useLayoutEffect(() => {
    if (openKind === null || !popup.current || !viewport.current) return;
    const panel = popup.current;
    const anchor = [...viewport.current.querySelectorAll<HTMLButtonElement>(".skill-node")]
      .find((node) => Number(node.dataset.kind) === openKind);
    if (!anchor) return;
    let frame = 0;
    const position = () => {
      frame = 0;
      const node = anchor.querySelector<HTMLElement>(".skill-node-orb")!.getBoundingClientRect();
      const area = viewport.current!.getBoundingClientRect();
      const visible = node.right > area.left && node.left < area.right && node.bottom > area.top && node.top < area.bottom
        && node.right > 0 && node.left < innerWidth && node.bottom > 0 && node.top < innerHeight;
      panel.style.visibility = visible ? "visible" : "hidden";
      if (!visible) return;
      const width = panel.offsetWidth;
      const height = panel.offsetHeight;
      const margin = 4;
      const overlapAtJoin = 7;
      const obstacles = [...viewport.current!.querySelectorAll<HTMLElement>(".skill-node-orb")]
        .filter((other) => other !== anchor.querySelector(".skill-node-orb")).map((other) => other.getBoundingClientRect())
        .filter((other) => other.right > area.left && other.left < area.right && other.bottom > area.top && other.top < area.bottom);
      const overlap = (a: DOMRect, x: number, y: number) =>
        Math.max(0, Math.min(a.right, x + width) - Math.max(a.left, x))
        * Math.max(0, Math.min(a.bottom, y + height) - Math.max(a.top, y));
      const y = Math.max(margin, Math.min(innerHeight - height - margin, node.top + node.height / 2 - height / 2));
      const candidates = (["right", "left"] as const).map((side) => {
        const x = side === "right" ? node.right - overlapAtJoin : node.left - width + overlapAtJoin;
        const offscreen = Math.max(0, margin - x) + Math.max(0, x + width + margin - innerWidth);
        const outsideMap = Math.max(0, area.left - x) + Math.max(0, x + width - area.right);
        const collision = obstacles.reduce((sum, other) => sum + overlap(other, x, y), 0);
        return { side, x, score: offscreen * 1000 + outsideMap * 15 + collision * 3 + (side === "left" ? 1 : 0) };
      });
      const { x, side } = candidates.reduce((best, candidate) => candidate.score < best.score ? candidate : best);
      panel.style.left = `${x}px`;
      panel.style.top = `${y}px`;
      panel.dataset.side = side;
    };
    const schedule = () => { if (!frame) frame = requestAnimationFrame(position); };
    position();
    const observer = new ResizeObserver(schedule);
    observer.observe(panel);
    observer.observe(viewport.current);
    window.addEventListener("resize", schedule);
    window.addEventListener("scroll", schedule, true);
    return () => {
      if (frame) cancelAnimationFrame(frame);
      observer.disconnect();
      window.removeEventListener("resize", schedule);
      window.removeEventListener("scroll", schedule, true);
    };
  }, [openKind, scale, tree.width, tree.height]);
  const closePopup = () => { setOpenKind(null); onSelect(undefined); };
  const chooseNode = (kind: number) => {
    if (openKind === kind) { closePopup(); return; }
    setOpenKind(kind);
    onSelect(kind);
  };
  const confirm = (kind: number) => {
    if (confirming.current) return;
    confirming.current = true;
    setOpenKind(null);
    onConfirm(kind);
    window.setTimeout(() => { confirming.current = false; }, 500);
  };
  useEffect(() => {
    const el = viewport.current!;
    const resize = () => setFit(Math.min(1, el.clientWidth / tree.width, el.clientHeight / tree.height));
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(el);
    return () => observer.disconnect();
  }, [tree.width, tree.height]);

  // A target with several incoming edges is one node; only its actual entry
  // path is lit. The existing engine treats those edges as alternative routes.
  const completedEdges = new Set(tree.edges.filter((edge) => {
    const index = acquired.indexOf(edge.to);
    const parents = tree.edges.filter((e) => e.to === edge.to);
    return index >= 0 && (parents.length === 1 || acquired[index - 1] === edge.from);
  }));
  const attained = new Set([piece.kind, ...acquired, ...[...completedEdges].map((e) => e.from)]);
  const status = (kind: number) => attained.has(kind)
    ? "acquired"
    : piece.can_upgrade && paths.some((p) => p.from === piece.kind && p.to === kind && p.unlocked)
      ? "available" : "locked";
  const statusText = (kind: number) => kind === piece.kind ? "현재 기물"
    : acquired.includes(kind) ? "획득 완료"
      : status(kind) === "available" ? "선택 가능"
        : tree.nodes.find((n) => n.kind === kind)?.incoming.length ? "잠김" : "기본 상태";
  const direct = active.incoming.find((p) => p.from === piece.kind);
  const available = !completed && status(active.kind) === "available" && !!direct;

  return (
    <section className="skill-tree" aria-label={`${piece.name} 강화 트리`}>
      <div className="skill-tree-heading">
        <h3>강화 경로</h3>
      </div>
      <div className="skill-map">
        <div className="skill-legend" aria-label="노드 상태 안내">
          <span className="acquired">◆ 획득</span><span className="available">◇ 선택 가능</span><span className="locked">◈ 잠김</span><span className="inspecting">◎ 살펴보는 중</span>
        </div>
        <div className="skill-viewport" ref={viewport} tabIndex={0} role="region" aria-label="강화 경로 지도. 확대 후 스크롤 또는 빈 공간을 드래그하여 이동"
          onClick={(e) => {
            if (dragged.current) { dragged.current = false; return; }
            if (!(e.target as Element).closest(".skill-node, .skill-node-popup")) closePopup();
          }}
          onPointerDown={(e) => {
            if (e.pointerType !== "mouse" || e.button !== 0 || (e.target as Element).closest("button")) return;
            const el = e.currentTarget;
            dragged.current = false;
            drag.current = { x: e.clientX, y: e.clientY, left: el.scrollLeft, top: el.scrollTop };
            el.setPointerCapture(e.pointerId);
            el.classList.add("dragging");
          }}
          onPointerMove={(e) => {
            if (!drag.current) return;
            if (Math.abs(e.clientX - drag.current.x) + Math.abs(e.clientY - drag.current.y) > 4) dragged.current = true;
            e.currentTarget.scrollLeft = drag.current.left - e.clientX + drag.current.x;
            e.currentTarget.scrollTop = drag.current.top - e.clientY + drag.current.y;
          }}
          onLostPointerCapture={(e) => { drag.current = null; e.currentTarget.classList.remove("dragging"); }}
          onPointerUp={(e) => { if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId); }}
        >
          <div className="skill-map-center">
            <div className="skill-map-size" style={{ width: tree.width * scale, height: tree.height * scale }}>
              <div className="skill-map-stage" style={{ width: tree.width, height: tree.height, transform: `scale(${scale})` }}>
                <svg className="skill-connections" width={tree.width} height={tree.height} aria-hidden="true">
                  {tree.edges.map((edge) => {
                    // Only actual rest commands activate a path. A promoted piece
                    // must not claim an upgrade from the ordinary piece of its new kind.
                    const done = completedEdges.has(edge);
                    const state = done ? "acquired" : piece.can_upgrade && edge.from === piece.kind && edge.unlocked ? "available" : "locked";
                    return <g key={`${edge.from}-${edge.to}`} className={`skill-connection ${state}`} data-from={edge.from} data-to={edge.to}>
                      <path className="skill-line-bed" d={edge.d} /><path className="skill-line" d={edge.d} />
                    </g>;
                  })}
                </svg>
                {tree.nodes.map((node) => (
                  <button key={node.kind} type="button" data-kind={node.kind}
                    className={`skill-node ${status(node.kind)} ${!node.incoming.length ? "root-node" : ""}`}
                    style={{ left: node.x, top: node.y }}
                    aria-pressed={openKind === node.kind}
                    aria-label={`${node.name} · ${statusText(node.kind)} · 상세 보기`}
                    title={`${node.name} · ${statusText(node.kind)}${node.incoming.flatMap((p) => p.required_pieces.map((r) => `\n${r.name}: ${r.owned ? "보유" : "미보유"}`)).join("")}`}
                    aria-controls={openKind === node.kind && available ? "skill-node-popup" : undefined}
                    aria-expanded={openKind === node.kind && available}
                    onClick={() => chooseNode(node.kind)}
                    onFocus={(e) => e.currentTarget.scrollIntoView({ block: "nearest", inline: "nearest" })}
                  >
                    <span className="skill-node-orb"><PieceImage pieceKey={node.key} /><span className="skill-node-mark">{status(node.kind) === "acquired" ? "◆" : !node.incoming.length ? "·" : status(node.kind) === "available" ? "+" : "◈"}</span></span>
                    <strong style={{ fontSize: 13 / Math.max(0.6, Math.min(1, scale)) }}>{node.name}</strong>
                    <small style={{ fontSize: 10 / Math.max(0.6, Math.min(1, scale)) }}>{statusText(node.kind)}</small>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
        <div className="skill-map-toolbar">
          <div role="group" aria-label="트리 확대 및 축소">
            <button aria-label="트리 축소" disabled={scale <= 0.35} onClick={() => setZoom(Math.max(0.35, scale - 0.2))}>−</button>
            <output aria-live="polite">{Math.round(scale * 100)}%</output>
            <button aria-label="트리 확대" disabled={scale >= 1.8} onClick={() => setZoom(Math.min(1.8, scale + 0.2))}>+</button>
            <button onClick={() => { setZoom(null); viewport.current?.scrollTo(0, 0); }}>전체 보기</button>
          </div>
        </div>
      </div>
      {!inlineConfirm && openKind !== null && available && createPortal(<div key={openKind} id="skill-node-popup" ref={popup} className="skill-node-popup" role="group" aria-label={`${active.name} 강화 동작`}>
        <button type="button" className="primary" onClick={() => confirm(active.kind)}>{selectionOnly ? "경로 선택" : "강화 확정"}</button>
      </div>, document.body)}
    </section>
  );
}
