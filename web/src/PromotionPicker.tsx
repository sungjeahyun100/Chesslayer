import { PieceImage } from "./PieceImage";
import type { Promotion } from "./types";

export function PromotionPicker({
  options,
  square,
  onChoose,
  onCancel,
}: {
  options: Promotion[];
  square: string;
  onChoose: (kind: number) => void;
  onCancel: () => void;
}) {
  return (
    <section className="promotion-picker" role="dialog" aria-label="승격 선택">
      <div className="promotion-heading">
        <div>
          <strong>{square} · 승격할 기물</strong>
          <p>선택하면 이동과 승격을 확정합니다.</p>
        </div>
        <button
          className="text-button"
          onClick={onCancel}
          aria-label="승격 선택 취소"
        >
          취소 · Esc
        </button>
      </div>
      <div className="promotion-options">
        {options.map((p, index) => (
          <button
            key={p.kind}
            autoFocus={index === 0}
            onClick={() => onChoose(p.kind)}
            title={p.description}
            aria-label={`${p.name}으로 승격`}
          >
            <PieceImage pieceKey={p.key} name={p.name} />
            <span>{p.name}</span>
          </button>
        ))}
      </div>
    </section>
  );
}
