use super::PieceDefinition;
use crate::PieceType;

pub const INVITING_FOLLOWER: PieceType = 66;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    description: "굶주린 신도 행마 유지. 전투당 1회, 시체 포획 시 두 칸 이내 적 비로얄 하나의 허기를 한 단계 악화합니다.",
    initial_double_step: true,
    ..PieceDefinition::new(INVITING_FOLLOWER, "inviting_follower", "초대의 신도", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1);\npiece-on(corpse,0,1) take(0,1)"
        .into()
}
