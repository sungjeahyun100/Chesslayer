import type { RestOption, UpgradePath } from "./types";

export interface SkillNode {
  kind: number;
  key: string;
  name: string;
  description: string;
  incoming: UpgradePath[];
  depth: number;
  x: number;
  y: number;
}

export interface SkillLane {
  id: number;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

const COLUMN = 225;
const ROW = 136;
const BRANCH_GAP = 8;
const LANE_GAP = 12;
const MARGIN_X = 104;
const MARGIN_Y = 16;

// Return the complete connected family without changing the engine's edges.
function collectFamily(paths: UpgradePath[], kind: number) {
  const kinds = new Set([kind]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const edge of paths) {
      if (!kinds.has(edge.from) && !kinds.has(edge.to)) continue;
      for (const endpoint of [edge.from, edge.to]) {
        if (!kinds.has(endpoint)) { kinds.add(endpoint); changed = true; }
      }
    }
  }
  return paths.filter((edge) => kinds.has(edge.from) && kinds.has(edge.to))
    .sort((a, b) => a.from - b.from || a.to - b.to || a.key.localeCompare(b.key));
}

function makeNodes(edges: UpgradePath[], piece: RestOption) {
  const kinds = new Set([piece.kind]);
  edges.forEach((edge) => { kinds.add(edge.from); kinds.add(edge.to); });
  return [...kinds].sort((a, b) => a - b).map((kind): SkillNode => {
    const incoming = edges.filter((edge) => edge.to === kind);
    const outgoing = edges.find((edge) => edge.from === kind);
    return {
      kind,
      key: incoming[0]?.to_key ?? outgoing?.from_key ?? piece.key,
      name: incoming[0]?.to_name ?? outgoing?.from_name ?? piece.name,
      description: kind === piece.kind ? piece.description : incoming[0]?.effect ?? "이 계열의 기본 기물입니다. 연결된 가지에서 강화 방향을 선택하세요.",
      incoming, depth: 0, x: 0, y: 0,
    };
  });
}

// A tree uses every edge. For legacy merges, choose one parent for placement
// while retaining all actual edges for display and gameplay.
function makeHierarchy(nodes: SkillNode[], edges: UpgradePath[]) {
  const byKind = new Map(nodes.map((node) => [node.kind, node]));
  const outgoing = new Map(nodes.map((node) => [node.kind, edges.filter((edge) => edge.from === node.kind).sort((a, b) => a.to - b.to)]));
  const indegree = new Map(nodes.map((node) => [node.kind, node.incoming.length]));
  const queue = nodes.filter((node) => !node.incoming.length).map((node) => node.kind);
  const topological: number[] = [];
  while (queue.length) {
    const kind = queue.shift()!;
    topological.push(kind);
    for (const edge of outgoing.get(kind) ?? []) {
      indegree.set(edge.to, indegree.get(edge.to)! - 1);
      if (indegree.get(edge.to) === 0) queue.push(edge.to);
    }
    queue.sort((a, b) => a - b);
  }
  const cyclic = topological.length !== nodes.length;
  const order = [...topological, ...nodes.map((node) => node.kind).filter((kind) => !topological.includes(kind))];
  // Longest depth keeps every forward edge moving right in an acyclic graph.
  if (!cyclic) for (const kind of order) {
    const parent = byKind.get(kind)!;
    for (const edge of outgoing.get(kind) ?? []) byKind.get(edge.to)!.depth = Math.max(byKind.get(edge.to)!.depth, parent.depth + 1);
  }
  const parent = new Map<number, number>();
  for (const node of nodes) {
    const candidates = node.incoming.filter((edge) => !cyclic || order.indexOf(edge.from) < order.indexOf(node.kind))
      .sort((a, b) => byKind.get(b.from)!.depth - byKind.get(a.from)!.depth || a.from - b.from);
    if (candidates.length) parent.set(node.kind, candidates[0].from);
  }
  const children = new Map(nodes.map((node) => [node.kind, [] as SkillNode[]]));
  for (const [child, ancestor] of parent) children.get(ancestor)!.push(byKind.get(child)!);
  for (const group of children.values()) group.sort((a, b) => a.kind - b.kind);
  const roots = nodes.filter((node) => !parent.has(node.kind));
  if (cyclic) {
    const visit = (node: SkillNode, depth: number) => {
      node.depth = depth;
      for (const child of children.get(node.kind)!) visit(child, depth + 1);
    };
    roots.forEach((root) => visit(root, 0));
  }
  return { children, roots, cyclic, multipleParents: nodes.filter((node) => node.incoming.length > 1).map((node) => node.kind) };
}

function placeSubtrees(nodes: SkillNode[], hierarchy: ReturnType<typeof makeHierarchy>) {
  const { children, roots } = hierarchy;
  const span = new Map<number, number>();
  const measure = (node: SkillNode): number => {
    const group = children.get(node.kind)!;
    const height = group.length ? Math.max(ROW, group.reduce((sum, child) => sum + measure(child), 0) + BRANCH_GAP * (group.length - 1)) : ROW;
    span.set(node.kind, height);
    return height;
  };
  roots.forEach(measure);
  const place = (node: SkillNode, top: number) => {
    node.x = MARGIN_X + node.depth * COLUMN;
    const group = children.get(node.kind)!;
    if (!group.length) { node.y = top + span.get(node.kind)! / 2; return; }
    const total = group.reduce((sum, child) => sum + span.get(child.kind)!, 0) + BRANCH_GAP * (group.length - 1);
    let cursor = top + (span.get(node.kind)! - total) / 2;
    for (const child of group) { place(child, cursor); cursor += span.get(child.kind)! + BRANCH_GAP; }
    node.y = (group[0].y + group[group.length - 1].y) / 2;
  };
  const lanes: SkillLane[] = [];
  let cursor = MARGIN_Y;
  for (const root of roots) {
    const group = children.get(root.kind)!;
    if (!group.length) { place(root, cursor); cursor += ROW + LANE_GAP; continue; }
    for (const child of group) {
      const top = cursor;
      place(child, top);
      lanes.push({ id: child.kind, name: `${child.name} 계열`, x: child.x - 88,
        y: top, width: 0, height: span.get(child.kind)! });
      cursor += span.get(child.kind)! + LANE_GAP;
    }
    root.x = MARGIN_X + root.depth * COLUMN;
    root.y = (group[0].y + group[group.length - 1].y) / 2;
  }
  const maxDepth = Math.max(0, ...nodes.map((node) => node.depth));
  const width = Math.max(260, MARGIN_X + maxDepth * COLUMN + 102);
  const height = Math.max(220, ...nodes.map((node) => node.y + 112));
  for (const lane of lanes) lane.width = width - lane.x;
  return { lanes, width, height };
}

function roundedPath(points: { x: number; y: number }[]) {
  const compact = points.filter((point, index) => !index || point.x !== points[index - 1].x || point.y !== points[index - 1].y);
  let d = `M ${compact[0].x} ${compact[0].y}`;
  for (let i = 1; i < compact.length - 1; i++) {
    const previous = compact[i - 1], corner = compact[i], next = compact[i + 1];
    const incoming = Math.hypot(corner.x - previous.x, corner.y - previous.y);
    const outgoing = Math.hypot(next.x - corner.x, next.y - corner.y);
    if (!incoming || !outgoing) continue;
    const radius = Math.min(9, incoming / 2, outgoing / 2);
    const before = { x: corner.x + (previous.x - corner.x) * radius / incoming, y: corner.y + (previous.y - corner.y) * radius / incoming };
    const after = { x: corner.x + (next.x - corner.x) * radius / outgoing, y: corner.y + (next.y - corner.y) * radius / outgoing };
    d += ` L ${before.x} ${before.y} Q ${corner.x} ${corner.y} ${after.x} ${after.y}`;
  }
  const end = compact[compact.length - 1];
  return `${d} L ${end.x} ${end.y}`;
}

function routeEdges(edges: UpgradePath[], nodes: SkillNode[]) {
  const byKind = new Map(nodes.map((node) => [node.kind, node]));
  return edges.map((edge) => {
    const from = byKind.get(edge.from)!;
    const to = byKind.get(edge.to)!;
    const start = { x: from.x + 50, y: from.y };
    const end = { x: to.x - 50, y: to.y };
    // Preserve the root's shared trunk when routing child connections.
    const trunk = Math.min(start.x + (from.incoming.length ? 70 : 50), end.x - 16);
    const points = start.y === end.y ? [start, end] : [start, { x: trunk, y: start.y }, { x: trunk, y: end.y }, end];
    return { ...edge, d: roundedPath(points) };
  });
}

export function buildUpgradeTree(paths: UpgradePath[], piece: RestOption) {
  const edges = collectFamily(paths, piece.kind);
  const nodes = makeNodes(edges, piece);
  const hierarchy = makeHierarchy(nodes, edges);
  const { lanes, width, height } = placeSubtrees(nodes, hierarchy);
  return { nodes, edges: routeEdges(edges, nodes), lanes, width, height,
    diagnostics: { multipleParents: hierarchy.multipleParents, cyclic: hierarchy.cyclic } };
}
