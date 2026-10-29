use super::*;
use crate::{content::pieces::*, ENEMY, PLAYER};
pub fn ally(b: &Battle) -> Vec<PieceId> {
    b.living(PLAYER)
        .into_iter()
        .filter(|id| !b.status(*id).disabled)
        .collect()
}
pub fn enemy(b: &Battle) -> Vec<PieceId> {
    b.living(ENEMY)
}
pub fn allied_nonroyals(b: &Battle) -> Vec<PieceId> {
    ally(b).into_iter().filter(|id| !b.is_royal(*id)).collect()
}
pub fn nonroyals(b: &Battle) -> Vec<PieceId> {
    ally(b)
        .into_iter()
        .chain(enemy(b))
        .filter(|id| !b.is_royal(*id))
        .collect()
}
pub fn corpse(b: &Battle) -> Vec<PieceId> {
    b.corpses()
}
pub fn ally_and_corpse(b: &Battle) -> Vec<PieceId> {
    ally(b).into_iter().chain(corpse(b)).collect()
}
pub fn followers(b: &Battle) -> Vec<PieceId> {
    allied_nonroyals(b)
        .into_iter()
        .filter(|id| b.located(*id).is_some_and(|(_, p)| is_follower(p.kind)))
        .collect()
}
pub fn troubled(b: &Battle) -> Vec<PieceId> {
    enemy(b)
        .into_iter()
        .filter(|id| {
            let s = b.status(*id);
            s.madness > 0 || s.guilt > 0
        })
        .collect()
}
fn none(_: &Battle) -> Vec<PieceId> {
    vec![]
}
pub fn definition(kind: CardKind) -> CardDefinition {
    let (name, cost, description, targets, mode): (&str, u32, &str, CardTargets, SelectionMode) =
        match kind {
            CardKind::Stimpack => (
                "스팀팩",
                1,
                "아군 하나에 민첩 4, 약화 1. 다음 아군 턴에 영구 1단계 하락.",
                ally,
                SelectionMode::Single,
            ),
            CardKind::MassCrash => (
                "단체 몸통박치기",
                0,
                "희생할 아군 비로얄과 같은 수의 적 비로얄을 선택해 함께 제거.",
                nonroyals,
                SelectionMode::Multiple,
            ),
            CardKind::InfantrySupport => (
                "보병지원",
                2,
                "아군 진영(1~2랭크)의 빈 칸에 시작 도끼 보병을 생성.",
                none,
                SelectionMode::Square,
            ),
            CardKind::Sacrifice => (
                "희생",
                0,
                "아군 비로얄 하나 희생, 마나 3 획득.",
                allied_nonroyals,
                SelectionMode::Single,
            ),
            CardKind::LastStand => (
                "마지막 저항",
                3,
                "아군 수 절반 이하: 전원 민첩 3, 영구 강화 1단계. 이번 턴 적을 죽이지 못하면 패배.",
                none,
                SelectionMode::None,
            ),
            CardKind::ExcitedSteps => (
                "들뜬 발걸음",
                1,
                "아군 하나에 이번 턴 민첩 1.",
                ally,
                SelectionMode::Single,
            ),
            CardKind::BloodyInvitation => (
                "피 묻은 초대장",
                1,
                "적 하나의 허기를 한 단계 악화.",
                enemy,
                SelectionMode::Single,
            ),
            CardKind::TableManners => (
                "식사의 예법",
                1,
                "아군의 다음 행동에 인접 시체를 행마와 무관하게 포획 가능.",
                ally,
                SelectionMode::Single,
            ),
            CardKind::Masquerade => (
                "가면무도회",
                2,
                "아군 비로얄 둘의 위치 교환. 사용 시 소멸.",
                allied_nonroyals,
                SelectionMode::Multiple,
            ),
            CardKind::PullTablecloth => (
                "식탁보 걷기",
                0,
                "시체 하나를 상하좌우 빈 칸으로 이동. 턴당 1회.",
                corpse,
                SelectionMode::Single,
            ),
            CardKind::InterludeWaltz => (
                "막간의 왈츠",
                1,
                "인접한 아군 비로얄 둘의 위치 교환.",
                allied_nonroyals,
                SelectionMode::Multiple,
            ),
            CardKind::LateDinner => (
                "뒤늦은 저녁",
                2,
                "아군 하나의 허기를 한 단계 개선.",
                ally,
                SelectionMode::Single,
            ),
            CardKind::MealTime => (
                "식사 시간입니다!",
                1,
                "적 하나 허기 악화. 이미 굶주림·기아였다면 드로우 1.",
                enemy,
                SelectionMode::Single,
            ),
            CardKind::DeliveredFeast => (
                "배달된 만찬",
                2,
                "아군 하나와 2칸 이내 시체 선택. 제자리에서 시체 포획 효과.",
                ally_and_corpse,
                SelectionMode::Multiple,
            ),
            CardKind::BackstageSacrifice => (
                "무대 뒤의 희생",
                1,
                "아군 비로얄 신도를 시체로 바꾸고 드로우 2. 전투당 1회.",
                followers,
                SelectionMode::Single,
            ),
            CardKind::TornMask => (
                "찢어진 가면",
                2,
                "적 하나의 황충의 축복을 이번 전투 동안 무효화.",
                enemy,
                SelectionMode::Single,
            ),
            CardKind::CrackedConscience => (
                "양심의 균열",
                2,
                "광기 또는 죄책감이 있는 적의 해당 스택 +1. 둘 다 있으면 광기 우선.",
                troubled,
                SelectionMode::Single,
            ),
            CardKind::LastSupper => (
                "최후의 만찬",
                3,
                "시체 1~3개와 같은 수의 아군 선택. 시체를 소비하고 각 아군 민첩 1.",
                ally_and_corpse,
                SelectionMode::Multiple,
            ),
            CardKind::EndlessJoy => (
                "환희는 끝나지 않는다",
                3,
                "이번 턴 아군 비로얄이 시체 포획 시 추가 행동 1회. 전투당 1회.",
                none,
                SelectionMode::None,
            ),
            CardKind::Volley => (
                "전원 일제사격",
                0,
                "아군의 포획 범위 안 모든 비로얄 적 사망. 사망 수만큼 탄약 소비.",
                none,
                SelectionMode::None,
            ),
            CardKind::FormLine => (
                "전선잡기",
                0,
                "아군을 선택하고 빈 칸으로 재배치. 원하는 만큼 배치 후 완료. 탄약 0.",
                ally,
                SelectionMode::Placement,
            ),
            CardKind::SpecialManeuver => (
                "기동",
                1,
                "임시 카드: 아군 하나에 이번 턴 민첩 1.",
                ally,
                SelectionMode::Single,
            ),
            _ => unreachable!("tactical card registry"),
        };
    let mut d = CardDefinition::new(kind, name, cost, description, |_, _| {
        Err("대상과 위치를 선택하세요.".into())
    });
    d.targeted = matches!(
        mode,
        SelectionMode::Single | SelectionMode::Multiple | SelectionMode::Placement
    );
    d.target_candidates = targets;
    d.selection_mode = mode;
    d.exhaust = kind == CardKind::Masquerade;
    d
}
