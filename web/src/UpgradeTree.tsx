import { useEffect, useMemo, useRef, useState } from "react";
import { PieceImage } from "./PieceImage";
import { buildUpgradeTree } from "./upgradeTree";
import type { RestOption, UpgradePath } from "./types";

export function UpgradeTree({ piece, paths, acquired, selected, onSelect, onConfirm, onCancel, completed }: {
  piece: RestOption;
  paths: UpgradePath[];
  acquired: number[];
  selected?: number;
  onSelect: (kind: number) => void;
  onConfirm: (target: number) => void;
  onCancel: () => void;
  completed: boolean;
}) {
  const tree = useMemo(() => buildUpgradeTree(paths, piece), [paths, piece]);
  const active = tree.nodes.find((n) => n.kind === selected) ?? tree.nodes.find((n) => n.kind === piece.kind)!;
  const viewport = useRef<HTMLDivElement>(null);
  const drag = useRef<{ x: number; y: number; left: number; top: number } | null>(null);
  const [zoom, setZoom] = useState<number | null>(null);
  const [fit, setFit] = useState(1);
  const scale = zoom ?? fit;
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
    : piece.can_upgrade && paths.some((p) => p.from === piece.kind && p.to === kind)
      ? "available" : "locked";
  const statusText = (kind: number) => kind === piece.kind ? "현재 기물"
    : acquired.includes(kind) ? "획득 완료"
      : status(kind) === "available" ? "선택 가능"
        : tree.nodes.find((n) => n.kind === kind)?.incoming.length ? "잠김" : "기본 상태";
  const direct = active.incoming.find((p) => p.from === piece.kind);
  const available = status(active.kind) === "available";
  const achieved = status(active.kind) === "acquired";
  const reachable = new Set([piece.kind]);
  for (let i = 0; i < tree.nodes.length; i++) {
    for (const edge of tree.edges) if (reachable.has(edge.from)) reachable.add(edge.to);
  }
  const reason = achieved
    ? active.kind === piece.kind ? "현재 기물에 적용된 능력입니다." : "이미 획득한 강화입니다."
    : !active.incoming.length ? "이 계열의 시작점입니다. 이전 종류로 되돌리는 강화는 없습니다."
        : !reachable.has(active.kind) ? `현재 ${piece.name}에서 이 가지로 변경하는 강화 경로는 없습니다.`
          : completed ? "이번 쉼터의 행동을 사용했습니다. 다음 쉼터에서 다시 살펴보세요."
          : `선행 기물 ${active.incoming.map((p) => p.from_name).join(" 또는 ")} 상태가 필요합니다.`;

  return (
    <section className="skill-tree" aria-label={`${piece.name} 강화 트리`}>
      <div className="skill-tree-heading">
        <div><span className="eyebrow">PATHS OF ASCENSION</span><h3>기물의 성장 경로</h3></div>
        <span className="skill-count">{tree.nodes.filter((n) => n.incoming.length).length}개의 강화</span>
      </div>
      <div className="skill-map">
        <div className="skill-legend" aria-label="노드 상태 안내">
          <span className="acquired">◆ 획득</span><span className="available">◇ 선택 가능</span><span className="locked">◈ 잠김</span><span className="inspecting">◎ 살펴보는 중</span>
        </div>
        <div className="skill-viewport" ref={viewport} tabIndex={0} role="region" aria-label="강화 경로 지도. 확대 후 스크롤 또는 빈 공간을 드래그하여 이동"
          onPointerDown={(e) => {
            if (e.pointerType !== "mouse" || e.button !== 0 || (e.target as Element).closest("button")) return;
            const el = e.currentTarget;
            drag.current = { x: e.clientX, y: e.clientY, left: el.scrollLeft, top: el.scrollTop };
            el.setPointerCapture(e.pointerId);
            el.classList.add("dragging");
          }}
          onPointerMove={(e) => {
            if (!drag.current) return;
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
                    const state = done ? "acquired" : piece.can_upgrade && edge.from === piece.kind ? "available" : "locked";
                    return <g key={`${edge.from}-${edge.to}`} className={`skill-connection ${state}`} data-from={edge.from} data-to={edge.to}>
                      <path className="skill-line-bed" d={edge.d} /><path className="skill-line" d={edge.d} />
                    </g>;
                  })}
                </svg>
                {tree.nodes.map((node) => (
                  <button key={node.kind} type="button" data-kind={node.kind}
                    className={`skill-node ${status(node.kind)} ${!node.incoming.length ? "root-node" : ""}`}
                    style={{ left: node.x, top: node.y }}
                    aria-pressed={active.kind === node.kind}
                    aria-label={`${node.name} · ${statusText(node.kind)} · 상세 보기`}
                    aria-controls="skill-inspector"
                    onClick={() => onSelect(node.kind)}
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
          <span>노드를 눌러 탐색 · 드래그로 이동</span>
          <div role="group" aria-label="트리 확대 및 축소">
            <button aria-label="트리 축소" disabled={scale <= 0.35} onClick={() => setZoom(Math.max(0.35, scale - 0.2))}>−</button>
            <output aria-live="polite">{Math.round(scale * 100)}%</output>
            <button aria-label="트리 확대" disabled={scale >= 1.8} onClick={() => setZoom(Math.min(1.8, scale + 0.2))}>+</button>
            <button onClick={() => { setZoom(null); viewport.current?.scrollTo(0, 0); }}>전체 보기</button>
          </div>
        </div>
      </div>
      <div id="skill-inspector" className="skill-inspector" aria-live="polite">
        <div className="skill-inspector-title"><h3>{active.name}</h3><span className={`skill-state ${status(active.kind)}`}>{statusText(active.kind)}</span></div>
        <p className="skill-effect">{active.description}</p>
        {active.incoming.length > 0 && <div className="skill-requirements">
          <strong>비용 · 선행 조건{active.incoming.length > 1 ? " (진입 경로별)" : ""}</strong>
          {active.incoming.map((p) => <p key={p.from}><span>{p.from_name} → {p.to_name}</span>{p.requirement}</p>)}
        </div>}
        {!tree.edges.length && <p className="rest-note">이 기물에 정의된 강화 경로가 없습니다.</p>}
        {available && direct ? <div className="skill-purchase">
          <p>{piece.name} → {active.name}<br /><small>확정하면 쉼터 행동 1회를 사용합니다.</small></p>
          <div className="rest-confirm">
            <button className="primary" onClick={() => onConfirm(active.kind)}>강화 확정</button>
            <button onClick={onCancel}>취소</button>
          </div>
        </div> : <p className="skill-lock-reason">{reason}</p>}
      </div>
    </section>
  );
}
