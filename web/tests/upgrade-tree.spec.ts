import { test, expect } from "@playwright/test";
import { buildUpgradeTree } from "../src/upgradeTree";
import type { RestOption, UpgradePath } from "../src/types";

test("트리 배치: 후속 분기·여러 부모·단계를 건너뛰는 연결을 손실 없이 표현", () => {
  // Synthetic topology only. These nodes never enter the game or its catalog.
  const paths: UpgradePath[] = [[1, 2], [1, 3], [2, 4], [3, 4], [1, 5], [4, 5], [8, 9]].map(([from, to]) => ({
    from, to, from_key: `piece-${from}`, from_name: `기물 ${from}`,
    to_key: `piece-${to}`, to_name: `기물 ${to}`, effect: "효과", requirement: "선행 기물",
  }));
  const piece: RestOption = { id: 1, kind: 2, key: "piece-2", name: "기물 2", description: "현재 기물", cell: 0, origin: 0, square: "a8", upgrades: [], ward: false, can_upgrade: true, can_revive: false, dead: false };
  const graph = buildUpgradeTree(paths, piece);
  expect(graph.nodes.map((n) => n.kind)).toEqual([1, 2, 3, 4, 5]);
  expect(graph.edges).toHaveLength(6);
  expect(graph.nodes.find((n) => n.kind === 4)?.incoming).toHaveLength(2);
  expect(graph.nodes.find((n) => n.kind === 5)?.depth).toBe(3);
  for (const node of graph.nodes) {
    expect(node.x).toBeGreaterThan(74);
    expect(node.x).toBeLessThan(graph.width - 74);
    expect(node.y).toBeGreaterThan(37);
    expect(node.y).toBeLessThan(graph.height - 100);
    for (const other of graph.nodes.filter((n) => n.kind !== node.kind && n.depth === node.depth))
      expect(Math.abs(node.y - other.y)).toBeGreaterThanOrEqual(170);
  }
  expect(graph.edges.find((e) => e.from === 1 && e.to === 5)?.d.match(/ C /g)).toHaveLength(3);
  expect(buildUpgradeTree(paths, piece)).toEqual(graph);
});
