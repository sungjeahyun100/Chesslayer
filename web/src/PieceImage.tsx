import { useState } from "react";

// Vite bundles only this curated set. Every surface uses the engine's stable piece key.
const assets = import.meta.glob<string>("./assets/pieces/*.svg", {
  eager: true,
  query: "?url",
  import: "default",
});
export function pieceImage(key: string, owner: number): string | undefined {
  if (key === "corpse") return assets["./assets/pieces/corpse.svg"];
  if (owner !== 0 && owner !== 1) return undefined;
  return assets[
    `./assets/pieces/${key}-${owner === 0 ? "white" : "black"}.svg`
  ];
}
export function PieceImage({
  pieceKey,
  owner = 0,
  name = "",
  className = "",
}: {
  pieceKey: string;
  owner?: number;
  name?: string;
  className?: string;
}) {
  const src = pieceImage(pieceKey, owner);
  const [failed, setFailed] = useState<string>();
  if (!src || failed === src)
    return (
      <span
        className={`missing-piece ${className}`}
        role="img"
        aria-label={`기물 이미지 누락: ${pieceKey}/${owner}`}
      >
        이미지 누락
        <br />
        {pieceKey}/{owner}
      </span>
    );
  return (
    <img
      className={`piece-image ${className}`}
      src={src}
      alt={name}
      draggable={false}
      data-piece-key={pieceKey}
      onError={() => setFailed(src)}
    />
  );
}
