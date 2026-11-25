import { useEffect, useRef, useState, type PointerEvent, type RefObject } from "react";
import { createPortal } from "react-dom";
import type { Card, Piece, Snapshot } from "../types";
import type { GameInput } from "./useGameInput";
import { PieceImage } from "../PieceImage";

const threshold = 7;
type Source = { kind: "card"; card: Card } | { kind: "piece"; piece: Piece };
type Pending = Source & { pointerId: number; x: number; y: number; rect: DOMRect; active: boolean };
type Drag = Source & { x: number; y: number; offsetX: number; offsetY: number; width: number; height: number;
  originX: number; originY: number; cell: number | null; valid: boolean; returning?: boolean };

function boardCell(board: HTMLDivElement | null, x: number, y: number): number | null {
  const surface = board?.querySelector<HTMLElement>(".board");
  if (!surface) return null;
  const rect = surface.getBoundingClientRect();
  const left = rect.left + surface.clientLeft;
  const top = rect.top + surface.clientTop;
  const file = Math.floor((x - left) / surface.clientWidth * 8);
  const rank = Math.floor((y - top) / surface.clientHeight * 8);
  return file >= 0 && file < 8 && rank >= 0 && rank < 8 ? rank * 8 + file : null;
}

export function useBoardDrag(state: Snapshot, input: GameInput, boardRef: RefObject<HTMLDivElement | null>) {
  const [drag, setDrag] = useState<Drag | null>(null);
  const pending = useRef<Pending | null>(null);
  const latest = useRef({ state, input });
  const returnTimer = useRef<number | null>(null);
  const suppressClick = useRef(false);
  const cancelledPointer = useRef<number | null>(null);
  latest.current = { state, input };

  function cancel(animate = false) {
    const prior = pending.current;
    pending.current = null;
    if (returnTimer.current !== null) window.clearTimeout(returnTimer.current);
    if (animate && prior?.active) {
      setDrag((current) => current ? { ...current, returning: true,
        x: prior.rect.left + current.offsetX, y: prior.rect.top + current.offsetY, cell: null } : null);
      returnTimer.current = window.setTimeout(() => setDrag(null), 210);
    } else setDrag(null);
  }
  function start(event: PointerEvent<HTMLElement>, source: Source) {
    if (event.button !== 0 || pending.current || !latest.current.input.policy.act) return;
    if (source.kind === "piece" && latest.current.input.input.mode !== "idle" && latest.current.input.input.mode !== "piece") return;
    if (source.kind === "piece" && (source.piece.owner !== 0 || !state.moves.some((move) => move.from === source.piece.cell))) return;
    cancelledPointer.current = null;
    const rect = event.currentTarget.getBoundingClientRect();
    pending.current = { ...source, pointerId: event.pointerId, x: event.clientX, y: event.clientY, rect, active: false };
  }
  function validAt(source: Source, cell: number | null) {
    if (cell === null) return false;
    const { state: current, input: controller } = latest.current;
    if (!controller.policy.act) return false;
    if (source.kind === "piece") return cell !== source.piece.cell && current.moves.some((move) => move.from === source.piece.cell && move.to === cell);
    if (source.card.disabled_reason || !current.hand.some((card) => card.id === source.card.id)) return false;
    if (!source.card.definition.targeted) return true;
    const piece = current.pieces.find((entry) => entry.cell === cell);
    return source.card.targets.includes(piece?.id ?? 10000 + cell);
  }
  useEffect(() => {
    const move = (event: globalThis.PointerEvent) => {
      const candidate = pending.current;
      if (!candidate || event.pointerId !== candidate.pointerId) return;
      if (!candidate.active && Math.hypot(event.clientX - candidate.x, event.clientY - candidate.y) < threshold) return;
      candidate.active = true;
      event.preventDefault();
      const cell = boardCell(boardRef.current, event.clientX, event.clientY);
      setDrag({ kind: candidate.kind, ...(candidate.kind === "card" ? { card: candidate.card } : { piece: candidate.piece }),
        x: event.clientX, y: event.clientY, offsetX: candidate.x - candidate.rect.left,
        offsetY: candidate.y - candidate.rect.top, width: candidate.rect.width, height: candidate.rect.height,
        originX: candidate.rect.left + candidate.rect.width / 2,
        originY: candidate.rect.top + candidate.rect.height / 2,
        cell, valid: validAt(candidate, cell) } as Drag);
    };
    const up = (event: globalThis.PointerEvent) => {
      if (cancelledPointer.current === event.pointerId) {
        cancelledPointer.current = null;
        suppressClick.current = true;
        window.setTimeout(() => { suppressClick.current = false; }, 0);
        return;
      }
      const candidate = pending.current;
      if (!candidate || event.pointerId !== candidate.pointerId) return;
      if (!candidate.active) { pending.current = null; return; }
      suppressClick.current = true;
      window.setTimeout(() => { suppressClick.current = false; }, 0);
      const cell = boardCell(boardRef.current, event.clientX, event.clientY);
      const valid = validAt(candidate, cell);
      if (valid && cell !== null) {
        const { input: controller } = latest.current;
        const accepted = candidate.kind === "card" ? controller.dropCard(candidate.card, cell)
          : controller.dropPiece(candidate.piece.cell, cell);
        if (accepted) { cancel(); return; }
      }
      if (candidate.kind === "card") latest.current.input.cancelCard();
      cancel(true);
    };
    const pointerCancel = (event: globalThis.PointerEvent) => {
      if (pending.current?.pointerId === event.pointerId) {
        cancel(true);
      }
    };
    const key = (event: KeyboardEvent) => {
      if (event.key === "Escape" && pending.current) {
        event.preventDefault();
        cancelledPointer.current = pending.current.pointerId;
        cancel(true);
      }
    };
    const click = (event: MouseEvent) => {
      if (!suppressClick.current && !pending.current?.active) return;
      suppressClick.current = false;
      event.preventDefault(); event.stopImmediatePropagation();
    };
    window.addEventListener("pointermove", move, { passive: false });
    window.addEventListener("pointerup", up);
    window.addEventListener("pointercancel", pointerCancel);
    window.addEventListener("keydown", key, true);
    window.addEventListener("click", click, true);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      window.removeEventListener("pointercancel", pointerCancel);
      window.removeEventListener("keydown", key, true);
      window.removeEventListener("click", click, true);
    };
  }, []);
  useEffect(() => {
    if (!input.policy.act || state.phase !== "combat") cancel();
  }, [input.policy.act, state.phase]);
  useEffect(() => { cancel(); }, [state]);
  useEffect(() => () => { if (returnTimer.current !== null) window.clearTimeout(returnTimer.current); }, []);

  const overlay = drag && createPortal(drag.kind === "piece"
    ? <div className={`board-drag-visual piece ${drag.valid ? "valid" : "invalid"} ${drag.returning ? "returning" : ""}`}
        style={{ left: drag.x - drag.offsetX, top: drag.y - drag.offsetY, width: drag.width, height: drag.height }} aria-hidden="true">
        <PieceImage pieceKey={drag.piece.key} owner={drag.piece.owner} />
      </div>
    : drag.card.definition.targeted && !drag.returning
      ? <svg className={`card-drag-arrow ${drag.cell === null ? "pending" : drag.valid ? "valid" : "invalid"}`}
          viewBox={`0 0 ${window.innerWidth} ${window.innerHeight}`} preserveAspectRatio="none" aria-hidden="true">
          <defs><marker id="card-drag-arrowhead" markerWidth="12" markerHeight="12" refX="10" refY="6"
            viewBox="0 0 12 12" orient="auto" markerUnits="userSpaceOnUse"><path d="M 0 0 L 12 6 L 0 12 Z" /></marker></defs>
          <path d={`M ${drag.originX} ${drag.originY} C ${drag.originX} ${Math.min(drag.originY, drag.y) - 35}, ${drag.x} ${Math.min(drag.originY, drag.y) - 35}, ${drag.x} ${drag.y}`}
            markerEnd="url(#card-drag-arrowhead)" />
        </svg>
      : null, document.body);
  return { drag, overlay,
    startCard: (event: PointerEvent<HTMLElement>, card: Card) => start(event, { kind: "card", card }),
    startPiece: (event: PointerEvent<HTMLElement>, piece: Piece) => start(event, { kind: "piece", piece }) };
}

export type BoardDrag = ReturnType<typeof useBoardDrag>;
