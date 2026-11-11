import { PieceImage } from "./PieceImage";
import type { Promotion } from "./types";

export function PromotionPicker({
  options,
  square,
  onChoose,
  onCancel,
  owner = 0,
  title,
  description = "선택하면 이동과 승격을 확정합니다.",
  choiceLabel = "승격",
  cancelLabel = "취소 · Esc",
  className = "",
}: {
  options: Promotion[];
  square: string;
  onChoose: (kind: number) => void;
  onCancel: () => void;
  owner?: number;
  title?: string;
  description?: string | null;
  choiceLabel?: string;
  cancelLabel?: string;
  className?: string;
}) {
  return (
    <section className={`promotion-picker ${className}`} role="dialog" aria-label={`${choiceLabel} 선택`}>
      <div className="promotion-heading">
        <div>
          <strong>{title ?? `${square} · 승격할 기물`}</strong>
          {description && <p>{description}</p>}
        </div>
        <button
          type="button"
          className="text-button"
          onClick={onCancel}
          aria-label={`${choiceLabel} 선택 취소`}
        >
          {cancelLabel}
        </button>
      </div>
      <div className="promotion-options">
        {options.map((p, index) => (
          <button
            type="button"
            key={p.kind}
            data-kind={p.kind}
            autoFocus={index === 0}
            onClick={() => onChoose(p.kind)}
            title={p.description}
            aria-label={`${p.name}으로 ${choiceLabel}`}
          >
            <PieceImage pieceKey={p.key} owner={owner} name={p.name} />
            <span>{p.name}</span>
          </button>
        ))}
      </div>
    </section>
  );
}
