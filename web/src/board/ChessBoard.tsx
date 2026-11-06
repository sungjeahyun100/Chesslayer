import { Fragment, type ComponentProps, type ReactNode } from "react";

export const squareName = (cell: number) => `${"abcdefgh"[cell % 8]}${8 - Math.floor(cell / 8)}`;

type ChessBoardProps = Omit<ComponentProps<"div">, "children"> & {
  renderSquare: (cell: number) => ReactNode;
  paths?: ReactNode;
  pieces?: ReactNode;
  annotations?: ReactNode;
  effects?: ReactNode;
};

// The same board surface serves combat, rest/upgrade and the sandbox. Callers
// supply interaction and presentation; this component never reads an engine.
export function ChessBoard({ renderSquare, paths, pieces, annotations, effects, className = "", ...props }: ChessBoardProps) {
  return <div {...props} className={`board ${className}`}>
    {Array.from({ length: 64 }, (_, cell) => <Fragment key={cell}>{renderSquare(cell)}</Fragment>)}
    {paths}
    {pieces}
    {annotations}
    {effects}
  </div>;
}

export function BoardSquare({ cell, className = "", ...props }: ComponentProps<"button"> & { cell: number }) {
  return <button {...props} data-square={squareName(cell)} className={`square ${(cell + Math.floor(cell / 8)) % 2 ? "dark" : "light"} ${className}`} />;
}
