use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "guardian",
    name: "수호자 부대",
    title: "희생에도 대가를",
    description:
        "기존 체스 기물과 분기 강화 트리, 가호와 성역 카드로 왕을 지키는 최초의 원정부대입니다.",
    relic: Relic::SelfDefense,
    unlock_floor: 0,
};
