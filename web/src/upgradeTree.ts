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

// The engine's edges remain authoritative. Include ancestors, siblings and all
// descendants, rather than just the edges touching the current piece kind.
export function buildUpgradeTree(paths: UpgradePath[], piece: RestOption) {
  const kinds = new Set([piece.kind]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const edge of paths) {
      if (!kinds.has(edge.from) && !kinds.has(edge.to)) continue;
      for (const kind of [edge.from, edge.to]) {
        if (!kinds.has(kind)) { kinds.add(kind); changed = true; }
      }
    }
  }
  const edges = paths.filter((p) => kinds.has(p.from) && kinds.has(p.to));
  const nodes = [...kinds].sort((a, b) => a - b).map((kind): SkillNode => {
    const incoming = edges.filter((p) => p.to === kind);
    const outgoing = edges.find((p) => p.from === kind);
    return {
      kind,
      key: incoming[0]?.to_key ?? outgoing?.from_key ?? piece.key,
      name: incoming[0]?.to_name ?? outgoing?.from_name ?? piece.name,
      description: kind === piece.kind ? piece.description : incoming[0]?.effect ?? "이 계열의 기본 기물입니다. 연결된 가지에서 강화 방향을 선택하세요.",
      incoming, depth: 0, x: 0, y: 0,
    };
  });
  const byKind = new Map(nodes.map((n) => [n.kind, n]));
  const remaining = new Map(nodes.map((n) => [n.kind, n.incoming.length]));
  const queue = nodes.filter((n) => !n.incoming.length);
  for (let i = 0; i < queue.length; i++) {
    const node = queue[i];
    for (const edge of edges.filter((p) => p.from === node.kind)) {
      const child = byKind.get(edge.to)!;
      child.depth = Math.max(child.depth, node.depth + 1);
      remaining.set(child.kind, remaining.get(child.kind)! - 1);
      if (remaining.get(child.kind) === 0) queue.push(child);
    }
  }

  // Virtual routing points reserve space for edges that skip a level. They are
  // never rendered as upgrade nodes. Multiple parents share one destination.
  type Vertex = { id: string; depth: number; x: number; y: number };
  const vertices: Vertex[] = nodes.map((n) => ({ ...n, id: String(n.kind) }));
  const routes = edges.map((edge, index) => {
    const from = byKind.get(edge.from)!;
    const to = byKind.get(edge.to)!;
    const ids = [String(from.kind)];
    for (let depth = from.depth + 1; depth < to.depth; depth++) {
      const id = `route-${index}-${depth}`;
      vertices.push({ id, depth, x: 0, y: 0 });
      ids.push(id);
    }
    ids.push(String(to.kind));
    return { edge, ids };
  });
  const depth = Math.max(...nodes.map((n) => n.depth));
  const layers = Array.from({ length: depth + 1 }, (_, d) => vertices.filter((v) => v.depth === d));
  const links = routes.flatMap((r) => r.ids.slice(1).map((id, i) => [r.ids[i], id]));
  // Barycentric sweeps keep sibling branches together and reduce crossings at merges.
  for (let pass = 0; pass < 4; pass++) {
    const forward = pass % 2 === 0;
    const order = forward ? layers : [...layers].reverse();
    for (const layer of order) {
      const positions = new Map(layers.flatMap((l) => l.map((v, i) => [v.id, i] as const)));
      const score = (vertex: Vertex) => {
        const neighbors = links.filter((l) => l[forward ? 1 : 0] === vertex.id).map((l) => positions.get(l[forward ? 0 : 1])!);
        return neighbors.length ? neighbors.reduce((a, b) => a + b, 0) / neighbors.length : positions.get(vertex.id)!;
      };
      layer.sort((a, b) => score(a) - score(b));
    }
  }
  const width = Math.max(520, depth * 250 + 220);
  const height = Math.max(360, Math.max(...layers.map((l) => l.length)) * 170 + 80);
  for (const layer of layers) {
    layer.forEach((v, i) => {
      v.x = nodes.length === 1 ? width / 2 : 110 + v.depth * 250;
      v.y = height / 2 - 22 + (i - (layer.length - 1) / 2) * 170;
      const node = byKind.get(Number(v.id));
      if (node) { node.x = v.x; node.y = v.y; }
    });
  }
  const byId = new Map(vertices.map((v) => [v.id, v]));
  return {
    nodes, width, height,
    edges: routes.map(({ edge, ids }) => {
      const outgoing = edges.filter((candidate) => candidate.from === edge.from).sort((a, b) => a.to - b.to);
      const incoming = edges.filter((candidate) => candidate.to === edge.to).sort((a, b) => a.from - b.from);
      const port = (index: number, count: number) => Math.max(-23, Math.min(23, (index - (count - 1) / 2) * 11));
      const exitY = byKind.get(edge.from)!.y + port(outgoing.indexOf(edge), outgoing.length);
      const entryY = byKind.get(edge.to)!.y + port(incoming.indexOf(edge), incoming.length);
      const points = ids.map((id, i) => {
        const v = byId.get(id)!;
        return { x: v.x + (i === 0 ? 39 : i === ids.length - 1 ? -39 : 0),
          y: i === 0 ? exitY : i === ids.length - 1 ? entryY : v.y };
      });
      let d = `M ${points[0].x} ${points[0].y}`;
      for (let i = 1; i < points.length; i++) {
        const a = points[i - 1], b = points[i];
        const mid = (a.x + b.x) / 2;
        d += ` C ${mid} ${a.y}, ${mid} ${b.y}, ${b.x} ${b.y}`;
      }
      return { ...edge, d };
    }),
  };
}
