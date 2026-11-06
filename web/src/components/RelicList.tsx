import type { Relic } from "../types";
import { RelicIcon } from "../ContentArt";

export function RelicList({ relics }: { relics: Relic[] }) {
  return (
    <details className="relic-panel">
      <summary className="section-label">
        보유 유물 <span>{relics.length}</span>
      </summary>
      {relics.map((r) => (
        <div className="relic" key={r.id}>
          <RelicIcon id={r.id} name={r.name} />
          <div>
            <strong>{r.name}</strong>
            <p>{r.description}</p>
            {r.status && <small className="relic-status" role="status">{r.status}</small>}
          </div>
        </div>
      ))}
    </details>
  );
}
