use crate::content::{Rng, BASE_MANA, DRAW_PER_TURN, HAND_LIMIT};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardKind {
    Jeans,
    Ward,
    Gun,
    FleetingWard,
}
#[derive(Clone, Serialize)]
pub struct CardDefinition {
    pub kind: CardKind,
    pub name: &'static str,
    pub cost: u8,
    pub description: &'static str,
    pub targeted: bool,
    pub exhaust: bool,
    pub retain: bool,
    pub innate: bool,
    pub ethereal: bool,
}
impl CardKind {
    pub fn definition(self) -> CardDefinition {
        let mut d = CardDefinition {
            kind: self,
            name: "가호",
            cost: 1,
            description: "기물 하나가 다음 일반 포획을 한 번 막습니다. 공격자는 제자리에 남습니다.",
            targeted: true,
            exhaust: false,
            retain: false,
            innate: false,
            ethereal: false,
        };
        match self {
            Self::Ward => (),
            Self::Jeans => {
                d.name = "청바지";
                d.cost = 0;
                d.description = "양 진영의 모든 룩과 퀸을 추방합니다. 양측 로얄은 보호됩니다.";
                d.targeted = false;
                d.exhaust = true;
                d.innate = true;
            }
            Self::Gun => {
                d.name = "알레킨의 총";
                d.cost = 3;
                d.description = "같은 파일에 전방부터 룩·룩·퀸을 연속 배치하세요. 맨 앞 룩을 선택하면 그 앞의 3개 파일에서 양측 기물을 제거합니다. 로얄 제외.";
                d.retain = true;
            }
            Self::FleetingWard => {
                d.name = "찰나의 가호";
                d.cost = 0;
                d.description = "가호와 같은 효과. 쓰지 않고 턴을 마치면 소멸합니다.";
                d.ethereal = true;
            }
        }
        d
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Card {
    pub id: u32,
    pub kind: CardKind,
}
#[derive(Clone, Default)]
pub struct CardCombat {
    pub draw: Vec<Card>,
    pub hand: Vec<Card>,
    pub discard: Vec<Card>,
    pub exhausted: Vec<Card>,
    pub mana: u8,
}
impl CardCombat {
    pub fn start(deck: &[Card], rng: &mut Rng) -> Self {
        let mut draw = deck.to_vec();
        rng.shuffle(&mut draw);
        // pop() draws from the end. Stable partition puts innate cards first.
        draw.sort_by_key(|c| c.kind.definition().innate);
        let mut state = Self {
            draw,
            ..Self::default()
        };
        state.begin_turn(rng);
        state
    }
    pub fn begin_turn(&mut self, rng: &mut Rng) {
        self.mana = BASE_MANA;
        for _ in 0..DRAW_PER_TURN {
            if self.hand.len() >= HAND_LIMIT {
                break;
            }
            if self.draw.is_empty() {
                self.draw = std::mem::take(&mut self.discard);
                rng.shuffle(&mut self.draw);
            }
            if let Some(card) = self.draw.pop() {
                self.hand.push(card);
            } else {
                break;
            }
        }
    }
    pub fn end_turn(&mut self) {
        let mut retained = vec![];
        for card in self.hand.drain(..) {
            let d = card.kind.definition();
            if d.ethereal {
                self.exhausted.push(card);
            } else if d.retain {
                retained.push(card);
            } else {
                self.discard.push(card);
            }
        }
        self.hand = retained;
    }
    pub fn spend(&mut self, index: usize) {
        let card = self.hand.remove(index);
        let d = card.kind.definition();
        self.mana -= d.cost;
        if d.exhaust {
            self.exhausted.push(card);
        } else {
            self.discard.push(card);
        }
    }
}
pub fn starter_deck() -> Vec<Card> {
    use CardKind::*;
    [
        Jeans,
        Ward,
        Ward,
        Ward,
        Ward,
        Ward,
        Ward,
        Gun,
        FleetingWard,
        FleetingWard,
    ]
    .into_iter()
    .enumerate()
    .map(|(i, kind)| Card {
        id: i as u32 + 1,
        kind,
    })
    .collect()
}
