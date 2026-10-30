use super::PieceDefinition;
use crate::PieceType;

pub const SPECIAL_COMMANDER: PieceType = 80;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 80,
    description: "로얄. 킹 행마. 여덟 방향 다섯 칸 이내 사격, 명중률 95%에서 거리당 4%p 감소. 매 전투 시작 시 가호를 얻습니다.",
    upgrades: &[
        super::UpgradeDefinition { target: super::SHADOW_COMMANDER, key: "shadow_commander", effect: "그림자 지휘관(으)로 영구 강화합니다.", requirement: "살아 있는 기물 · 쉼터 행동 1회" }
    ],
    ..PieceDefinition::new(SPECIAL_COMMANDER, "special_commander", "특수부대 지휘관", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1)".into()
}
