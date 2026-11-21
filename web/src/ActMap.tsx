import { useLayoutEffect, useMemo, useRef, type ReactNode } from "react";
import { PieceImage } from "./PieceImage";
import type { Command, MapNode, NodeKind, Snapshot } from "./types";
import { floorLabel, floorNames } from "./progress";

export const nodeTypes: Record<NodeKind, { icon: ReactNode; label: string }> = {
  combat: { icon: "⚔", label: "일반 전투" },
  elite: { icon: <PieceImage pieceKey="rook" owner={1} />, label: "엘리트" },
  rest: { icon: "⌂", label: "휴식" },
  treasure: { icon: "◇", label: "보물" },
  boss: { icon: "♛", label: "보스" },
  event: { icon: "?", label: "이벤트" },
  shop: { icon: "¤", label: "상점" },
};
const status = (node: MapNode) =>
  node.reward_revisit
    ? node.unclaimed_reward ? "미획득 보상 확인 가능" : "보상 내역 확인 가능"
    : node.current
    ? "현재 위치"
    : node.visited
      ? "방문 완료"
      : node.available
        ? "선택 가능"
        : node.skipped
          ? "지나친 경로"
          : "아직 선택 불가";

const ROAD_POINTS_PER_PIXEL = 0.12;
const ROAD_ROTATION_ERROR = 12;
const NODE_X_ERROR = 5;
const NODE_Y_ERROR = 12;
const MAP_CENTER_X = 250;
const LANE_LEFT_X = 58;
const LANE_GAP = 64;
const nodeX = (node: MapNode) => LANE_LEFT_X + node.lane * LANE_GAP;

function roadEdges(nodes: MapNode[]): [MapNode, MapNode][] {
  const visited = new Set<number>();
  const edges: [MapNode, MapNode][] = [];
  const visit = (id: number) => {
    if (visited.has(id)) return;
    visited.add(id);
    const node = nodes[id];
    for (const next of node.next) {
      edges.push([node, nodes[next]]);
      visit(next);
    }
  };
  for (const node of nodes) {
    if (node.row === 0) visit(node.id);
  }
  return edges;
}

function rotationError(seed: number, from: number, to: number, point: number): number {
  let value = (seed ^ Math.imul(from + 1, 0x9e3779b1) ^ Math.imul(to + 1, 0x85ebca6b) ^ point) >>> 0;
  value = Math.imul(value ^ (value >>> 16), 0x7feb352d);
  value = Math.imul(value ^ (value >>> 15), 0x846ca68b);
  return (((value ^ (value >>> 16)) >>> 0) / 0xffffffff * 2 - 1) * ROAD_ROTATION_ERROR;
}

function nodePositionError(seed: number, id: number, axis: 0 | 1): number {
  let value = (seed ^ Math.imul(id + 1, 0x9e3779b1) ^ (axis === 0 ? 0x85ebca6b : 0xc2b2ae35)) >>> 0;
  value = Math.imul(value ^ (value >>> 16), 0x7feb352d);
  value = Math.imul(value ^ (value >>> 15), 0x846ca68b);
  const extent = axis === 0 ? NODE_X_ERROR : NODE_Y_ERROR;
  return (((value ^ (value >>> 16)) >>> 0) / 0xffffffff * 2 - 1) * extent;
}

export function ActMap({
  state,
  send,
  onClose,
  coveredByReward = false,
  overlay = false,
}: {
  state: Snapshot;
  send: (command: Command) => void;
  onClose?: () => void;
  coveredByReward?: boolean;
  overlay?: boolean;
}) {
  const { map } = state;
  const top = useRef<HTMLElement>(null);
  useLayoutEffect(() => {
    const position = top.current?.querySelector<HTMLElement>(
      map.current === null ? '.map-node[data-row="0"]' : ".map-node-slot.is-current",
    );
    position?.scrollIntoView({ behavior: "instant", block: "center" });
  }, [state.floor, state.map.current]);
  const ended = state.phase === "victory" || state.phase === "defeat";
  const mapPhase = state.phase === "map" || coveredByReward;
  const rows = useMemo(() => Array.from({ length: map.rows }, (_, row) =>
    map.nodes.filter((node) => node.row === row),
  ), [map.nodes, map.rows]);
  const column = (node: MapNode) =>
    rows[node.row].findIndex((peer) => peer.id === node.id);
  const edges = useMemo(() => roadEdges(map.nodes), [map.nodes]);
  const centerOffset = useMemo(() => {
    const positions = map.nodes.filter((node) => node.kind !== "boss")
      .map((node) => nodeX(node) + nodePositionError(state.seed, node.id, 0));
    if (positions.length === 0) return 0;
    return MAP_CENTER_X - (Math.min(...positions) + Math.max(...positions)) / 2;
  }, [map.nodes, state.seed]);
  const x = (node: MapNode) => node.kind === "boss"
    ? MAP_CENTER_X
    : nodeX(node) + nodePositionError(state.seed, node.id, 0) + centerOffset;
  const rowGap = 104.5;
  const y = (node: MapNode) => 58 + (map.rows - 1 - node.row) * rowGap
    + (node.kind === "boss" ? 0 : nodePositionError(state.seed, node.id, 1));
  const chartHeight = 58 + (map.rows - 1) * rowGap + 92;
  const choose = (id: number) => send({ type: "select_node", id });
  return (
    <main className={`act-map-screen ${overlay ? "map-overlay" : ""}`} data-phase={mapPhase ? "map" : state.phase} data-floor={state.floor} ref={top}>
      <div className="map-heading">
        <div>
          <p className="eyebrow">{floorLabel(state.floor, state.final_battle)} / {state.character}</p>
          <h1>{state.mode === "endless" ? "무한 원정" : state.final_battle ? "부족의 독재자" : floorNames[state.floor]}</h1>
          <p>{state.mode === "endless" ? `누적 ${state.cleared_stages}단계 클리어` : state.final_battle ? "최후의 로얄을 향한 결전." : "하나의 원정, 당신이 고르는 길."} · 금화 {state.gold}</p>
        </div>
        <div className="map-progress">
          <strong>{map.nodes.filter((n) => n.visited).length}</strong>
          <span> / {map.rows}개 지점 완료</span>
          {overlay && onClose && (
            <button className="primary" onClick={onClose}>
              {ended ? "원정 결과로 돌아가기" : "진행 중인 노드로 돌아가기"} →
            </button>
          )}
        </div>
      </div>
      <div className="map-content">
        <section className="map-chart" aria-label={`${state.floor}층 전체 경로`} style={{ height: chartHeight }}>
          <div className="map-direction">보스를 향해 ↑</div>
          <svg
            className="map-connections"
            viewBox={`0 0 500 ${chartHeight}`}
            preserveAspectRatio="none"
            aria-hidden="true"
          >
            {edges.map(([from, to]) => {
                const id = to.id;
                const taken = map.path.some(
                  (nodeId, i) =>
                    nodeId === from.id && map.path[i + 1] === to.id,
                );
                const next = from.current && to.available;
                const x1 = x(from);
                const y1 = y(from);
                const x2 = x(to);
                const y2 = y(to);
                const distance = Math.hypot(x2 - x1, y2 - y1);
                const count = Math.max(1, Math.round(distance * ROAD_POINTS_PER_PIXEL));
                const angle = Math.atan2(y2 - y1, x2 - x1) * 180 / Math.PI;
                return (
                  <g
                    key={`${from.id}-${id}`}
                    data-edge={`${from.id}-${id}`}
                    data-road-points={count}
                    data-taken={taken || undefined}
                    className={
                      taken
                        ? "taken"
                        : next
                          ? "next"
                          : from.skipped || to.skipped
                            ? "skipped"
                            : ""
                    }
                  >
                    {Array.from({ length: count }, (_, point) => {
                      const t = (point + 1) / (count + 1);
                      const px = x1 + (x2 - x1) * t;
                      const py = y1 + (y2 - y1) * t;
                      const rotation = angle + rotationError(state.seed, from.id, id, point);
                      return <rect key={point} className="road-point" x={px - 2.5} y={py - 1} width="5" height="2" transform={`rotate(${rotation} ${px} ${py})`} />;
                    })}
                  </g>
                );
              })}
          </svg>
          {map.nodes.map((node) => (
            <div
              key={node.id}
              className={`map-node-slot kind-${node.kind} ${node.current ? "is-current" : ""} ${node.visited ? "is-visited" : ""} ${node.available ? "is-available" : ""} ${node.reward_revisit ? "has-reward-history" : ""} ${node.skipped ? "is-skipped" : ""}`}
              style={{ left: `${x(node) / 5}%`, top: `${(y(node) / chartHeight) * 100}%` }}
            >
              {node.current && <span className="current-marker">현재</span>}
              <button
                className="map-node"
                data-node-id={node.id}
                data-kind={node.kind}
                data-row={node.row}
                data-lane={node.lane}
                data-next={node.next.join(",")}
                data-status={status(node)}
                disabled={!node.available && !node.reward_revisit}
                onClick={() => choose(node.id)}
                title={`${nodeTypes[node.kind].label}${node.kind === "boss" ? ` · ${node.name}` : ""} · ${status(node)}`}
                aria-label={`${node.kind === "boss" ? "별도 보스 노드" : `${node.row + 1}단계 경로 ${column(node) + 1}`} · ${nodeTypes[node.kind].label}${node.kind === "boss" ? ` · ${node.name}` : ""} · ${status(node)}`}
                aria-current={node.current ? "location" : undefined}
              >
                <span aria-hidden="true">
                  {node.visited ? "✓" : nodeTypes[node.kind].icon}
                </span>
              </button>
              {node.unclaimed_reward && <span className="map-reward-badge" aria-label="미획득 보상">✦</span>}
            </div>
          ))}
        </section>

      </div>
    </main>
  );
}
