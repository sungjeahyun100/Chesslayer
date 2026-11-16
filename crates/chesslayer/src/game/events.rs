//! Seeded, eligibility-filtered expedition events. Choice execution is transactional.
use super::*;
use crate::content::cards::avenger::Rarity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Ambush,
    Roulette,
    LuckyDay,
    UnluckyDay,
    Hidden,
    Supplies,
    Disposal,
    RandomBurn,
    DoubleBurn,
    Restaurant,
}
#[derive(Clone, Serialize)]
pub struct EventChoiceView {
    pub id: String,
    pub label: String,
    pub disabled: bool,
    pub card: Option<CardDefinition>,
}
#[derive(Clone, Serialize)]
pub struct EventView {
    pub kind: EventKind,
    pub title: String,
    pub description: String,
    pub grade: &'static str,
    pub choices: Vec<EventChoiceView>,
    pub result: Option<String>,
}
#[derive(Clone, Default)]
pub(super) struct EventState {
    pub active: Option<EventView>,
    pub last: Option<EventKind>,
    pub ambushed: bool,
    pub ambush_combat: bool,
    pub next_battle_penalty: bool,
    pub hidden_outcome: usize,
    pub selected_burns: Vec<u32>,
}
fn choice(id: impl ToString, label: impl ToString, enabled: bool) -> EventChoiceView {
    EventChoiceView {
        id: id.to_string(),
        label: label.to_string(),
        disabled: !enabled,
        card: None,
    }
}
impl Game {
    fn event_nonroyals(&self) -> Vec<PieceId> {
        self.battle
            .outcome()
            .iter()
            .filter(|p| p.position.is_some() && !self.battle.is_royal_piece(p.id))
            .map(|p| p.id)
            .collect()
    }
    fn threshold_candidates(&self) -> Vec<PieceId> {
        self.battle
            .outcome()
            .iter()
            .filter(|p| {
                p.position.is_some()
                    && !self.battle.is_royal_piece(p.id)
                    && p.persistent.get("death_threshold").copied().unwrap_or(0) == 0
            })
            .map(|p| p.id)
            .collect()
    }
    fn event_upgrades(&self) -> Vec<(PieceId, Vec<PieceType>)> {
        self.battle
            .outcome()
            .iter()
            .filter(|p| p.position.is_some())
            .filter_map(|p| {
                let roster = self.battle.outcome();
                let options: Vec<_> = content::pieces::definition(p.kind)
                    .unwrap()
                    .upgrades
                    .iter()
                    .filter(|u| {
                        content::pieces::required_owned(u.target).iter().all(|k| {
                            roster
                                .iter()
                                .any(|p| content::pieces::owned_matches_kind(p, *k))
                        })
                    })
                    .map(|u| u.target)
                    .collect();
                (!options.is_empty()).then_some((p.id, options))
            })
            .collect()
    }
    fn event_relics(&self, min: Rarity) -> Vec<Relic> {
        content::relics::reward_pool(&self.squad_id)
            .iter()
            .copied()
            .filter(|r| !self.relics.contains(r) && r.rarity() >= min)
            .collect()
    }
    pub(super) fn enter_random_event(&mut self) {
        use EventKind::*;
        let removable = !self.event_nonroyals().is_empty();
        let upgradeable = !self.event_upgrades().is_empty();
        let mut pool = vec![UnluckyDay, Supplies, LuckyDay, Hidden, Restaurant];
        if removable && upgradeable {
            pool.push(Roulette);
        }
        if !self.deck.is_empty() {
            pool.push(Disposal);
            if !self.legacy_content {
                pool.extend([RandomBurn, DoubleBurn]);
            }
        }
        if !self.event_state.ambushed
            && self
                .card_pool()
                .iter()
                .any(|k| k.definition().rarity >= Rarity::Epic)
        {
            pool.push(Ambush);
        }
        pool.retain(|k| {
            Some(*k) != self.event_state.last
                && (*k != UnluckyDay
                    || !self.threshold_candidates().is_empty()
                    || self.gold > 0
                    || !self.event_state.next_battle_penalty)
        });
        let kind = pool[self.rng.index(pool.len())];
        self.event_state.selected_burns.clear();
        self.event_state.last = Some(kind);
        self.phase = RunPhase::Event;
        let (title, description, grade, choices) = match kind {
            Ambush => {
                self.event_state.ambushed = true;
                ("엠부쉬","1층 보스 선봉대가 길을 막았습니다. 승리하면 영웅 이상 카드 보상과 45골드를 얻습니다.","danger",vec![choice("fight","기습에 맞선다",true)])
            }
            Roulette => (
                "룰렛",
                "50%: 무작위 비로얄 영구 제거. 50%: 무작위 아군 영구 강화.",
                "common",
                vec![choice("spin", "룰렛을 돌린다", true)],
            ),
            LuckyDay => {
                let pool = self.card_pool();
                let kinds: Vec<_> = if self.legacy_content {
                    (0..15).map(|_| pool[self.rng.index(pool.len())]).collect()
                } else {
                    let mut kinds = pool.to_vec();
                    kinds.sort_unstable();
                    kinds.dedup();
                    self.rng.shuffle(&mut kinds);
                    kinds.into_iter().take(15).collect()
                };
                let choices = kinds
                    .into_iter()
                    .enumerate()
                    .map(|(i, kind)| {
                        let card = kind.definition();
                        EventChoiceView {
                            id: i.to_string(),
                            label: card.name.into(),
                            disabled: false,
                            card: Some(card),
                        }
                    })
                    .collect();
                (
                    "운수 좋은 날",
                    "희귀도 가중치 없이 추첨한 서로 다른 카드 중 한 장을 선택하세요.",
                    "rare",
                    choices,
                )
            }
            UnluckyDay => (
                "운수 나쁜 날",
                "다음 손실 중 하나를 선택해야 합니다.",
                "danger",
                vec![
                    choice("gold", "현재 골드의 50%를 잃는다", self.gold > 0),
                    choice(
                        "threshold",
                        "무작위 아군 비로얄에 죽음의 문턱 (다음 전투 즉사)",
                        !self.threshold_candidates().is_empty(),
                    ),
                    choice(
                        "penalty",
                        "다음 전투 초기 마나 −1, 첫 손패 −1",
                        !self.event_state.next_battle_penalty,
                    ),
                ],
            ),
            Hidden => {
                let mut outcomes = vec![2];
                if !self.event_relics(Rarity::Common).is_empty() {
                    outcomes.push(0);
                }
                if removable && !self.event_relics(Rarity::Epic).is_empty() {
                    outcomes.push(1);
                }
                if upgradeable {
                    outcomes.push(3);
                }
                self.event_state.hidden_outcome = outcomes[self.rng.index(outcomes.len())];
                let (title,desc,label) = match self.event_state.hidden_outcome {
                    0 => ("히든 · 숨겨진 창고","봉인된 창고에서 유물을 발견했습니다.","무작위 유물 획득"),
                    1 => ("히든 · 수상한 계약","위험: 무작위 아군 비로얄 하나를 영구 희생합니다. 영웅 이상 유물 하나를 얻습니다.","희생을 받아들인다"),
                    3 => ("히든 · 비밀 병기","무작위 아군 하나를 한 단계 영구 강화합니다.","강화한다"),
                    _ => ("히든 · 운명의 반전","위험: 골드·덱·부대 중 하나가 무작위로 바뀝니다. 골드 25 손실, 카드 영구 제거, 비로얄 영구 제거가 발생할 수 있습니다.","운명을 바꾼다"),
                };
                (
                    title,
                    desc,
                    "rare",
                    vec![
                        choice("accept", label, true),
                        choice("leave", "떠난다", true),
                    ],
                )
            }
            Supplies => (
                "보급품 상자",
                "골드·카드·미보유 유물 중 가능한 보상을 하나 얻습니다.",
                "common",
                vec![choice("open", "상자를 연다", true)],
            ),
            Disposal => (
                "폐기물 처리장",
                "덱에서 카드 한 장을 무료로 영구 소각합니다.",
                "common",
                self.deck
                    .iter()
                    .map(|c| EventChoiceView {
                        id: c.id.to_string(),
                        label: format!("{} 소각", c.kind.definition().name),
                        disabled: false,
                        card: Some(c.kind.definition()),
                    })
                    .collect(),
            ),
            RandomBurn => (
                "무작위 카드 소각",
                "현재 덱에서 카드 한 장을 무작위로 영구 소각합니다.",
                "common",
                vec![choice("continue", "원정 경로로 돌아가기", true)],
            ),
            DoubleBurn => (
                "선택 카드 2장 소각",
                "덱에서 서로 다른 카드 두 장을 골라 무료로 영구 소각합니다.",
                "common",
                self.deck
                    .iter()
                    .map(|c| EventChoiceView {
                        id: c.id.to_string(),
                        label: format!("{} #{} 선택", c.kind.definition().name, c.id),
                        disabled: false,
                        card: Some(c.kind.definition()),
                    })
                    .collect(),
            ),
            Restaurant => (
                "작은 식당",
                "따뜻한 식사로 아군 전체의 지속 허기를 제거합니다.",
                "common",
                vec![
                    choice("eat", "식사하기 · 무료", true),
                    choice("leave", "떠나기", true),
                ],
            ),
        };
        self.event_state.active = Some(EventView {
            kind,
            title: title.into(),
            description: description.into(),
            grade,
            choices,
            result: None,
        });
        if kind == RandomBurn {
            let view = self.event_state.active.as_mut().unwrap();
            if self.deck.is_empty() {
                view.result = Some("소각 가능한 카드가 없습니다.".into());
            } else {
                let id = self.deck[self.rng.index(self.deck.len())].id;
                let card = self.burn_card(id).expect("selected deck card exists");
                let name = card.kind.definition().name;
                let view = self.event_state.active.as_mut().unwrap();
                view.result = Some(format!("{} #{} 영구 소각 완료", name, id));
                view.choices.insert(
                    0,
                    EventChoiceView {
                        id: id.to_string(),
                        label: "소각된 카드".into(),
                        disabled: true,
                        card: Some(card.kind.definition()),
                    },
                );
            }
        }
    }
    fn event_remove_random(&mut self) -> Result<String, String> {
        let ids = self.event_nonroyals();
        if ids.is_empty() {
            return Err("희생할 비로얄이 없습니다.".into());
        }
        let id = ids[self.rng.index(ids.len())];
        let mut roster = self.battle.outcome();
        let name = piece_name(roster.iter().find(|p| p.id == id).unwrap().kind);
        roster.retain(|p| p.id != id);
        self.battle
            .replace_roster_at_rest(roster)
            .map_err(engine_error)?;
        Ok(format!("{name} 영구 제거"))
    }
    fn event_upgrade_random(&mut self) -> Result<String, String> {
        let options = self.event_upgrades();
        if options.is_empty() {
            return Err("강화 가능한 기물이 없습니다.".into());
        }
        let (id, paths) = &options[self.rng.index(options.len())];
        let target = paths[self.rng.index(paths.len())];
        self.upgrade_piece(*id, target, "이벤트")?;
        Ok(format!("{} 영구 강화", piece_name(target)))
    }
    fn event_grant_relic(&mut self, min: Rarity) -> Result<String, String> {
        let candidates = self.event_relics(min);
        if candidates.is_empty() {
            return Err("획득할 유물이 없습니다.".into());
        }
        let relic = candidates[self.rng.index(candidates.len())];
        self.grant_relic(relic);
        Ok(format!("{} 획득", relic.name()))
    }
    pub(super) fn choose_event(&mut self, id: &str) -> Result<(), String> {
        if self.phase != RunPhase::Event {
            return Err("이벤트 중이 아닙니다.".into());
        }
        let event = self
            .event_state
            .active
            .clone()
            .ok_or("이벤트가 없습니다.")?;
        if event.result.is_some() {
            if id != "continue" {
                return Err("결과를 확인하고 출발하세요.".into());
            }
            self.event_state.active = None;
            self.complete_node();
            return Ok(());
        }
        let selection = event
            .choices
            .iter()
            .find(|c| c.id == id && !c.disabled)
            .ok_or("사용할 수 없는 선택지입니다.")?;
        let mut next = self.clone();
        use EventKind::*;
        let result = match event.kind {
            Ambush => {
                next.event_state.ambush_combat = true;
                next.map.nodes[next.current_node.unwrap()].encounter =
                    Some(content::encounters::VANGUARD_BOSS);
                next.start_battle()?;
                next.event_state.active = None;
                *self = next;
                return Ok(());
            }
            Roulette => {
                if next.rng.index(2) == 0 {
                    next.event_remove_random()?
                } else {
                    next.event_upgrade_random()?
                }
            }
            LuckyDay => {
                let card = selection.card.as_ref().unwrap();
                next.add_card(card.kind);
                format!("{} 획득", card.name)
            }
            UnluckyDay => match id {
                "gold" => {
                    let n = next.gold.div_ceil(2);
                    next.gold -= n;
                    format!("{n}골드 손실")
                }
                "threshold" => {
                    let ids = next.threshold_candidates();
                    let chosen = ids[next.rng.index(ids.len())];
                    let mut roster = next.battle.outcome();
                    roster
                        .iter_mut()
                        .find(|p| p.id == chosen)
                        .unwrap()
                        .persistent
                        .insert("death_threshold".into(), 1);
                    next.battle
                        .replace_roster_at_rest(roster)
                        .map_err(engine_error)?;
                    "죽음의 문턱 부여: 다음 전투에서 즉사합니다.".into()
                }
                _ => {
                    next.event_state.next_battle_penalty = true;
                    "다음 전투 초기 마나와 첫 손패 각각 −1".into()
                }
            },
            Hidden if id == "leave" => "계약을 거절하고 떠났습니다.".into(),
            Hidden => match next.event_state.hidden_outcome {
                0 => next.event_grant_relic(Rarity::Common)?,
                1 => {
                    let loss = next.event_remove_random()?;
                    format!("{loss} · {}", next.event_grant_relic(Rarity::Epic)?)
                }
                3 => next.event_upgrade_random()?,
                _ => match next.rng.index(3) {
                    0 => {
                        if next.rng.index(2) == 0 {
                            next.gold = next.gold.saturating_sub(25);
                            "최대 25골드 손실".into()
                        } else {
                            next.gold = next.gold.saturating_add(50);
                            "50골드 획득".into()
                        }
                    }
                    1 => {
                        if !next.deck.is_empty() && next.rng.index(2) == 0 {
                            let i = next.rng.index(next.deck.len());
                            let card = next.deck.remove(i);
                            format!("{} 영구 소각", card.kind.definition().name)
                        } else {
                            let pool = next.card_pool();
                            let k = pool[next.rng.index(pool.len())];
                            next.add_card(k);
                            format!("{} 획득", k.definition().name)
                        }
                    }
                    _ => {
                        if !next.event_nonroyals().is_empty() && next.rng.index(2) == 0 {
                            next.event_remove_random()?
                        } else if !next.event_upgrades().is_empty() {
                            next.event_upgrade_random()?
                        } else {
                            next.gold += 25;
                            "25골드 획득".into()
                        }
                    }
                },
            },
            Supplies => {
                let n = next
                    .rng
                    .index(if next.event_relics(Rarity::Common).is_empty() {
                        2
                    } else {
                        3
                    });
                match n {
                    0 => {
                        next.gold += 30;
                        "30골드 획득".into()
                    }
                    1 => {
                        let pool = next.card_pool();
                        let k = pool[next.rng.index(pool.len())];
                        next.add_card(k);
                        format!("{} 획득", k.definition().name)
                    }
                    _ => next.event_grant_relic(Rarity::Common)?,
                }
            }
            Disposal => {
                let card_id = id.parse::<u32>().map_err(|_| "올바르지 않은 카드")?;
                next.burn_card(card_id)?;
                format!("{} · 무료 소각 완료", selection.label)
            }
            DoubleBurn => {
                let card_id = id.parse::<u32>().map_err(|_| "올바르지 않은 카드")?;
                if !next.deck.iter().any(|c| c.id == card_id)
                    || next.event_state.selected_burns.contains(&card_id)
                {
                    return Err("덱의 다른 카드를 선택하세요.".into());
                }
                next.event_state.selected_burns.push(card_id);
                let needed = next.deck.len().min(2);
                if next.event_state.selected_burns.len() < needed {
                    let view = next.event_state.active.as_mut().unwrap();
                    view.choices
                        .iter_mut()
                        .find(|c| c.id == id)
                        .unwrap()
                        .disabled = true;
                    view.description =
                        "한 장을 선택했습니다. 소각할 다른 카드를 선택하세요.".into();
                    *self = next;
                    return Ok(());
                }
                let mut names = Vec::new();
                for card_id in next.event_state.selected_burns.clone() {
                    let card = next.burn_card(card_id)?;
                    names.push(format!("{} #{}", card.kind.definition().name, card_id));
                }
                format!("{} 영구 소각 완료", names.join(", "))
            }
            RandomBurn => return Err("이미 소각 결과를 확인했습니다.".into()),
            Restaurant => {
                if id == "eat" {
                    let mut roster = next.battle.outcome();
                    for p in &mut roster {
                        p.persistent.remove("hunger");
                        p.persistent.remove("hunger_worsened");
                    }
                    next.battle
                        .replace_roster_at_rest(roster)
                        .map_err(engine_error)?;
                }
                "식당을 떠날 준비가 되었습니다.".into()
            }
        };
        next.log(result.clone());
        let view = next.event_state.active.as_mut().unwrap();
        view.result = Some(result);
        view.choices = vec![choice("continue", "원정 경로로 돌아가기", true)];
        *self = next;
        Ok(())
    }
}
