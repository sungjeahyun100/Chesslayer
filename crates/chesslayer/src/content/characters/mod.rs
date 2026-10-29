//! The current characters differ only by their starting relic.
mod guardian;
mod vanguard;
// Wanderer belonged to the v0.2 prototype and is not a v0.3 playable squad.

use super::Relic;
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub struct Character {
    pub id: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub relic: Relic,
}
pub const ALL: &[Character] = &[
    guardian::DEFINITION,
    vanguard::DEFINITION,
    Character {
        id: "heretic", name: "이단 부대", title: "끝나지 않는 연회",
        description: "시체와 허기 시스템을 위한 플레이어블 슬롯입니다. 전용 효과 적용 전까지 소용돌이를 임시 시작 유물로 사용합니다.",
        relic: Relic::Whirlwind,
    },
    Character {
        id: "artillery", name: "포병대대", title: "사거리 밖에서",
        description: "전용 소총·포격 기물 구현 전까지 선공권을 임시 시작 유물로 사용합니다.",
        relic: Relic::Initiative,
    },
    Character {
        id: "special_ops", name: "특수부대", title: "그림자 속의 한 수",
        description: "전용 사격·잠입 기물 구현 전까지 아이스크림을 임시 시작 유물로 사용합니다.",
        relic: Relic::IceCream,
    },
];

pub fn characters() -> Vec<Character> {
    ALL.to_vec()
}
