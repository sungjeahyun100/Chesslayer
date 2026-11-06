import { useEffect, useRef, useState, type MouseEvent, type PointerEvent } from "react";

export type BoardAnnotation = { from: number; to: number };

function boardCellAt(board: HTMLDivElement, clientX: number, clientY: number): number | null {
  const bounds = board.getBoundingClientRect();
  const x = (clientX - bounds.left - board.clientLeft) / board.clientWidth;
  const y = (clientY - bounds.top - board.clientTop) / board.clientHeight;
  if (x < 0 || x >= 1 || y < 0 || y >= 1) return null;
  return Math.floor(y * 8) * 8 + Math.floor(x * 8);
}

export function useBoardAnnotations(combat: boolean, enabled: boolean, revision: number) {
  const [boardAnnotations, setBoardAnnotations] = useState<BoardAnnotation[]>([]);
  const [annotationPreview, setAnnotationPreview] = useState<BoardAnnotation | null>(null);
  const annotationDrag = useRef<{ pointerId: number; from: number } | null>(null);
  useEffect(() => {
    setBoardAnnotations([]);
    setAnnotationPreview(null);
    annotationDrag.current = null;
  }, [combat, revision]);
  useEffect(() => {
    if (!enabled) { annotationDrag.current = null; setAnnotationPreview(null); }
  }, [enabled]);
  function startBoardAnnotation(event: PointerEvent<HTMLDivElement>) {
    if (!combat || !enabled || event.button !== 2) return;
    const from = boardCellAt(event.currentTarget, event.clientX, event.clientY);
    if (from === null) return;
    event.preventDefault();
    annotationDrag.current = { pointerId: event.pointerId, from };
    setAnnotationPreview({ from, to: from });
    event.currentTarget.setPointerCapture(event.pointerId);
  }
  function moveBoardAnnotation(event: PointerEvent<HTMLDivElement>) {
    const drag = annotationDrag.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const to = boardCellAt(event.currentTarget, event.clientX, event.clientY);
    setAnnotationPreview(to === null ? null : { from: drag.from, to });
  }
  function endBoardAnnotation(event: PointerEvent<HTMLDivElement>) {
    const drag = annotationDrag.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const to = boardCellAt(event.currentTarget, event.clientX, event.clientY);
    if (to !== null) {
      setBoardAnnotations((current) => current.some((mark) => mark.from === drag.from && mark.to === to)
        ? current.filter((mark) => mark.from !== drag.from || mark.to !== to)
        : [...current, { from: drag.from, to }]);
    }
    annotationDrag.current = null;
    setAnnotationPreview(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
  }
  function cancelBoardAnnotation(event: PointerEvent<HTMLDivElement>) {
    if (annotationDrag.current?.pointerId !== event.pointerId) return;
    annotationDrag.current = null;
    setAnnotationPreview(null);
  }
  return { boardAnnotations, annotationPreview, handlers: {
    onContextMenu: (event: MouseEvent<HTMLDivElement>) => { if (combat) event.preventDefault(); },
    onPointerDown: startBoardAnnotation, onPointerMove: moveBoardAnnotation,
    onPointerUp: endBoardAnnotation, onPointerCancel: cancelBoardAnnotation,
  } };
}
