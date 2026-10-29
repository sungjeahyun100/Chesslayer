use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "heretic",
    name: "이단 부대",
    title: "환희는 끝나지 않는다",
    description: "시체를 먹어 민첩을 얻고 허기로 적의 행동을 압박합니다. 카니발의 신호탄이 전장을 식재료로 물들입니다.",
    relic: Relic::CarnivalFlare,
    unlock_floor: 2,
};
