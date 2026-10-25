use super::PieceDefinition;
use crate::PieceType;

pub const PAWN: PieceType = 1;
pub(super) const PAWN_MOVEMENT: &str = "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1)";

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "앞으로 한 칸 이동하고 앞 대각선 한 칸의 적을 포획합니다. 이동하지 않은 폰은 초기 랭크에서 두 칸 전진할 수 있습니다.",
    initial_double_step: true,
    promotions: &[super::QUEEN, super::ROOK, super::BISHOP, super::KNIGHT],
    upgrades: &[
        super::UpgradeDefinition {
            target: super::TEMPEST_PAWN,
            key: "tempest",
            effect: "앞·좌우 빈칸 이동, 앞 대각선 및 전방 두 칸 포획. 초기 두 칸 전진은 없어집니다. 최종 랭크에서 템페스트 4종으로 승격합니다.",
            requirement: "살아 있는 일반 폰 · 쉼터 행동 1회 · 추가 비용 없음",
        },
        super::UpgradeDefinition {
            target: super::BOUNCING_PAWN,
            key: "bouncing",
            effect: "일반 폰 행마와 이동 이력을 유지하며 양측 바운싱 기물의 반사벽이 됩니다. 최종 랭크에서 바운싱 3종으로 승격합니다.",
            requirement: "살아 있는 일반 폰 · 쉼터 행동 1회 · 추가 비용 없음",
        },
    ],
    ..PieceDefinition::new(PAWN, "pawn", "폰", movement)
};

fn movement() -> String {
    PAWN_MOVEMENT.into()
}
