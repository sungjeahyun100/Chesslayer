import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { PieceImage } from "./PieceImage";
import { buildUpgradeTree } from "./upgradeTree";
import type { RestOption, UpgradePath } from "./types";

type Bounds = { left: number; top: number; right: number; bottom: number };
type ViewMode = "default" | "overview" | "custom";
const MAP_PADDING = 12;
const DEFAULT_SCALE = 0.56;

function unionBounds(boxes: Bounds[]): Bounds {
  return { left: Math.min(...boxes.map((box) => box.left)), top: Math.min(...boxes.map((box) => box.top)),
    right: Math.max(...boxes.map((box) => box.right)), bottom: Math.max(...boxes.map((box) => box.bottom)) };
}

export function UpgradeTree({ piece, paths, acquired, onSelect, onConfirm, completed, selectionOnly = false, inlineConfirm = false, readOnly = false, instant = false, selectedKind }: {
  piece: RestOption;
  paths: UpgradePath[];
  acquired: number[];
  onSelect: (kind: number | undefined) => void;
  onConfirm: (target: number) => void;
  completed: boolean;
  selectionOnly?: boolean;
  inlineConfirm?: boolean;
  readOnly?: boolean;
  instant?: boolean;
  selectedKind?: number;
}) {
  const tree = useMemo(() => buildUpgradeTree(paths, piece), [paths, piece]);
  const viewport = useRef<HTMLDivElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const [openKind, setOpenKind] = useState<number | null>(null);
  const confirming = useRef(false);
  const dragged = useRef(false);
  const active = tree.nodes.find((n) => n.kind === openKind) ?? tree.nodes.find((n) => n.kind === piece.kind)!;
  const drag = useRef<{ x: number; y: number; left: number; top: number } | null>(null);
  const focusKinds = useMemo(() => {
    const kinds = [piece.kind];
    const nearest = (from: number) => tree.edges.filter((edge) => edge.from === from)
      .sort((a, b) => Math.abs(tree.nodes.find((node) => node.kind === a.to)!.y - tree.nodes.find((node) => node.kind === from)!.y)
        - Math.abs(tree.nodes.find((node) => node.kind === b.to)!.y - tree.nodes.find((node) => node.kind === from)!.y) || a.to - b.to)[0];
    const parent = tree.edges.find((edge) => edge.to === piece.kind);
    if (parent) kinds.push(parent.from);
    const child = nearest(piece.kind);
    if (child) { kinds.push(child.to); const grandchild = nearest(child.to); if (grandchild) kinds.push(grandchild.to); }
    return new Set(kinds);
  }, [tree, piece.kind]);
  const initial = useMemo(() => {
    const nodeBox = (node: typeof tree.nodes[number]): Bounds => ({ left: node.x - 88, top: node.y - 34, right: node.x + 88, bottom: node.y + 100 });
    return { content: unionBounds(tree.nodes.map(nodeBox)),
      focus: unionBounds(tree.nodes.filter((node) => focusKinds.has(node.kind)).map(nodeBox)) };
  }, [tree, focusKinds]);
  const [display, setDisplay] = useState(initial);
  const [viewSize, setViewSize] = useState({ width: 0, height: 0 });
  const [viewMode, setViewMode] = useState<ViewMode>("default");
  const [customZoom, setCustomZoom] = useState(1);
  const pendingCenter = useRef<{ x: number; y: number } | null>(null);
  const { content, focus } = display;
  const contentWidth = content.right - content.left;
  const contentHeight = content.bottom - content.top;
  const overviewScale = viewSize.width ? Math.min(1.35, (viewSize.width - MAP_PADDING * 2) / contentWidth,
    (viewSize.height - MAP_PADDING * 2) / contentHeight) : 1;
  const scale = viewMode === "overview" ? overviewScale : viewMode === "default" ? DEFAULT_SCALE : customZoom;
  const canvasWidth = contentWidth * scale + MAP_PADDING * 2;
  const canvasHeight = contentHeight * scale + MAP_PADDING * 2;
  const centerAt = (point: { x: number; y: number }, nextScale = scale) => {
    const el = viewport.current;
    if (!el) return;
    const offsetX = Math.max(0, (el.clientWidth - (contentWidth * nextScale + MAP_PADDING * 2)) / 2);
    const offsetY = Math.max(0, (el.clientHeight - (contentHeight * nextScale + MAP_PADDING * 2)) / 2);
    el.scrollTo({ left: (point.x - content.left) * nextScale + MAP_PADDING + offsetX - el.clientWidth / 2,
      top: (point.y - content.top) * nextScale + MAP_PADDING + offsetY - el.clientHeight / 2 });
  };
  const changeZoom = (next: number) => {
    const el = viewport.current;
    if (el) {
      const offsetX = Math.max(0, (el.clientWidth - canvasWidth) / 2);
      const offsetY = Math.max(0, (el.clientHeight - canvasHeight) / 2);
      pendingCenter.current = { x: content.left + (el.scrollLeft + el.clientWidth / 2 - offsetX - MAP_PADDING) / scale,
        y: content.top + (el.scrollTop + el.clientHeight / 2 - offsetY - MAP_PADDING) / scale };
    }
    setCustomZoom(next);
    setViewMode("custom");
  };
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
    if (instant) { onConfirm(kind); return; }
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
  useLayoutEffect(() => {
    const el = viewport.current!;
    const stage = el.querySelector<HTMLElement>(".skill-map-stage")!;
    let mounted = true;
    const measure = () => {
      if (!mounted) return;
      setViewSize((previous) => previous.width === el.clientWidth && previous.height === el.clientHeight
        ? previous : { width: el.clientWidth, height: el.clientHeight });
      const origin = stage.getBoundingClientRect();
      const box = (element: Element): Bounds => {
        const rect = element.getBoundingClientRect();
        const round = (value: number) => Math.round(value * 10) / 10;
        return { left: round((rect.left - origin.left) / scale), top: round((rect.top - origin.top) / scale),
          right: round((rect.right - origin.left) / scale), bottom: round((rect.bottom - origin.top) / scale) };
      };
      const nodes = [...stage.querySelectorAll<HTMLElement>(".skill-node")];
      const next = { content: unionBounds([...nodes.map(box), ...[...stage.querySelectorAll(".skill-line-bed")].map(box)]),
        focus: unionBounds(nodes.filter((node) => focusKinds.has(Number(node.dataset.kind))).map(box)) };
      setDisplay((previous) => {
        const close = (a: Bounds, b: Bounds) => (Object.keys(a) as (keyof Bounds)[]).every((key) => Math.abs(a[key] - b[key]) < 0.5);
        return close(previous.content, next.content) && close(previous.focus, next.focus) ? previous : next;
      });
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    for (const node of stage.querySelectorAll(".skill-node")) observer.observe(node);
    void document.fonts.ready.then(measure);
    return () => { mounted = false; observer.disconnect(); };
  }, [tree, focusKinds, scale]);
  useLayoutEffect(() => {
    if (viewMode === "overview") { viewport.current?.scrollTo(0, 0); return; }
    if (pendingCenter.current) {
      centerAt(pendingCenter.current);
      pendingCenter.current = null;
    } else if (viewMode === "default") centerAt({ x: (focus.left + focus.right) / 2, y: (focus.top + focus.bottom) / 2 });
  // Measurements and the current piece define the viewport target.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [piece.kind, viewMode, scale, content, focus, viewSize]);

  // A target with several incoming edges is one node; only its actual entry
  // path is lit. The existing engine treats those edges as alternative routes.
  const completedEdges = new Set(tree.edges.filter((edge) => {
    const index = acquired.indexOf(edge.to);
    const parents = tree.edges.filter((e) => e.to === edge.to);
    return index >= 0 && (parents.length === 1 || acquired[index - 1] === edge.from);
  }));
  const attained = new Set([piece.kind, ...acquired, ...[...completedEdges].map((e) => e.from)]);
  const status = (kind: number) => readOnly ? "available" : instant ? kind === piece.kind ? "acquired" : "available" : attained.has(kind)
    ? "acquired"
    : piece.can_upgrade && paths.some((p) => p.from === piece.kind && p.to === kind && p.unlocked)
      ? "available" : "locked";
  const statusText = (kind: number) => readOnly ? "정보 열람" : kind === piece.kind ? "현재 기물" : instant ? "즉시 변경"
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
        {!readOnly && <div className="skill-legend" aria-label="노드 상태 안내">
          {instant ? <><span className="acquired">◆ 현재 적용</span><span className="available">◇ 클릭 즉시 적용</span></>
            : <><span className="acquired">◆ 획득</span><span className="available">◇ 선택 가능</span><span className="locked">◈ 잠김</span><span className="inspecting">◎ 살펴보는 중</span></>}
        </div>}
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
            <div className="skill-map-size" style={{ width: canvasWidth, height: canvasHeight }}>
              <div className="skill-map-stage" style={{ width: tree.width, height: tree.height,
                transform: `translate(${MAP_PADDING - content.left * scale}px, ${MAP_PADDING - content.top * scale}px) scale(${scale})` }}>
                <svg className="skill-connections" width={tree.width} height={tree.height} aria-hidden="true">
                  {tree.edges.map((edge) => {
                    // Only actual rest commands activate a path. A promoted piece
                    // must not claim an upgrade from the ordinary piece of its new kind.
                    const done = completedEdges.has(edge);
                    const state = readOnly ? "available" : done ? "acquired" : instant ? "available" : piece.can_upgrade && edge.from === piece.kind && edge.unlocked ? "available" : "locked";
                    return <g key={`${edge.from}-${edge.to}`} className={`skill-connection ${state}`} data-from={edge.from} data-to={edge.to}>
                      <path className="skill-line-bed" d={edge.d} /><path className="skill-line" d={edge.d} />
                    </g>;
                  })}
                </svg>
                {tree.nodes.map((node) => (
                  <button key={node.kind} type="button" data-kind={node.kind}
                    className={`skill-node ${status(node.kind)} ${!node.incoming.length ? "root-node" : ""} ${selectedKind === node.kind ? "sandbox-chosen" : ""}`}
                    style={{ left: node.x, top: node.y }}
                    aria-pressed={selectedKind === undefined ? openKind === node.kind : selectedKind === node.kind}
                    aria-label={`${node.name} · ${statusText(node.kind)}${instant ? " · 클릭하여 적용" : " · 상세 보기"}`}
                    title={`${node.name} · ${statusText(node.kind)}${instant ? "" : node.incoming.flatMap((p) => p.required_pieces.map((r) => `\n${r.name}: ${r.owned ? "보유" : "미보유"}`)).join("")}`}
                    aria-controls={openKind === node.kind && available && !readOnly && !instant ? "skill-node-popup" : undefined}
                    aria-expanded={openKind === node.kind && available && !readOnly && !instant}
                    onClick={() => chooseNode(node.kind)}
                    onFocus={(e) => e.currentTarget.scrollIntoView({ block: "nearest", inline: "nearest" })}
                  >
                    <span className="skill-node-orb"><PieceImage pieceKey={node.key} /><span className="skill-node-mark">{readOnly ? "·" : status(node.kind) === "acquired" ? "◆" : !node.incoming.length ? "·" : status(node.kind) === "available" ? "+" : "◈"}</span></span>
                    <strong>{node.name}</strong>
                    <small>{statusText(node.kind)}</small>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
        <div className="skill-map-toolbar">
          <div role="group" aria-label="트리 확대 및 축소">
            <button aria-label="트리 축소" disabled={scale <= 0.35} onClick={() => changeZoom(Math.max(0.35, scale - 0.15))}>−</button>
            <output aria-live="polite">{Math.round(scale * 100)}%</output>
            <button aria-label="트리 확대" disabled={scale >= 1.8} onClick={() => changeZoom(Math.min(1.8, scale + 0.15))}>+</button>
            <button aria-pressed={viewMode === "overview"} onClick={() => setViewMode("overview")}>전체 보기</button>
            <button aria-pressed={viewMode === "default"} onClick={() => { setViewMode("default");
              centerAt({ x: (focus.left + focus.right) / 2, y: (focus.top + focus.bottom) / 2 }, DEFAULT_SCALE); }}>기본 보기</button>
          </div>
        </div>
      </div>
      {readOnly && <div className="compendium-upgrade-detail" role="status"><strong>{active.name}</strong><p>{active.description}</p>{active.incoming.map((path) => <p key={`${path.from}-${path.to}`}>{path.from_name} → {path.to_name}: {path.effect} · {path.requirement}</p>)}</div>}
      {!readOnly && !instant && !inlineConfirm && openKind !== null && available && createPortal(<div key={openKind} id="skill-node-popup" ref={popup} className="skill-node-popup" role="group" aria-label={`${active.name} 강화 동작`}>
        <button type="button" className="primary" onClick={() => confirm(active.kind)}>{selectionOnly ? "경로 선택" : "강화 확정"}</button>
      </div>, document.body)}
    </section>
  );
}
