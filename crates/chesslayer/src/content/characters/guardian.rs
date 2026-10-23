use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "guardian",
    name: "수호자",
    title: "희생에도 대가를",
    description: "호신용품으로 역습에 반격합니다. 로얄은 반드시 직접 포획해야 합니다.",
    relic: Relic::SelfDefense,
};
