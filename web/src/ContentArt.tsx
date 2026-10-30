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
  stimpack: "M26 49L48 27M43 20L55 32M48 16L59 27M20 54L27 47M26 40L35 49M35 31L44 40",
  body_slam: "M15 24L30 40L15 56M65 24L50 40L65 56M30 40H50M34 21L40 31L46 21M34 59L40 49L46 59",
  infantry_support: "M22 42V58H58V42M40 19V49M28 31L40 19L52 31M22 58H58",
  sacrifice: "M18 55H62L54 62H26ZM40 15L32 31L40 39L48 31ZM22 30L58 53M58 30L22 53",
  last_stand: "M18 19L40 12L62 19V40Q58 56 40 65Q22 56 18 40ZM30 28L50 48M50 28L30 48",
  excited_steps: "M20 45L29 31L35 37L29 51L20 54ZM43 30L52 16L58 22L52 36L43 39M18 63H32M43 48H58",
  bloody_invitation: "M16 24H64V58H16ZM16 24L40 44L64 24M40 15Q28 30 40 30Q52 30 40 15",
  dining_etiquette: "M20 16V32Q20 38 26 38V64M26 16V36M32 16V32Q32 38 26 38M54 16Q41 28 54 40V64M54 16V40",
  masquerade: "M14 28Q40 16 66 28L62 49L40 60L18 49ZM23 33L34 37L25 43ZM46 37L57 33L55 43M34 49Q40 54 46 49",
  clear_tablecloth: "M15 32H65L55 51H25ZM21 51L18 65M59 51L62 65M28 20H54M46 13L54 20L46 27",
  intermission_waltz: "M18 30Q40 5 62 30M52 19L62 30L48 32M62 50Q40 75 18 50M28 61L18 50L32 48",
  late_dinner: "M16 47H64M23 44Q24 25 40 25Q56 25 57 44M37 21H43M29 54L40 65L51 54",
  meal_time: "M14 40Q14 17 40 17Q65 17 65 40M23 42H57L52 57H28ZM40 24V36L49 41M30 64H50",
  delivered_supper: "M17 38H60M23 35Q25 21 39 21Q53 21 55 35M14 48H35L42 54L61 46M27 62L42 64L65 51",
  backstage_sacrifice: "M17 14H63V64H17M23 14Q42 34 23 55M57 14Q38 34 57 55M34 51L40 38L46 51Z",
  torn_mask: "M17 22L34 18L29 33L37 43L31 61L19 49ZM46 18L63 22L61 49L43 61L49 43L41 33ZM23 30L28 34M51 34L57 30",
  cracked_conscience: "M40 62L17 39Q7 18 26 16Q36 16 40 26Q44 16 54 16Q73 18 63 39ZM42 23L35 36L44 43L38 54",
  last_supper: "M12 43H68M18 43V65M62 43V65M25 43V56M55 43V56M22 32H32M48 32H58M37 17H43V32L40 37L37 32Z",
  endless_ecstasy: "M40 39Q16 11 14 39Q16 67 40 39Q64 11 66 39Q64 67 40 39M33 15L40 7L47 15M33 65L40 73L47 65",
  volley: "M16 58L48 26M29 63L61 31M12 44L44 12M38 12H44V18M42 26H48V32M55 31H61V37",
  frontline: "M14 20H66V60H14ZM30 20V60M48 20V60M14 40H66M20 32L25 27L30 32M54 47L59 52L64 47",
  aimed_shot: "M40 10V25M40 55V70M10 40H25M55 40H70M40 20A20 20 0 1 0 40 60A20 20 0 1 0 40 20M36 40H44M40 36V44",
  infiltration: "M18 62L28 39L22 31L41 14L49 21L37 33L47 41L62 28M17 48L28 39M47 41L40 61",
  smoke_screen: "M16 51Q6 42 18 34Q12 20 28 21Q35 7 47 20Q63 15 62 32Q76 39 64 51ZM22 60H59",
  extraction: "M16 18H64V64H16M40 52V28M29 39L40 28L51 39M23 56H57",
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
  const path = glyphs[id] ?? glyphs.aimed_shot;
  return <svg className={className} viewBox="0 0 80 80" role="img" aria-label={`${name} 아이콘`}>
    <circle cx="40" cy="40" r="37" fill="#e9e9d9" />
    <path d={path} fill="none" stroke="#486244" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
  </svg>;
}
