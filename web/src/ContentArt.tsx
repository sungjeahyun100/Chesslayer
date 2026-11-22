import type { RelicId } from "./types";

const cards = import.meta.glob<string>("./assets/cards/*.svg", {
  eager: true,
  query: "?url",
  import: "default",
});
const relics = import.meta.glob<string>("./assets/relics/*.svg", {
  eager: true,
  query: "?url",
  import: "default",
});

export function CardArt({ kind, name }: { kind: string; name: string }) {
  const src = cards[`./assets/cards/${kind}.svg`];
  return src ? <img className="card-art" src={src} alt={`${name} 일러스트`} draggable={false} /> : <TacticalIcon id={kind} name={name} className="card-art" />;
}

export function RelicIcon({ id, name, className = "" }: { id: RelicId; name: string; className?: string }) {
  const src = relics[`./assets/relics/${id}.svg`];
  return src ? <img className={`relic-icon ${className}`} src={src} alt={`${name} 아이콘`} draggable={false} /> : <TacticalIcon id={id} name={name} className={`relic-icon ${className}`} />;
}

const glyphs: Record<string, string> = {
  card: "M20 15H60V65H20ZM28 27H52M28 39H52M28 51H46",
  reinforcement: "M22 58V44H58V58M40 18V49M28 30L40 18L52 30M17 64H63",
  carnival_flare: "M29 61L44 33L52 38L39 65ZM48 14V26M61 20L55 29M33 18L40 28M48 34L38 32L42 22L49 27L56 26L58 35Z",
  silver_cutlery: "M19 15V35L26 42V65M26 15V35M33 15V35L26 42M55 15Q39 24 55 40V65M55 15V40",
  empty_goblet: "M24 15H56V32Q56 47 40 47Q24 47 24 32ZM40 47V64M28 65H52",
  torn_menu: "M20 14H61L56 27L63 39L55 53L61 66H20ZM28 25H47M28 36H46M28 47H44M28 58H46",
  last_plate: "M40 14A26 26 0 1 0 40 66A26 26 0 1 0 40 14M40 24A16 16 0 1 0 40 56A16 16 0 1 0 40 24M36 39H44",
  golden_tray: "M15 40H65L59 48H21ZM23 37Q24 19 40 19Q56 19 57 37M36 14H44M12 54H68M21 61H59",
  smiling_mask: "M19 19Q40 11 61 19L57 45Q40 70 23 45ZM27 28L33 32M47 32L53 28M29 43Q40 57 51 43",
  eternal_feast_bell: "M19 53L24 45V30Q24 17 40 17Q56 17 56 30V45L61 53ZM34 59Q40 68 46 59M36 13H44",
  fire_control: "M22 58L41 26L52 33L36 60ZM38 25L44 15L58 23L53 34M15 64H65M18 34L10 26M26 22L24 10",
  ballistic_computer: "M16 18H64V52H16ZM25 60H55M40 52V60M23 42L33 32L43 36L56 25",
  recon_scope: "M12 44L21 25L34 28L32 50ZM46 28L59 25L68 44L48 50ZM34 34H46M22 18V25M58 18V25",
  silencer: "M16 48L47 22L56 31L25 57ZM47 22L58 13L67 22L56 31M24 50L29 55M32 43L37 48M40 36L45 41",
};

function TacticalIcon({ id, name, className }: { id: string; name: string; className: string }) {
  const path = glyphs[id] ?? glyphs.card;
  return <svg className={className} viewBox="0 0 80 80" role="img" aria-label={`${name} 아이콘`}>
    <circle cx="40" cy="40" r="37" fill="#e9e9d9" />
    <path d={path} fill="none" stroke="#486244" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
  </svg>;
}
