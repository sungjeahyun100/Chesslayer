use super::PieceDefinition;
use crate::PieceType;

pub const DRONE: PieceType = 72;
pub const C4_DRONE: PieceType = 83;
pub const MACHINE_GUN_DRONE: PieceType = 84;
pub const TACTICAL_NUKE_DRONE: PieceType = 85;
pub const GRENADE_LAUNCHER_DRONE: PieceType = 86;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 45,
    description: "나이트 도약과 상하좌우 한 칸 이동·포획. 십자포화 자폭으로 자신과 직교 인접 칸의 모든 기물을 제거합니다.",
    upgrades: &[
        super::UpgradeDefinition { target: C4_DRONE, key: "c4_drone", effect: C4_DEFINITION.description, requirement: "살아 있는 수류탄 드론 · 쉼터 행동 1회" },
        super::UpgradeDefinition { target: MACHINE_GUN_DRONE, key: "machine_gun_drone", effect: MACHINE_GUN_DEFINITION.description, requirement: "살아 있는 수류탄 드론 · 쉼터 행동 1회" },
    ],
    ..PieceDefinition::new(DRONE, "drone", "수류탄 드론", movement)
};
pub const C4_DEFINITION: PieceDefinition = PieceDefinition {
    value: 55,
    description: "기존 드론 행마. 광역 자폭으로 자신과 주변 8칸의 모든 기물을 제거합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: TACTICAL_NUKE_DRONE,
        key: "tactical_nuke_drone",
        effect: TACTICAL_NUKE_DEFINITION.description,
        requirement: "살아 있는 C4 드론 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(C4_DRONE, "c4_drone", "C4 드론", movement)
};
pub const MACHINE_GUN_DEFINITION: PieceDefinition = PieceDefinition {
    value: 60,
    description: "기존 드론 행마. 직교 4칸 이내 명중률 90% 기관총 사격(탄약 1) 또는 십자포화 자폭을 선택합니다.",
    upgrades: &[super::UpgradeDefinition { target: GRENADE_LAUNCHER_DRONE, key: "grenade_launcher_drone", effect: GRENADE_LAUNCHER_DEFINITION.description, requirement: "살아 있는 기관총 드론 · 쉼터 행동 1회" }],
    ..PieceDefinition::new(MACHINE_GUN_DRONE, "machine_gun_drone", "기관총 드론", movement)
};
pub const TACTICAL_NUKE_DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description: "기존 드론 행마. 자신과 킹·나이트 범위의 모든 기물을 제거하고 해당 칸에 2라운드 방사성 낙진을 남깁니다.",
    ..PieceDefinition::new(TACTICAL_NUKE_DRONE, "tactical_nuke_drone", "전술핵 드론", movement)
};
pub const GRENADE_LAUNCHER_DEFINITION: PieceDefinition = PieceDefinition {
    value: 75,
    description: "기존 드론 행마. 직교 4칸 이내 유탄 사격은 명중 지점과 직교 인접 칸을 폭발시킵니다. 십자포화 자폭도 사용할 수 있습니다.",
    ..PieceDefinition::new(GRENADE_LAUNCHER_DRONE, "grenade_launcher_drone", "유탄발사기 드론", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
pub(super) fn movement() -> String {
    "take-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2);\ntake-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1)".into()
}
