//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const VANGUARD_COMMANDER: PieceType = 46;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "기획서의 독립 체섬블리 행마. ",
    upgrades: &[super::UpgradeDefinition {
        target: super::MOUNTED_COMMANDER,
        key: "mounted_commander",
        effect: "기획서의 독립 체섬블리 행마. ",
        requirement: "생존 기물 · 강화 기회 1회",
    }],
    initial_double_step: false,
    ..PieceDefinition::new(
        VANGUARD_COMMANDER,
        "vanguard_commander",
        "선봉대장",
        movement,
    )
};
fn movement() -> String {
    r#"take-move(1, 0)
{take-move(1, 0)}
{take-move(1, 1)}
{take-move(1, -1)};

take-move(-1, 0)
{take-move(-1, 0)}
{take-move(-1, -1)}
{take-move(-1, 1)};

take-move(0, -1)
{take-move(0, -1)}
{take-move(1, -1)}
{take-move(-1, -1)};

take-move(0, 1)
{take-move(0, 1)}
{take-move(1, 1)}
{take-move(-1, 1)};

take-move(1, 1)
{take-move(1, 1)}
{take-move(1, 0)}
{take-move(0, 1)};

take-move(1, -1)
{take-move(1, -1)}
{take-move(1, 0)}
{take-move(0, -1)};

take-move(-1, 1)
{take-move(-1, 1)}
{take-move(-1, 0)}
{take-move(0, 1)};

take-move(-1, -1)
{take-move(-1, -1)}
{take-move(-1, 0)}
{take-move(0, -1)};"#
        .into()
}
