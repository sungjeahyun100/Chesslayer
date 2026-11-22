import { test, expect } from "@playwright/test";
import { buildUpgradeTree } from "../src/upgradeTree";
import type { RestOption, UpgradePath } from "../src/types";

const piece: RestOption = { id: 1, kind: 1, key: "piece-1", name: "기물 1", description: "현재 기물", cell: 0, origin: 0, square: "a8", upgrades: [], ward: false, can_upgrade: true, can_revive: false, dead: false };
function paths(pairs: [number, number][]): UpgradePath[] {
  return pairs.map(([from, to]) => ({ from, to, from_key: `piece-${from}`, from_name: `기물 ${from}`,
    to_key: `piece-${to}`, to_name: `기물 ${to}`, effect: "효과", requirement: "선행 기물", required_pieces: [], unlocked: true, key: `${from}-${to}` }));
}

test("하위 트리 영역과 계열 구역은 겹치지 않고 깊이는 왼쪽에서 오른쪽으로 진행한다", () => {
  const input = paths([[1, 2], [1, 3], [1, 4], [2, 5], [2, 6], [5, 7], [3, 8], [8, 9]]);
  const tree = buildUpgradeTree(input, piece);
  expect(tree.diagnostics).toEqual({ multipleParents: [], cyclic: false });
  expect(tree.lanes).toHaveLength(3);
  const byKind = new Map(tree.nodes.map((node) => [node.kind, node]));
  for (const edge of tree.edges) {
    expect(byKind.get(edge.from)!.x).toBeLessThan(byKind.get(edge.to)!.x);
    expect(edge.d).toMatch(/^M /);
    expect(edge.d).not.toContain(" C ");
  }
  for (const node of tree.nodes) {
    expect(node.x).toBeGreaterThanOrEqual(95);
    expect(node.x + 95).toBeLessThan(tree.width);
    expect(node.y).toBeGreaterThan(37);
    expect(node.y + 100).toBeLessThan(tree.height);
    for (const other of tree.nodes.filter((candidate) => candidate.depth === node.depth && candidate.kind !== node.kind))
      expect(Math.abs(node.y - other.y)).toBeGreaterThanOrEqual(144);
  }
  for (let i = 1; i < tree.lanes.length; i++) expect(tree.lanes[i].y).toBeGreaterThanOrEqual(tree.lanes[i - 1].y + tree.lanes[i - 1].height);
  expect(tree.lanes.map((lane) => lane.id)).toEqual([2, 3, 4]);
  expect(buildUpgradeTree([...input].reverse(), piece)).toEqual(tree);
});

test("합류와 순환 관계는 삭제하지 않고 진단한다", () => {
  const merged = buildUpgradeTree(paths([[1, 2], [1, 3], [2, 4], [3, 4]]), piece);
  expect(merged.edges).toHaveLength(4);
  expect(merged.nodes.find((node) => node.kind === 4)?.incoming).toHaveLength(2);
  expect(merged.diagnostics.multipleParents).toEqual([4]);
  const cyclic = buildUpgradeTree(paths([[1, 2], [2, 3], [3, 1]]), piece);
  expect(cyclic.edges).toHaveLength(3);
  expect(cyclic.diagnostics.cyclic).toBe(true);
});
