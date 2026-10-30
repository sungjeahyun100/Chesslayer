use super::PieceDefinition;
use crate::PieceType;

pub const VANGUARD_COMMANDER: PieceType = 42;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 80,
    description: "로얄. 모든 방향 한 칸 이동한 뒤 같은 방향 또는 인접한 방향으로 한 칸 더 이동·포획할 수 있습니다.",
    upgrades: &[
        super::UpgradeDefinition { target: super::MOUNTED_COMMANDER, key: "mounted_commander", effect: "말을 탄 선봉대장(으)로 영구 강화합니다.", requirement: "살아 있는 기물 · 쉼터 행동 1회" }
    ],
    ..PieceDefinition::new(VANGUARD_COMMANDER, "vanguard_commander", "선봉대장", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0) {take-move(1,0)} {take-move(1,1)} {take-move(1,-1)};\ntake-move(-1,0) {take-move(-1,0)} {take-move(-1,1)} {take-move(-1,-1)};\ntake-move(0,-1) {take-move(0,-1)} {take-move(1,-1)} {take-move(-1,-1)};\ntake-move(0,1) {take-move(0,1)} {take-move(1,1)} {take-move(-1,1)};\ntake-move(1,1) {take-move(1,1)} {take-move(1,0)} {take-move(0,1)};\ntake-move(1,-1) {take-move(1,-1)} {take-move(1,0)} {take-move(0,-1)};\ntake-move(-1,1) {take-move(-1,1)} {take-move(-1,0)} {take-move(0,1)};\ntake-move(-1,-1) {take-move(-1,-1)} {take-move(-1,0)} {take-move(0,-1)}".into()
}
