use super::PieceDefinition;
use crate::PieceType;

pub const COMMANDO: PieceType = 74;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 50,
    description:
        "킹 행마와 전방 두 칸 도약. 전방 세 방향 네 칸 이내 사격, 기본 명중 90%, 거리마다 5%p 감소.",
    upgrades: &[super::UpgradeDefinition {
        target: super::ELITE_COMMANDO,
        key: "elite_commando",
        effect: "정예 특전대원(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(COMMANDO, "commando", "돌격 특전대원", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1);\ntake-move(0,2)".into()
}
