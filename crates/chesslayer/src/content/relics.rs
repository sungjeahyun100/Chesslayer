//! Stable relic IDs and Chesslayer opening presets, not canonical chess positions.
use crate::{PieceId, Position};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Relic {
    Haste,
    Whirlwind,
    SelfDefense,
    KingsIndian,
    QueensIndian,
    Hedgehog,
    Initiative,
    IceCream,
    Reinforcement,
    CarnivalFlare,
    SilverCutlery,
    EmptyGoblet,
    TornMenu,
    LastPlate,
    GoldenTray,
    SmilingMask,
    EternalFeastBell,
    FireControl,
    BallisticComputer,
    ReconScope,
    Silencer,
}

pub const RELIC_REWARD_POOL: &[Relic] = &[
    Relic::Haste,
    Relic::Whirlwind,
    Relic::SelfDefense,
    Relic::KingsIndian,
    Relic::QueensIndian,
    Relic::Hedgehog,
    Relic::Initiative,
    Relic::IceCream,
];
pub const INITIATIVE_TURNS: u8 = 10;

// Entries correspond to permanent standard-roster IDs 1..=16, never current kinds.
// Coordinates: a8=(0,0), h1=(7,7). See docs/RELICS.md for the full square table.
pub const KIA: [(i32, i32); 16] = [
    (0, 7),
    (3, 6),
    (2, 7),
    (3, 7),
    (6, 7),
    (6, 6),
    (5, 5),
    (5, 7),
    (0, 6),
    (1, 6),
    (2, 6),
    (3, 5),
    (4, 6),
    (5, 6),
    (6, 5),
    (7, 6),
];
pub const QID: [(i32, i32); 16] = [
    (0, 7),
    (3, 6),
    (1, 6),
    (3, 7),
    (6, 7),
    (4, 6),
    (5, 5),
    (5, 7),
    (0, 6),
    (1, 5),
    (2, 6),
    (3, 4),
    (4, 5),
    (5, 6),
    (6, 6),
    (7, 6),
];
pub const HEDGEHOG: [(i32, i32); 16] = [
    (2, 7),
    (3, 6),
    (1, 6),
    (2, 6),
    (6, 7),
    (4, 6),
    (5, 5),
    (4, 7),
    (0, 5),
    (1, 5),
    (2, 5),
    (3, 5),
    (4, 5),
    (5, 6),
    (6, 6),
    (7, 6),
];

impl Relic {
    pub fn name(self) -> &'static str {
        match self {
            Self::Haste => "민첩성",
            Self::Whirlwind => "소용돌이",
            Self::SelfDefense => "호신용품",
            Self::KingsIndian => "인디언 킹의 전략책",
            Self::QueensIndian => "인디언 퀸의 전략책",
            Self::Hedgehog => "고슴도치의 지혜",
            Self::Initiative => "선공권",
            Self::IceCream => "아이스크림",
            Self::Reinforcement => "충원",
            Self::CarnivalFlare => "카니발의 신호탄",
            Self::SilverCutlery => "은제 식기",
            Self::EmptyGoblet => "빈 술잔",
            Self::TornMenu => "찢어진 메뉴판",
            Self::LastPlate => "마지막 접시",
            Self::GoldenTray => "금박 쟁반",
            Self::SmilingMask => "웃는 가면",
            Self::EternalFeastBell => "영원한 향연의 종",
            Self::FireControl => "사격 통제장치",
            Self::BallisticComputer => "탄도 계산기",
            Self::ReconScope => "정찰 망원경",
            Self::Silencer => "소음기",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Haste => "아군 부대가 한 턴에 최대 두 번 움직일 수 있습니다.",
            Self::Whirlwind => "전투 시작 시 적들끼리 시작 위치를 무작위로 교환합니다.",
            Self::SelfDefense => "전투 시작 시 무작위 아군 하나를 보호합니다. 적에게 직접 포획되어 사망하면 포획한 적도 사망합니다.",
            Self::KingsIndian => "다음 전투부터 아군을 킹사이드 피앙케토의 KIA 프리셋으로 배치합니다. 가장 최근에 얻은 시작 진형만 적용됩니다.",
            Self::QueensIndian => "다음 전투부터 아군을 백 진영으로 변환한 b파일 피앙케토의 QID 프리셋으로 배치합니다. 가장 최근에 얻은 시작 진형만 적용됩니다.",
            Self::Hedgehog => "다음 전투부터 아군을 백 진영의 헤지호그 프리셋으로 배치합니다. 가장 최근에 얻은 시작 진형만 적용됩니다.",
            Self::Initiative => "전투마다 적 턴 10회 동안 적의 일반 포획을 막습니다. 아군의 일반 포획이 성공하면 즉시 해제됩니다. 가호로 취소된 포획과 카드 제거는 해제하지 않습니다.",
            Self::IceCream => "플레이어 턴 시작마다 남은 코스트에 3을 더합니다. 전투 종료 시 누적 코스트가 초기화됩니다.",
            Self::Reinforcement => "전투 종료 후 죽은 기물 중 하나를 선택해 시작 기물과 초기 상태로 부활시킵니다.",
            Self::CarnivalFlare => "매 전투 시작 시 양 진영의 살아 있는 기물에 식재료를, 아군에게 황충의 축복을 부여합니다.",
            Self::SilverCutlery => "매 전투 아군의 첫 시체 포획 시 카드 한 장을 뽑습니다.",
            Self::EmptyGoblet => "턴마다 아군의 첫 시체 포획 시 마나 1을 회복합니다.",
            Self::TornMenu => "전투 시작 시 무작위 적 비로얄 하나에 공복을 부여합니다.",
            Self::LastPlate => "전투 종료 시 허기가 악화될 아군 하나를 선택해 이번 악화를 막습니다.",
            Self::GoldenTray => "턴당 한 번, 아군이 시체를 포획하면 인접한 다른 아군 하나에 민첩 1을 부여합니다.",
            Self::SmilingMask => "전투당 한 번, 아군의 시체 포획 시 적 비로얄 하나의 허기를 한 단계 악화합니다.",
            Self::EternalFeastBell => "전투당 한 번, 아군의 세 번째 시체 포획 시 카드 두 장을 뽑고 마나 1을 회복합니다.",
            Self::FireControl => "아군의 사격 명중률이 10%p 증가합니다. 탄약은 창고에 저장되며 턴마다 회복되지 않습니다.",
            Self::BallisticComputer => "아군 사격 명중률이 추가로 5%p 증가합니다.",
            Self::ReconScope => "아군 사격 명중률이 15%p 증가합니다.",
            Self::Silencer => "전투당 첫 사격 명중 시 사수에게 민첩 1을 부여합니다.",

        }
    }
    pub fn formation(self) -> Option<&'static [(i32, i32); 16]> {
        match self {
            Self::KingsIndian => Some(&KIA),
            Self::QueensIndian => Some(&QID),
            Self::Hedgehog => Some(&HEDGEHOG),
            _ => None,
        }
    }
    pub fn position(self, id: PieceId) -> Option<Position> {
        let &(file, rank) = self.formation()?.get(id.checked_sub(1)? as usize)?;
        Some(Position::new(file, rank))
    }
}

/// Rewards are private to the selected troop in v0.3.0.
pub fn reward_pool(character: &str) -> &'static [Relic] {
    match character {
        "vanguard" => &[Relic::Reinforcement],
        "heretic" => &[
            Relic::SilverCutlery,
            Relic::EmptyGoblet,
            Relic::TornMenu,
            Relic::LastPlate,
            Relic::GoldenTray,
            Relic::SmilingMask,
            Relic::EternalFeastBell,
        ],
        "artillery" => &[Relic::BallisticComputer],
        "special" => &[Relic::Silencer],
        _ => RELIC_REWARD_POOL,
    }
}
