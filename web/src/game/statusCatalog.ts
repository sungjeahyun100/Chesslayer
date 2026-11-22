import { hungerStages, statusDefinitions, type StatusKind } from "./statusVisuals";

export type StatusCatalogEntry = {
  kind: string;
  name: string;
  icon: string;
  effect: string;
  duration?: string;
  stacking?: string;
  removal?: string;
};

// Only states that the current game can grant are listed. Keep the battle
// badge names and icons in statusVisuals; adding a new badge needs one entry here.
const activeBadgeKinds: StatusKind[] = [
  "ingredient", "blessing", "madness", "guilt", "agility",
  "incapacitated", "pest", "corpse_bags", "spirit_bomb", "death_threshold", "bell", "temporary", "ward",
];

const badgeRules: Partial<Record<StatusKind, Pick<StatusCatalogEntry, "duration" | "stacking" | "removal">>> = {
  ingredient: { duration: "이번 전투 동안 유지됩니다." },
  blessing: { duration: "살아남은 아군은 다음 전투에도 지닙니다.", stacking: "다시 부여해도 효과가 늘어나지 않습니다.", removal: "SCP-500으로 해제할 수 있습니다." },
  madness: { duration: "살아남은 아군은 다음 전투에도 지닙니다.", stacking: "시체를 먹을 때 쌓이며, 3중첩에서 발동하고 수치가 사라집니다.", removal: "3중첩 발동 또는 SCP-500 사용 시 사라집니다." },
  guilt: { duration: "살아남은 아군은 다음 전투에도 지닙니다.", stacking: "시체를 먹을 때 쌓이며, 2중첩에서 발동하고 수치가 사라집니다.", removal: "2중첩 발동 또는 SCP-500 사용 시 사라집니다." },
  agility: { duration: "해당 기물의 진영 턴이 끝날 때 남은 수치가 사라집니다.", stacking: "여러 번 얻으면 수치가 합쳐집니다.", removal: "추가 행동에 사용하거나 해당 진영 턴이 끝나면 사라집니다." },
  incapacitated: { duration: "현재 층을 벗어날 때까지 전투 사이에도 유지됩니다.", stacking: "중첩 수만큼 자기 턴이 끝날 때 1씩 감소합니다.", removal: "중첩이 0이 되거나 층을 벗어나면 해제됩니다." },
  pest: { duration: "중첩이 0이 될 때까지 유지됩니다.", stacking: "감염 시 페스트 3이 됩니다.", removal: "0이 되면 기물이 사망합니다." },
  corpse_bags: { duration: "이번 전투 동안 기물이 보유합니다.", stacking: "시체 포획마다 1 증가합니다.", removal: "특수능력 사용 시 1 감소합니다." },
  spirit_bomb: { duration: "이번 전투 동안 유지됩니다.", removal: "전투 종료 시 사라집니다." },
  death_threshold: { duration: "다음 전투 시작 시 발동합니다.", stacking: "중복 적용해도 한 번만 발동합니다.", removal: "다음 전투 시작 시 사망하거나 SCP-500으로 해제하면 사라집니다." },
  bell: { duration: "다음 종소리가 울릴 때까지 유지됩니다.", stacking: "같은 종소리로 여러 번 지정되지 않습니다.", removal: "종소리가 발동하거나 대상이 사망하면 사라집니다. 종소리를 울리는 기물이 없어져도 다음 라운드에 해제됩니다." },
  temporary: { duration: "이번 전투 동안 유지됩니다.", removal: "기물이 사망하거나 전투가 끝나면 사라집니다." },
  ward: { duration: "이번 전투에서 한 번 막거나 전투가 끝날 때까지 유지됩니다.", stacking: "다시 부여해도 한 번만 막습니다.", removal: "직접 포획 또는 명중한 사격을 막으면 소모됩니다." },
};

const hungerRemoval = "시체를 먹거나 식당 이벤트, SCP-500으로 해소할 수 있습니다.";
const hungerEntries: StatusCatalogEntry[] = hungerStages.map((stage, index) => ({
  kind: `hunger_${index + 1}`,
  name: stage.name,
  icon: statusDefinitions.hunger.icon,
  effect: stage.description,
  duration: index < 2
    ? `살아남은 아군은 전투 종료 시 ${hungerStages[index + 1].label} 단계로 악화됩니다. 악화를 막으면 현재 상태를 유지합니다.`
    : "살아남은 아군은 다음 전투에도 기아 상태이며 더 심해지지 않습니다.",
  stacking: index < 2
    ? `허기가 한 단계 더 심해지면 ${hungerStages[index + 1].label} 단계로 바뀝니다.`
    : "허기가 더 쌓이지 않습니다.",
  removal: hungerRemoval,
}));

const badges: StatusCatalogEntry[] = activeBadgeKinds.flatMap((kind) => {
  const badge = {
    kind,
    name: statusDefinitions[kind].name,
    icon: statusDefinitions[kind].icon,
    effect: statusDefinitions[kind].description,
    ...badgeRules[kind],
  };
  return kind === "blessing" ? [badge, ...hungerEntries] : [badge];
});

// These effects have their own marks on the battle piece instead of a status badge.
const pieceMarks: StatusCatalogEntry[] = [
  { kind: "plague", name: "역병", icon: "☣", effect: "칸에 들어온 양측 기물에 식재료와 페스트 3을 부여합니다.", duration: "2라운드 유지됩니다.", removal: "지속시간이 끝나면 사라집니다." },
  { kind: "move_rights", name: "행마권", icon: "↪", effect: "수치만큼 카드로 기물을 움직일 수 있습니다. 적 기물에게도 부여할 수 있습니다.", duration: "플레이어 턴 종료 시 남은 수치가 사라집니다. 원기옥 상태의 기물은 이번 전투 동안 다음 턴으로 이월합니다.", stacking: "여러 번 부여하면 수치가 합쳐집니다.", removal: "카드 이동에 사용하거나 원기옥이 없는 기물은 플레이어 턴이 끝나면 사라집니다." },
  { kind: "disarmed", name: "비무장", icon: "비무장", effect: "상대 기물을 포획하는 행동을 할 수 없습니다.", duration: "해당 기물 진영의 턴 종료 시 해제됩니다.", stacking: "중복 적용해도 효과는 같습니다." },
  { kind: "self_defense", name: "호신용품", icon: "◆", effect: "보호 대상이 적에게 직접 포획되어 사망하면 포획한 적도 사망합니다.", duration: "이번 전투 동안 유지됩니다.", removal: "보호 대상이 사망하거나 전투가 끝나면 사라집니다." },
];

export const statusCatalog: StatusCatalogEntry[] = [...badges, ...pieceMarks];
