import { useEffect, useLayoutEffect, useRef, type RefObject } from "react";
import type { Snapshot } from "../types";

type Movement = { sequence: number; before: Snapshot; next: Snapshot } | null;
type CardPosition = { rect: DOMRect; face: HTMLElement };
type Pile = "draw" | "discard" | "exhausted";

function pileOf(state: Snapshot, id: number): Pile | null {
  if (state.draw_pile.some((card) => card.id === id)) return "draw";
  if (state.discard_pile.some((card) => card.id === id)) return "discard";
  if (state.exhausted_pile.some((card) => card.id === id)) return "exhausted";
  return null;
}

function rects(root: HTMLElement): Map<number, CardPosition> {
  return new Map([...root.querySelectorAll<HTMLElement>(".hand [data-card-id]")].map((element) => {
    const face = element.cloneNode(true) as HTMLElement;
    face.style.visibility = "visible";
    return [Number(element.dataset.cardId), { rect: element.getBoundingClientRect(), face }];
  }));
}

export function useCardMotion(root: RefObject<HTMLElement | null>, state: Snapshot, movement: Movement) {
  const previous = useRef<Map<number, CardPosition>>(new Map());
  const seen = useRef<number | null>(null);
  const running = useRef(new Set<() => void>());
  useEffect(() => () => {
    [...running.current].forEach((cancel) => cancel());
    seen.current = null;
  }, []);

  useLayoutEffect(() => {
    const area = root.current;
    if (!area) return;
    const current = rects(area);
    const beforePositions = previous.current;
    previous.current = current;
    if (!movement || movement.sequence === seen.current || movement.next !== state) return;
    seen.current = movement.sequence;
    const { before, next } = movement;
    if (next.phase !== "combat" || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const oldHand = new Set(before.hand.map((card) => card.id));
    const newHand = new Set(next.hand.map((card) => card.id));
    const entering = next.hand.filter((card) => !oldHand.has(card.id));
    const leaving = before.hand.filter((card) => !newHand.has(card.id));
    const reshuffled = before.draw_count === 0 && before.discard_count > 0 && next.discard_count < before.discard_count;
    if (!entering.length && !leaving.length && !reshuffled) return;

    const layer = document.createElement("div");
    layer.className = "card-motion-layer";
    layer.setAttribute("aria-hidden", "true");
    document.body.append(layer);
    const animations: Animation[] = [];
    const restores: (() => void)[] = [];
    const reroutes: (() => void)[] = [];
    let stopped = false;
    let active = 0;
    let cleanup = () => {};
    const anchor = (pile: Pile) => area.querySelector<HTMLElement>(`[data-pile-anchor="${pile}"]`);
    const finish = (ghost: HTMLElement, reveal?: HTMLElement) => {
      if (!ghost.isConnected) return;
      ghost.remove();
      if (reveal) reveal.style.visibility = "";
      active--;
      if (active === 0) cleanup();
    };
    const fly = (face: HTMLElement, from: DOMRect, target: () => DOMRect | undefined,
      options: { delay?: number; duration?: number; draw?: boolean; appear?: boolean; reveal?: HTMLElement; vanish?: boolean } = {}) => {
      const ghost = document.createElement("div");
      ghost.className = "card-flight";
      ghost.style.left = `${from.left}px`;
      ghost.style.top = `${from.top}px`;
      ghost.style.width = `${from.width}px`;
      ghost.style.height = `${from.height}px`;
      if (options.draw) {
        const inner = document.createElement("div");
        inner.className = "card-flight-inner";
        face.classList.add("card-flight-front");
        const back = document.createElement("div");
        back.className = "card-flight-back";
        inner.append(face, back);
        ghost.append(inner);
        const flip = inner.animate([{ transform: "rotateY(180deg)" }, { transform: "rotateY(180deg)", offset: .35 },
          { transform: "rotateY(0deg)" }], { duration: options.duration ?? 480, delay: options.delay ?? 0, easing: "ease-in-out", fill: "both" });
        animations.push(flip);
      } else ghost.append(face);
      layer.append(ghost);
      active++;
      const start = performance.now();
      let animation: Animation | null = null;
      const route = () => {
        if (stopped || !ghost.isConnected) return;
        const destination = target();
        if (!destination) { finish(ghost, options.reveal); return; }
        const currentRect = ghost.getBoundingClientRect();
        animation?.cancel();
        ghost.style.left = `${currentRect.left}px`;
        ghost.style.top = `${currentRect.top}px`;
        ghost.style.width = `${currentRect.width}px`;
        ghost.style.height = `${currentRect.height}px`;
        const elapsed = performance.now() - start;
        const remaining = Math.max(80, (options.duration ?? 430) - Math.max(0, elapsed - (options.delay ?? 0)));
        const dx = destination.left - currentRect.left;
        const dy = destination.top - currentRect.top;
        const sx = destination.width / Math.max(1, currentRect.width);
        const sy = destination.height / Math.max(1, currentRect.height);
        animation = ghost.animate([
          { transform: options.appear ? "translate(0, 0) scale(.6)" : "translate(0, 0) scale(1)",
            opacity: options.appear ? 0 : 1, offset: 0 },
          { transform: `translate(${dx * .26}px, ${dy * .18 - (options.draw ? 18 : 24)}px) scale(${1 + (sx - 1) * .26}, ${1 + (sy - 1) * .26})`, opacity: 1, offset: .25 },
          { transform: `translate(${dx}px, ${dy}px) scale(${sx}, ${sy})`, opacity: options.vanish ? 0 : 1, offset: 1 },
        ], { duration: remaining, delay: Math.max(0, (options.delay ?? 0) - elapsed), easing: "cubic-bezier(.2,.7,.24,1)", fill: "forwards" });
        animations.push(animation);
        animation.onfinish = () => finish(ghost, options.reveal);
      };
      route();
      reroutes.push(route);
      if (options.reveal) {
        options.reveal.style.visibility = "hidden";
        restores.push(() => { options.reveal!.style.visibility = ""; });
      }
    };

    // The real hand has already committed its new layout. FLIP the survivors into place.
    for (const [id, position] of current) {
      const old = beforePositions.get(id);
      const node = area.querySelector<HTMLElement>(`.hand [data-card-id="${id}"]`);
      if (!old || !node || entering.some((card) => card.id === id)) continue;
      const dx = old.rect.left - position.rect.left;
      const dy = old.rect.top - position.rect.top;
      if (Math.abs(dx) + Math.abs(dy) < 2) continue;
      animations.push(node.animate([{ transform: `translate(${dx}px, ${dy}px)` }, { transform: "translate(0, 0)" }],
        { duration: 300, easing: "cubic-bezier(.2,.7,.24,1)" }));
    }

    if (reshuffled) {
      const from = anchor("discard")?.getBoundingClientRect();
      if (from) {
        const back = document.createElement("div");
        back.className = "card-flight-back pile-transfer-back";
        fly(back, from, () => anchor("draw")?.getBoundingClientRect(), { duration: 340 });
      }
    }

    entering.forEach((card, index) => {
      const destination = area.querySelector<HTMLElement>(`.hand [data-card-id="${card.id}"]`);
      if (!destination) return;
      const sourcePile = before.phase !== "combat" || (reshuffled && pileOf(before, card.id) === "discard")
        ? "draw" : pileOf(before, card.id);
      const source = sourcePile ? anchor(sourcePile)?.getBoundingClientRect() : destination.getBoundingClientRect();
      if (!source) return;
      const face = destination.cloneNode(true) as HTMLElement;
      face.style.visibility = "visible";
      fly(face, source, () => destination.getBoundingClientRect(), {
        delay: index * 90 + (reshuffled && pileOf(before, card.id) === "discard" ? 340 : 0),
        duration: 480, draw: !!sourcePile, appear: !sourcePile, reveal: destination,
      });
    });
    leaving.forEach((card, index) => {
      const old = beforePositions.get(card.id);
      if (!old) return;
      const destinationPile = pileOf(next, card.id);
      fly(old.face, old.rect, () => destinationPile ? anchor(destinationPile)?.getBoundingClientRect() : old.rect,
        { delay: index * 80, duration: 420, vanish: !destinationPile });
    });

    const onGeometry = () => reroutes.forEach((route) => route());
    window.addEventListener("resize", onGeometry);
    window.addEventListener("scroll", onGeometry, true);
    cleanup = () => {
      if (stopped) return;
      stopped = true;
      window.removeEventListener("resize", onGeometry);
      window.removeEventListener("scroll", onGeometry, true);
      animations.forEach((animation) => animation.cancel());
      restores.forEach((restore) => restore());
      layer.remove();
      running.current.delete(cleanup);
    };
    running.current.add(cleanup);
    if (active === 0) window.setTimeout(cleanup, 320);
  });
}
