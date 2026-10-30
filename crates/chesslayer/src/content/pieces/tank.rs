use super::PieceDefinition;
use crate::PieceType;

pub const TANK: PieceType = 73;
pub const HE_TANK: PieceType = 87;
pub const AP_TANK: PieceType = 88;
pub const DOUBLE_BARREL: PieceType = 89;
pub const FRAGMENT_TANK: PieceType = 90;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 80,
    description: "킹 행마와 직선 두 칸 이동·포획. 직선으로 일곱 칸 이내 명중률 90% 사격, 탄약 2 소모. 포병 승패는 전차가 아닌 창고로 결정합니다.",
    upgrades: &[
        super::UpgradeDefinition { target: HE_TANK, key: "he_tank", effect: HE_DEFINITION.description, requirement: "살아 있는 전차 · 쉼터 행동 1회" },
        super::UpgradeDefinition { target: AP_TANK, key: "ap_tank", effect: AP_DEFINITION.description, requirement: "살아 있는 전차 · 쉼터 행동 1회" },
    ],
    ..PieceDefinition::new(TANK, "tank", "전차", movement)
};
pub const HE_DEFINITION: PieceDefinition = PieceDefinition {
    value: 95,
    description: "기존 전차 행마·사격. 명중 시 탄착 칸과 직교 인접 4칸의 모든 기물을 제거합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: DOUBLE_BARREL,
        key: "double_barrel",
        effect: DOUBLE_BARREL_DEFINITION.description,
        requirement: "살아 있는 고폭탄 전차 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(HE_TANK, "he_tank", "고폭탄 전차", movement)
};
pub const AP_DEFINITION: PieceDefinition = PieceDefinition {
    value: 95,
    description:
        "기존 전차 행마·사격. 최초 명중 후 직진 관통경로에 80·60·40·20% 순으로 추가 관통합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: FRAGMENT_TANK,
        key: "fragment_tank",
        effect: FRAGMENT_DEFINITION.description,
        requirement: "살아 있는 철갑탄 전차 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(
        AP_TANK,
        "ap_tank",
        "열화우라늄날개안정분리철갑탄 전차",
        movement,
    )
};
pub const DOUBLE_BARREL_DEFINITION: PieceDefinition = PieceDefinition {
    value: 110,
    description: "기존 전차 행마. 템페스트 룩의 행마 범위로 사격하고 명중 시 탄착 칸과 직교 인접 4칸을 폭발시킵니다.",
    ..PieceDefinition::new(DOUBLE_BARREL, "double_barrel", "쌍열포", movement)
};
pub const FRAGMENT_DEFINITION: PieceDefinition = PieceDefinition {
    value: 110,
    description: "기존 전차 행마·사격. 최초 명중 지점에서 직진·좌우 45도 세 갈래 관통경로가 각각 75·50·25%로 진행합니다.",
    ..PieceDefinition::new(FRAGMENT_TANK, "fragment_tank", "파편 첨가물", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
pub(super) fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1);\ntake-move(1,0) take-move(1,0);\ntake-move(-1,0) take-move(-1,0);\ntake-move(0,-1) take-move(0,-1);\ntake-move(0,1) take-move(0,1)".into()
}
