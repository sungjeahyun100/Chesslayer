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
  event: { icon: "♨", label: "식당" },
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

function layoutNodeX(rows: MapNode[][]) {
  const gap = 56;
  const parents = new Map<number, number[]>();
  const positions = new Map<number, number>();
  const anchors = new Map<number, number>();

  for (const row of rows) {
    row.forEach((node, index) => {
      const x = 250 + (index - (row.length - 1) / 2) * gap;
      positions.set(node.id, x);
      anchors.set(node.id, x);
      for (const child of node.next) {
        const incoming = parents.get(child) ?? [];
        incoming.push(node.id);
        parents.set(child, incoming);
      }
    });
  }

  // Pull connected nodes together. Pool adjacent positions when they would
  // overlap, preserving the graph's left-to-right order and route choices.
  for (let pass = 0; pass < 24; pass++) {
    const orderedRows = pass % 2 === 0 ? rows : [...rows].reverse();
    for (const row of orderedRows) {
      const targets = row.map((node) => {
        const neighbors = [...(parents.get(node.id) ?? []), ...node.next];
        const total = neighbors.reduce((sum, id) => sum + (positions.get(id) ?? 250), 0);
        const weight = neighbors.length + 0.35;
        return { x: (total + (anchors.get(node.id) ?? 250) * 0.35) / weight, weight };
      });
      const blocks: { start: number; end: number; mean: number; weight: number }[] = [];
      targets.forEach((target, index) => {
        blocks.push({ start: index, end: index, mean: target.x - index * gap, weight: target.weight });
        while (blocks.length > 1 && blocks[blocks.length - 2].mean > blocks[blocks.length - 1].mean) {
          const right = blocks.pop()!;
          const left = blocks.pop()!;
          blocks.push({
            start: left.start,
            end: right.end,
            mean: (left.mean * left.weight + right.mean * right.weight) / (left.weight + right.weight),
            weight: left.weight + right.weight,
          });
        }
      });
      for (const block of blocks) {
        for (let index = block.start; index <= block.end; index++) {
          positions.set(row[index].id, block.mean + index * gap);
        }
      }
    }
  }
  return positions;
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
      map.current === null ? ".map-start" : ".map-node-slot.is-current",
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
  const positions = useMemo(() => layoutNodeX(rows), [rows]);
  const x = (node: MapNode) => positions.get(node.id) ?? 250;
  const rowGap = 104.5;
  const y = (node: MapNode) => 58 + (map.rows - 1 - node.row) * rowGap;
  const chartHeight = 58 + (map.rows - 1) * rowGap + 92;
  const choose = (id: number) => send({ type: "select_node", id });
  return (
    <main className={`act-map-screen ${overlay ? "map-overlay" : ""}`} data-phase={mapPhase ? "map" : state.phase} data-floor={state.floor} ref={top}>
      <div className="map-heading">
        <div>
          <p className="eyebrow">{floorLabel(state.floor, state.final_battle)} / {state.character}</p>
          <h1>{state.final_battle ? "부족의 독재자" : floorNames[state.floor]}</h1>
          <p>{state.final_battle ? "최후의 로얄을 향한 결전." : "하나의 원정, 당신이 고르는 길."} · 금화 {state.gold}</p>
        </div>
        <div className="map-progress">
          <strong>{map.nodes.filter((n) => n.visited).length}</strong>
          <span> / {map.rows}개 지점 완료</span>
          {state.phase === "map" && map.nodes.some((node) => node.current && node.visited && node.kind === "boss") &&
            <button className="primary" onClick={() => send({ type: "advance_floor" })}>원정 마치기 →</button>}
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
            {map.nodes.flatMap((from) =>
              from.next.map((id) => {
                const to = map.nodes[id];
                const taken = map.path.some(
                  (nodeId, i) =>
                    nodeId === from.id && map.path[i + 1] === to.id,
                );
                const next = from.current && to.available;
                return (
                  <path
                    key={`${from.id}-${id}`}
                    data-edge={`${from.id}-${id}`}
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
                    d={`M ${x(from)} ${y(from) - 22} L ${x(to)} ${y(to) + 22}`}
                  />
                );
              }),
            )}
            {map.nodes
              .filter((n) => n.row === 0)
              .map((n) => (
                <path
                  key={`start-${n.id}`}
                  className={
                    map.path[0] === n.id
                      ? "taken"
                      : n.available
                        ? "next"
                        : "skipped"
                  }
                  d={`M 250 ${chartHeight - 33} L ${x(n)} ${y(n) + 22}`}
                />
              ))}
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
                data-next={node.next.join(",")}
                data-status={status(node)}
                disabled={!node.available && !node.reward_revisit}
                onClick={() => choose(node.id)}
                title={`${nodeTypes[node.kind].label}${node.kind === "boss" ? ` · ${node.name}` : ""} · ${status(node)}`}
                aria-label={`${node.row + 1}단계 경로 ${column(node) + 1} · ${nodeTypes[node.kind].label}${node.kind === "boss" ? ` · ${node.name}` : ""} · ${status(node)}`}
                aria-current={node.current ? "location" : undefined}
              >
                <span aria-hidden="true">
                  {node.visited ? "✓" : nodeTypes[node.kind].icon}
                </span>
              </button>
              {node.unclaimed_reward && <span className="map-reward-badge" aria-label="미획득 보상">✦</span>}
            </div>
          ))}
          <span
            className={`map-start ${map.current === null ? "at-start" : ""}`}
          >
            출발{map.current === null ? " · 현재 위치" : ""}
          </span>
        </section>

      </div>
    </main>
  );
}
