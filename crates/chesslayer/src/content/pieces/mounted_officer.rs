//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const MOUNTED_OFFICER: PieceType = 45;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "기획서의 독립 체섬블리 행마. ",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(
        MOUNTED_OFFICER,
        "mounted_officer",
        "말을 탄 선봉 부대장",
        movement,
    )
};
fn movement() -> String {
    r#"take-move(1, 0);
take-move(-1, 0);
take-move(0, -1);
take-move(0, 1);
take-move(1, 1);
take-move(1, -1);
take-move(-1, 1);
take-move(-1, -1);
take-move(1, 2);
take-move(2, 1);
take-move(2, -1);
take-move(1, -2);
take-move(-1, -2);
take-move(-2, -1);
take-move(-2, 1);
take-move(-1, 2);"#
        .into()
}
