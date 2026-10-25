import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { PieceImage } from "./PieceImage";
import { RelicIcon } from "./ContentArt";
import type { RestOption, Snapshot } from "./types";

export function ExpeditionStatus({ state, onClose }: { state: Snapshot; onClose: () => void }) {
  const closeRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [selectedRelicId, setSelectedRelicId] = useState<string | null>(null);
  const groups = useMemo(() => {
    const byKind = new Map<number, RestOption[]>();
    for (const piece of state.rest_options) {
      const list = byKind.get(piece.kind) ?? [];
      list.push(piece);
      byKind.set(piece.kind, list);
    }
    return [...byKind.values()];
  }, [state.rest_options]);
  const selectedPiece = state.rest_options.find((piece) => piece.id === selectedId);
  const selectedGroup = selectedPiece && groups.find((group) => group[0].kind === selectedPiece.kind);
  const selectedRelic = state.relics.find((relic) => relic.id === selectedRelicId);
  const positions = new Map(state.next_battle_pieces.map((piece) => [piece.cell, piece]));
  // Upgrade keys such as "tempest" are shared by several piece families.
  // Walk backward from the permanent current kind to resolve the exact path.
  const upgradeSteps = selectedPiece?.upgrades.map((key) => ({ key, path: undefined as typeof state.upgrade_paths[number] | undefined })) ?? [];
  let upgradeKind = selectedPiece?.kind;
  for (let index = upgradeSteps.length - 1; index >= 0; index--) {
    const path = state.upgrade_paths.find((candidate) => candidate.key === upgradeSteps[index].key && candidate.to === upgradeKind);
    upgradeSteps[index].path = path;
    if (path) upgradeKind = path.from;
  }

  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement | null;
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    closeRef.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
      if (event.key !== "Tab" || !dialogRef.current) return;
      const focusable = [...dialogRef.current.querySelectorAll<HTMLButtonElement>("button:not(:disabled)")];
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      document.body.style.overflow = previousOverflow;
      window.removeEventListener("keydown", onKeyDown);
      if (previousFocus?.isConnected) previousFocus.focus();
    };
  }, [onClose]);

  return createPortal(
    <div className="expedition-status-overlay" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
      <div className="expedition-status-dialog" ref={dialogRef} role="dialog" aria-modal="true" aria-labelledby="expedition-status-title">
        <header className="expedition-status-heading">
          <div><p className="eyebrow">CHESSLAYER / ACT {state.floor === 1 ? "I" : "II"}</p><h2 id="expedition-status-title">원정 현황</h2></div>
          <button className="expedition-status-close" ref={closeRef} onClick={onClose} aria-label="원정 현황 닫기">닫기 ×</button>
        </header>
        <div className="expedition-status-content">
          <section className="status-section status-roster" aria-labelledby="status-roster-title">
            <h3 id="status-roster-title">현재 덱 구성 <span>{state.rest_options.length}</span></h3>
            <div className="status-piece-list">
              {groups.map((group) => {
                const example = group[0];
                const alive = group.filter((piece) => !piece.dead).length;
                const upgraded = group.filter((piece) => piece.upgrades.length > 0).length;
                return <button key={example.kind} className={`status-piece-group ${selectedGroup?.[0].kind === example.kind ? "is-selected" : ""}`}
                  onClick={() => setSelectedId(group[0].id)} aria-pressed={selectedGroup?.[0].kind === example.kind}>
                  <PieceImage pieceKey={example.key} owner={0} name={example.name} />
                  <span><strong>{example.name}</strong><small>{alive !== group.length ? `전투 가능 ${alive} / 전체 ${group.length}` : `보유 ${group.length}`}{upgraded > 0 ? ` · 강화 ${upgraded}` : ""}</small></span>
                  <b>×{group.length}</b>
                </button>;
              })}
            </div>
            {selectedPiece && selectedGroup && <div className="status-piece-detail">
              <div className="status-instance-list" aria-label={`${selectedPiece.name} 개별 기물`}>
                {selectedGroup.map((piece, index) => <button key={piece.id} className={piece.id === selectedPiece.id ? "is-selected" : ""}
                  onClick={() => setSelectedId(piece.id)} aria-pressed={piece.id === selectedPiece.id}>
                  {selectedPiece.name} {index + 1}{piece.dead ? " · 전투 불가" : ""}{piece.upgrades.length ? ` · 강화 ${piece.upgrades.length}` : ""}
                </button>)}
              </div>
              <strong>{selectedPiece.name}</strong>
              <p>{selectedPiece.description}</p>
              <small>{selectedPiece.dead ? "전투 불가" : "다음 전투 참가"} · 강화 {selectedPiece.upgrades.length}회</small>
              {selectedPiece.upgrades.length > 0 && <ol className="status-upgrade-history">
                {upgradeSteps.map(({ key, path }, index) => {
                  return <li key={`${key}-${index}`}>{path ? <><strong>{path.from_name} → {path.to_name}</strong><span>{path.effect}</span></> : <span>{key}</span>}</li>;
                })}
              </ol>}
            </div>}
          </section>
          <section className="status-section status-formation" aria-labelledby="status-formation-title">
            <h3 id="status-formation-title">전투 진입 포지션 <span>{state.next_battle_pieces.length}</span></h3>
            <div className="status-board" role="group" aria-label="다음 전투 시작 아군 배치">
              {Array.from({ length: 64 }, (_, cell) => {
                const piece = positions.get(cell);
                const square = `${"abcdefgh"[cell % 8]}${8 - Math.floor(cell / 8)}`;
                return <button key={cell} className={`status-square ${(cell + Math.floor(cell / 8)) % 2 ? "dark" : "light"} ${piece?.id === selectedId ? "is-selected" : ""}`}
                  data-cell={cell} aria-label={`${square}${piece ? ` 아군 ${piece.name}` : " 빈 칸"}`} disabled={!piece}
                  onClick={() => piece && setSelectedId(piece.id)}>
                  {piece && <PieceImage pieceKey={piece.key} owner={0} name={piece.name} />}
                </button>;
              })}
            </div>
            {state.phase === "combat" && state.relics.some((relic) => relic.id === "hard_disk") && <p className="status-formation-note">하드디스크 · 현재 생존 기물의 위치를 기준으로 표시합니다. 전투 종료 위치에 따라 다음 진입 배치가 바뀝니다.</p>}
            {(state.phase === "victory" || state.phase === "defeat") && <p className="status-formation-note">다음 전투가 없습니다.</p>}
          </section>
          <section className="status-section status-relics" aria-labelledby="status-relics-title">
            <h3 id="status-relics-title">보유 유물 <span>{state.relics.length}</span></h3>
            <div className="status-relic-list">
              {state.relics.map((relic) => <button key={relic.id} className={`status-relic ${selectedRelicId === relic.id ? "is-selected" : ""}`}
                title={relic.description} aria-pressed={selectedRelicId === relic.id}
                onMouseEnter={() => setSelectedRelicId(relic.id)} onFocus={() => setSelectedRelicId(relic.id)} onClick={() => setSelectedRelicId(relic.id)}>
                <RelicIcon id={relic.id} name={relic.name} /><span>{relic.name}</span>
              </button>)}
            </div>
            {selectedRelic && <div className="status-relic-detail"><strong>{selectedRelic.name}</strong><p>{selectedRelic.description}</p>{selectedRelic.status && <small>{selectedRelic.status}</small>}</div>}
          </section>
        </div>
      </div>
    </div>, document.body
  );
}
