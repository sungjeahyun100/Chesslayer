use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "artillery",
    name: "포병대대",
    title: "마지막 포탄까지",
    description: "소총수·박격포·드론·전차가 탄약으로 사격합니다. 창고를 모두 잃으면 패배하며 탄약은 전투 사이에도 유지됩니다.",
    relic: Relic::FireControl,
    unlock_floor: 3,
};
