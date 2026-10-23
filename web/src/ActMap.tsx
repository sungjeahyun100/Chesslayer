import { useEffect, useRef, useState } from "react";
import type { Command, MapNode, NodeKind, Snapshot } from "./types";

export const nodeTypes: Record<NodeKind, { icon: string; label: string }> = {
  combat: { icon: "⚔", label: "일반 전투" },
  elite: { icon: "♜", label: "엘리트" },
  rest: { icon: "⌂", label: "휴식" },
  treasure: { icon: "◇", label: "보물" },
  boss: { icon: "♛", label: "보스" },
};
const status = (node: MapNode) =>
  node.current
    ? "현재 위치"
    : node.visited
      ? "방문 완료"
      : node.available
        ? "선택 가능"
        : node.skipped
          ? "지나친 경로"
          : "아직 선택 불가";

export function ActMap({
  state,
  send,
  onClose,
}: {
  state: Snapshot;
  send: (command: Command) => void;
  onClose: () => void;
}) {
  const [inspected, inspect] = useState<number | null>(null);
  const top = useRef<HTMLElement>(null);
  useEffect(() => {
    top.current?.scrollIntoView({ behavior: "smooth", block: "start" });
  }, []);
  const { map } = state;
  const ended = state.phase === "victory" || state.phase === "defeat";
  const current = map.nodes.find((n) => n.current);
  const available = map.nodes.filter((n) => n.available);
  const detail =
    map.nodes.find((n) => n.id === inspected) ?? available[0] ?? current;
  const rows = Array.from({ length: map.rows }, (_, row) =>
    map.nodes.filter((node) => node.row === row),
  );
  const column = (node: MapNode) =>
    rows[node.row].findIndex((peer) => peer.id === node.id);
  // Center each layer in reading order; irregular random lane gaps obscure the edges.
  const x = (node: MapNode) =>
    250 + (column(node) - (rows[node.row].length - 1) / 2) * 135;
  const y = (node: MapNode) => 58 + (map.rows - 1 - node.row) * 95;
  const choose = (id: number) => send({ type: "select_node", id });
  return (
    <main className="act-map-screen" data-phase={state.phase} ref={top}>
      <div className="map-heading">
        <div>
          <p className="eyebrow">ACT I / {state.character}</p>
          <h1>갈림길의 숲</h1>
          <p>하나의 원정, 당신이 고르는 길.</p>
        </div>
        <div className="map-progress">
          <strong>{map.nodes.filter((n) => n.visited).length}</strong>
          <span> / {map.rows}개 지점 완료</span>
          {state.phase !== "map" && (
            <button className="primary" onClick={onClose}>
              {ended ? "원정 결과로 돌아가기" : "진행 중인 노드로 돌아가기"} →
            </button>
          )}
        </div>
      </div>
      <div className="map-legend" aria-label="노드 유형">
        {Object.entries(nodeTypes).map(([kind, entry]) => (
          <span key={kind} className={`legend-${kind}`}>
            <b>{entry.icon}</b>
            {entry.label}
          </span>
        ))}
      </div>
      <div className="map-content">
        <section className="map-chart" aria-label="제1막 전체 경로">
          <div className="map-direction">보스를 향해 ↑</div>
          <svg
            className="map-connections"
            viewBox="0 0 500 720"
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
                  d={`M 250 687 L ${x(n)} ${y(n) + 22}`}
                />
              ))}
          </svg>
          {map.nodes.map((node) => (
            <div
              key={node.id}
              className={`map-node-slot kind-${node.kind} ${node.current ? "is-current" : ""} ${node.visited ? "is-visited" : ""} ${node.available ? "is-available" : ""} ${node.skipped ? "is-skipped" : ""}`}
              style={{ left: `${x(node) / 5}%`, top: `${y(node) / 7.2}%` }}
              onMouseEnter={() => inspect(node.id)}
            >
              {node.current && <span className="current-marker">현재</span>}
              <button
                className="map-node"
                data-node-id={node.id}
                data-kind={node.kind}
                data-row={node.row}
                data-next={node.next.join(",")}
                data-status={status(node)}
                disabled={!node.available}
                onFocus={() => inspect(node.id)}
                onClick={() => choose(node.id)}
                title={`${node.name} · ${node.description}`}
                aria-label={`${node.row + 1}단계 경로 ${column(node) + 1} · ${nodeTypes[node.kind].label} · ${node.name} · ${status(node)}`}
                aria-current={node.current ? "location" : undefined}
              >
                <span aria-hidden="true">
                  {node.visited ? "✓" : nodeTypes[node.kind].icon}
                </span>
              </button>
              <span className="node-caption">
                {node.kind === "combat"
                  ? node.name
                  : nodeTypes[node.kind].label}
              </span>
            </div>
          ))}
          <span
            className={`map-start ${map.current === null ? "at-start" : ""}`}
          >
            출발{map.current === null ? " · 현재 위치" : ""}
          </span>
        </section>
        <aside className="map-aside">
          <section className="map-location">
            <p className="eyebrow">YOUR EXPEDITION</p>
            <h2>{current ? current.name : "첫 갈림길"}</h2>
            <p>
              {ended
                ? "원정이 끝났습니다. 지나온 경로를 확인하세요."
                : state.phase === "map"
                  ? current
                    ? "이곳을 마쳤습니다. 이어진 길 중 하나를 선택하세요."
                    : "세 출발점 중 하나를 선택하세요. 이 지도는 보스까지 유지됩니다."
                  : "노드를 완료하면 연결된 다음 길을 선택할 수 있습니다."}
            </p>
            <div className="map-supplies">
              <span>
                생존 기물{" "}
                <b>{state.pieces.filter((p) => p.owner === 0).length}</b>
              </span>
              <span>
                덱 <b>{state.deck_count}</b>
              </span>
              <span>
                유물 <b>{state.relics.length}</b>
              </span>
            </div>
          </section>
          <section className="next-choices">
            <h3>{available.length ? "다음 행선지" : "경로 탐색"}</h3>
            {available.map((node) => (
              <button
                key={node.id}
                onClick={() => choose(node.id)}
                onMouseEnter={() => inspect(node.id)}
                onFocus={() => inspect(node.id)}
              >
                <span className={`choice-icon kind-${node.kind}`}>
                  {nodeTypes[node.kind].icon}
                </span>
                <span>
                  <small>
                    경로 {column(node) + 1} · {nodeTypes[node.kind].label}
                  </small>
                  <strong>{node.name}</strong>
                </span>
                <span>↗</span>
              </button>
            ))}
            {!available.length && (
              <p>
                {ended
                  ? "새 원정은 결과 화면에서 시작할 수 있습니다."
                  : "진행 중인 노드로 돌아가 완료하세요."}
              </p>
            )}
          </section>
          {detail && (
            <section className={`map-detail detail-${detail.kind}`}>
              <p className="eyebrow">{nodeTypes[detail.kind].label}</p>
              <h3>{detail.name}</h3>
              <p>{detail.description}</p>
            </section>
          )}
          <div className="map-key">
            <span>
              <i className="key-current" />
              현재 위치
            </span>
            <span>
              <i className="key-available" />
              선택 가능한 길
            </span>
            <span>
              <i className="key-visited" />
              지나온 길
            </span>
            <span>
              <i />
              아직 갈 수 없는 길
            </span>
          </div>
          <p className="map-note">
            기물의 위치와 손실은 다음 전투로 이어집니다. 적의 배치와 겹치면
            역습을 받습니다.
          </p>
        </aside>
      </div>
      {state.log.length > 0 && (
        <div className="map-recent" role="log" aria-label="최근 원정 기록">
          {state.log.slice(-3).map((line, i) => (
            <p key={i}>{line}</p>
          ))}
        </div>
      )}
    </main>
  );
}
