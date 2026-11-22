import { expect, test } from "@playwright/test";
import { buildUpgradeTree } from "../src/upgradeTree";
import type { RestOption, UpgradePath } from "../src/types";

const bases = ["킹", "퀸", "룩", "비숍", "나이트", "폰"];

test("실제 여섯 계열의 강화 관계는 단일 부모 순수 트리다", async ({ page }) => {
  await page.goto("/");
  const pieces = await page.evaluate(async () => {
    const { default: init, compendium } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const entries = JSON.parse(compendium()) as { id: string; pieces: { kind: number; key: string; name: string; description: string;
      upgrades: { from: number; to: number; from_key: string; from_name: string; to_key: string; to_name: string; effect: string; requirement: string }[] }[] }[];
    return entries.find((entry) => entry.id === "avenger")!.pieces;
  });
  const paths = pieces.flatMap((item) => item.upgrades.map((edge): UpgradePath => ({ ...edge, key: `${edge.from}-${edge.to}`, unlocked: false, required_pieces: [] })));
  for (const name of bases) {
    const selected = pieces.find((item) => item.name === name)!;
    const piece: RestOption = { id: 0, kind: selected.kind, key: selected.key, name: selected.name, description: selected.description,
      cell: 0, origin: 0, square: "a8", upgrades: [], ward: false, can_upgrade: false, can_revive: false, dead: false };
    const tree = buildUpgradeTree(paths, piece);
    expect(tree.diagnostics, name).toEqual({ multipleParents: [], cyclic: false });
    expect(tree.nodes.length, name).toBeGreaterThan(1);
    const root = tree.nodes.find((node) => !node.incoming.length)!;
    const laneOf = new Map<number, number>();
    const children = new Map(tree.nodes.map((node) => [node.kind, tree.edges.filter((edge) => edge.from === node.kind).map((edge) => edge.to)]));
    for (const lane of tree.lanes) {
      const pending = [lane.id];
      while (pending.length) {
        const kind = pending.pop()!;
        expect(laneOf.has(kind), `${name}: ${kind} is in two lanes`).toBe(false);
        laneOf.set(kind, lane.id);
        const node = tree.nodes.find((candidate) => candidate.kind === kind)!;
        expect(node.y, name).toBeGreaterThan(lane.y);
        expect(node.y, name).toBeLessThan(lane.y + lane.height);
        pending.push(...children.get(kind)!);
      }
    }
    for (const edge of tree.edges.filter((edge) => edge.from !== root.kind))
      expect(laneOf.get(edge.from), `${name}: cross-lane edge`).toBe(laneOf.get(edge.to));
  }
});

test("six compendium upgrade trees keep readable nodes, invisible groups and distinct default/overview", async ({ page }, testInfo) => {
  await page.goto("/");
  await page.getByRole("button", { name: /도감 보기/ }).click();
  await page.getByRole("navigation", { name: "도감 종류" }).getByRole("button", { name: /기물/ }).click();
  for (const name of bases) {
    await page.locator(".compendium-piece-list button").filter({ has: page.locator("strong").getByText(name, { exact: true }) }).click();
    const tree = page.locator(".skill-tree");
    await expect(tree.locator(".skill-node").first()).toBeVisible();
    await expect(tree.getByRole("button", { name: "기본 보기" })).toHaveAttribute("aria-pressed", "true");
    await expect(tree.locator(".skill-map-toolbar output")).toHaveText("56%");
    const metrics = await tree.evaluate((element, selectedName) => {
      const viewport = element.querySelector<HTMLElement>(".skill-viewport")!;
      const nodes = [...element.querySelectorAll<HTMLElement>(".skill-node")];
      const lanes = [...element.querySelectorAll<HTMLElement>(".skill-lane")];
      const rects = nodes.map((node) => ({ kind: node.dataset.kind, rect: node.getBoundingClientRect() }));
      const overlaps: string[] = [];
      for (let i = 0; i < rects.length; i++) for (let j = i + 1; j < rects.length; j++) {
        const a = rects[i].rect, b = rects[j].rect;
        if (Math.min(a.right, b.right) - Math.max(a.left, b.left) > 1 && Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top) > 1)
          overlaps.push(`${rects[i].kind}-${rects[j].kind}`);
      }
      const current = nodes.find((node) => node.querySelector("strong")?.textContent === selectedName)!.getBoundingClientRect();
      const view = viewport.getBoundingClientRect();
      return { count: nodes.length, lanes: lanes.length, overlaps,
        currentVisible: current.left < view.right && current.right > view.left && current.top < view.bottom && current.bottom > view.top,
        orthogonal: [...element.querySelectorAll<SVGPathElement>(".skill-line")].every((line) => !line.getAttribute("d")?.includes(" C ")) };
    }, name);
    expect(metrics.count, name).toBeGreaterThan(1);
    expect(metrics.lanes, name).toBe(0);
    expect(metrics.overlaps, name).toEqual([]);
    expect(metrics.currentVisible, name).toBe(true);
    expect(metrics.orthogonal, name).toBe(true);
    if (name === "퀸" || name === "나이트") {
      await tree.locator(".skill-map").screenshot({ path: testInfo.outputPath(`${name}-default.png`) });
    }
    await tree.getByRole("button", { name: "전체 보기" }).click();
    await expect(tree.getByRole("button", { name: "전체 보기" })).toHaveAttribute("aria-pressed", "true");
    const fit = await tree.evaluate((element) => {
      const view = element.querySelector<HTMLElement>(".skill-viewport")!.getBoundingClientRect();
      const nodes = [...element.querySelectorAll<HTMLElement>(".skill-node")];
      const rects = nodes.map((node) => node.getBoundingClientRect());
      return { allVisible: nodes.every((node) => {
        const rect = node.getBoundingClientRect();
        return rect.left >= view.left - 2 && rect.right <= view.right + 2 && rect.top >= view.top - 2 && rect.bottom <= view.bottom + 2;
      }), occupied: Math.max((Math.max(...rects.map((rect) => rect.right)) - Math.min(...rects.map((rect) => rect.left))) / view.width,
        (Math.max(...rects.map((rect) => rect.bottom)) - Math.min(...rects.map((rect) => rect.top))) / view.height) };
    });
    expect(fit.allVisible, name).toBe(true);
    expect(fit.occupied, name).toBeGreaterThan(0.8);
    if (name === "퀸" || name === "나이트") await tree.locator(".skill-map").screenshot({ path: testInfo.outputPath(`${name}-overview.png`) });
    await tree.getByRole("button", { name: "기본 보기" }).click();
    await expect(tree.getByRole("button", { name: "기본 보기" })).toHaveAttribute("aria-pressed", "true");
    await expect(tree.locator(".skill-map-toolbar output")).toHaveText("56%");
    if (name === "퀸") {
      const before = Number((await tree.locator(".skill-map-toolbar output").textContent())!.replace("%", ""));
      await tree.getByRole("button", { name: "트리 확대" }).click();
      await expect.poll(async () => Number((await tree.locator(".skill-map-toolbar output").textContent())!.replace("%", ""))).toBeGreaterThan(before);
      expect((await tree.locator(".skill-map-toolbar").boundingBox())!.height).toBeLessThanOrEqual(44);
      await tree.getByRole("button", { name: "기본 보기" }).click();
      await expect(tree.locator(".skill-map-toolbar output")).toHaveText("56%");
    }
  }
});
