import { PromotionPicker } from "./PromotionPicker";
import type { Card, Piece, UpgradePath } from "./types";

export function CardUpgradePanel({ card, piece, paths, onConfirm, onCancel }: {
  card: Card;
  piece: Piece;
  paths: UpgradePath[];
  onConfirm: (target: number) => void;
  onCancel: () => void;
}) {
  // Rust supplies the card's legal results; paths supply their labels and art.
  const legal = new Set(card.upgrade_options.filter((option) => option.id === piece.id).map((option) => option.target));
  const options = paths.filter((path) => path.from === piece.kind && legal.has(path.to))
    .map((path) => ({ kind: path.to, key: path.to_key, name: path.to_name, description: path.effect }));

  return <PromotionPicker
    options={options}
    square={piece.square}
    owner={piece.owner}
    title={`${card.definition.name} · ${piece.name} ${piece.square}`}
    description={null}
    choiceLabel="일시 강화"
    cancelLabel="대상 재선택"
    className="card-upgrade-picker"
    onChoose={onConfirm}
    onCancel={onCancel}
  />;
}
