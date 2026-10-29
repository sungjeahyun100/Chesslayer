use super::PieceDefinition;
use crate::PieceType;

pub const TOAST_LORD: PieceType = 45;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description: "로얄. 전투당 1회, 세 칸 이내 아군이 시체를 포획하면 카드 1장을 뽑고 포획자에게 민첩 1을 추가합니다.",
    upgrades: &[
        super::UpgradeDefinition { target: super::ECSTATIC_LORD, key: "ecstatic_lord", effect: "환희의 군주(으)로 영구 강화합니다.", requirement: "살아 있는 기물 · 쉼터 행동 1회" }
    ],
    ..PieceDefinition::new(TOAST_LORD, "toast_lord", "축배의 군주", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1)".into()
}
