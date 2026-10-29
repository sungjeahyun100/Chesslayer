//! Stable relic IDs and Chesslayer opening presets, not canonical chess positions.
use crate::{PieceId, Position};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Relic {
    Agility,
    Reinforcements,
    CarnivalFlare,
    SilverCutlery,
    EmptyGoblet,
    TornMenu,
    LastPlate,
    GoldenTray,
    LaughingMask,
    EternalBell,
    SupplyMarker,
    SpecialInsignia,
    Haste,
    Whirlwind,
    SelfDefense,
    KingsIndian,
    QueensIndian,
    Hedgehog,
    Initiative,
    IceCream,
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
            Self::Agility => "민첩성",
            Self::Reinforcements => "충원",
            Self::CarnivalFlare => "카니발의 신호탄",
            Self::SilverCutlery => "은제 식기",
            Self::EmptyGoblet => "빈 술잔",
            Self::TornMenu => "찢어진 메뉴판",
            Self::LastPlate => "마지막 접시",
            Self::GoldenTray => "금박 쟁반",
            Self::LaughingMask => "웃는 가면",
            Self::EternalBell => "영원한 향연의 종",
            Self::SupplyMarker => "보급 표식",
            Self::SpecialInsignia => "특수부대 표식",
            Self::Haste => "가속",
            Self::Whirlwind => "소용돌이",
            Self::SelfDefense => "호신용품",
            Self::KingsIndian => "인디언 킹의 전략책",
            Self::QueensIndian => "인디언 퀸의 전략책",
            Self::Hedgehog => "고슴도치의 지혜",
            Self::Initiative => "선공권",
            Self::IceCream => "아이스크림",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Agility => "아군 부대가 턴마다 일반 행동을 2회 합니다.",
            Self::Reinforcements => "전투 승리 후 사망한 기물 하나를 초기 종류로 부활시킵니다.",
            Self::CarnivalFlare => "전투 시작 시 살아 있는 양 진영에 식재료, 아군에 황충의 축복을 부여합니다.",
            Self::SilverCutlery => "전투 첫 아군 시체 포획 시 카드 1장 드로우.",
            Self::EmptyGoblet => "턴마다 첫 아군 시체 포획 시 마나 1 회복.",
            Self::TornMenu => "전투 시작 시 무작위 비로얄 적 하나에 공복.",
            Self::LastPlate => "전투 종료 시 선택한 아군 하나의 허기 악화를 방지.",
            Self::GoldenTray => "시체 포획 시 인접한 다른 아군 하나에 민첩 1. 턴당 1회.",
            Self::LaughingMask => "아군 시체 포획 시 선택한 적 하나의 허기 악화. 전투당 1회.",
            Self::EternalBell => "전투 세 번째 아군 시체 포획 시 드로우 2, 마나 1.",
            Self::SupplyMarker => "포병대대의 임시 시작 표식. 추가 효과 없음.",
            Self::SpecialInsignia => "특수부대의 임시 시작 표식. 추가 효과 없음.",
            Self::Haste => "양 진영이 턴마다 최대 2회 이동합니다.",
            Self::Whirlwind => "전투 시작 시 적들끼리 시작 위치를 무작위로 교환합니다.",
            Self::SelfDefense => "전투 시작 시 무작위 아군 하나를 보호합니다. 적에게 직접 포획되어 사망하면 포획한 적도 사망합니다.",
            Self::KingsIndian => "다음 전투부터 아군을 킹사이드 피앙케토의 KIA 프리셋으로 배치합니다. 가장 최근에 얻은 시작 진형만 적용됩니다.",
            Self::QueensIndian => "다음 전투부터 아군을 백 진영으로 변환한 b파일 피앙케토의 QID 프리셋으로 배치합니다. 가장 최근에 얻은 시작 진형만 적용됩니다.",
            Self::Hedgehog => "다음 전투부터 아군을 백 진영의 헤지호그 프리셋으로 배치합니다. 가장 최근에 얻은 시작 진형만 적용됩니다.",
            Self::Initiative => "전투마다 적 턴 10회 동안 적의 일반 포획을 막습니다. 아군의 일반 포획이 성공하면 즉시 해제됩니다. 가호로 취소된 포획과 카드 제거는 해제하지 않습니다.",
            Self::IceCream => "플레이어 턴 시작마다 남은 코스트에 3을 더합니다. 전투 종료 시 누적 코스트가 초기화됩니다.",
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
