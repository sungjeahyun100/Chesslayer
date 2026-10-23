use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_TEMPEST_BISHOP: PieceType = 26;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Bishop,
    value: 85,
    description: "직교 경유 칸에서 양쪽 대각선으로 갈라집니다. 각 실제 경로는 경계나 바운싱 폰 직전에 최대 한 번 반사하며 포획하면 멈춥니다.",
    ..PieceDefinition::new(BOUNCING_TEMPEST_BISHOP, "bouncing_tempest_bishop", "바운싱 템페스트 비숍", movement)
};

pub(super) fn movement() -> String {
    super::movement::bouncing_tempest(
        super::movement::ORTHOGONAL,
        super::movement::tempest_bishop_branches,
    )
}
