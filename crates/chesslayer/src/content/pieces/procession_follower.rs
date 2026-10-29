use super::PieceDefinition;
use crate::PieceType;

pub const PROCESSION_FOLLOWER: PieceType = 63;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "앞으로 한 칸 이동하고 전방 대각선으로 포획합니다. 처음에는 두 칸 전진할 수 있습니다. 마지막 랭크에서 승격하지 않습니다.",
    upgrades: &[
        super::UpgradeDefinition { target: super::HUNGRY_FOLLOWER, key: "hungry_follower", effect: "굶주린 신도(으)로 영구 강화합니다.", requirement: "살아 있는 기물 · 쉼터 행동 1회" }
    ],
    initial_double_step: true,
    ..PieceDefinition::new(PROCESSION_FOLLOWER, "procession_follower", "행렬의 신도", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1)".into()
}
