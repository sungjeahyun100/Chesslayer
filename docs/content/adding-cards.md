# 카드 추가

카드 정의 위치는 `crates/chesslayer/src/content/cards/`다. 전투 더미 구현인 `crates/chesslayer/src/cards.rs`는 일반적인 카드 추가 시 수정하지 않는다.

## 복수 선택과 동적 비용

기존 단일 대상 카드의 `effect: fn(&mut Battle, Option<PieceId>)`와 `.with_targets()`는 그대로 사용한다. 복수 선택 카드는 `.with_selection(SelectionMode::Multiple, targets, effect)`를 쓰며, 효과의 두 번째 인자는 `&CardSelection`이다. `selection.targets`에는 선택된 ID가 들어온다. 최소 선택 수, 중복 ID, 현재 보드의 소속과 이동 가능 여부를 효과와 Battle 메서드에서 다시 검증해야 한다. `폰 스톰`은 대상 수에 관계없이 비용 1이다.

강화 카드는 `.with_selection(SelectionMode::Upgrade, targets, effect)`를 사용한다. `selection.upgrades`는 `(PieceId, PieceType)` 쌍이며 비용은 쌍의 수다. Rust가 최소 1개와 마나를 확인한 뒤 효과를 실행한다. 각 경로가 현재 기물 종류에서 이어지는 합법적인 한 단계인지 효과에서 검증한다. `Game::resolve_selection`은 Battle 복제본에서 효과를 실행하고 성공할 때만 보드, 코스트, 손패를 반영한다. 실패 시 세 상태 모두 그대로 남는다.

WASM 명령은 기존 `{"type":"play_card","id":42,"target":9}`와 새 `{"type":"play_card_selection","id":42,"targets":[9,10]}` 또는 `{"type":"play_card_selection","id":42,"upgrades":[{"id":9,"target":8}]}`를 받는다. 선택과 취소는 UI의 임시 상태이며 명령 기록에 남지 않는다. 확인 시에만 명령을 보낸다. 대상 후보와 강화 경로 목록은 Rust 스냅샷에서 받고 React는 사용한 효과를 재계산하지 않는다.

`리미터 해제`는 양측 기물에 즉시 적용되며, 대상은 총 3회 이동할 수 있다. 대상의 남은 이동 횟수는 일반 행동과 별도로 확보하고 턴을 넘겨도 유지한다. 세 번째 이동으로 남은 횟수가 0이 되면 로얄을 포함한 대상이 즉시 사망하고 그 턴도 종료된다. 적은 AI가 남은 이동을 우선 사용한다. 전투 종료 시 효과 상태를 지운다. `일시 강화`는 Battle 보드의 종류만 바꾸며 영구 로스터 종류와 강화 이력은 바꾸지 않는다.

## 1. 완전한 정의 파일 만들기

다음은 기존 `ward.rs`의 효과를 재사용하는 **새 카드 예시**다. 아직 게임에 등록되어 있지 않으며, 아래 등록 단계까지 함께 적용해야 컴파일된다.

`crates/chesslayer/src/content/cards/steady_ward.rs` 전체:

```rust
use super::{ward, CardDefinition, CardKind};

pub fn definition() -> CardDefinition {
    CardDefinition {
        retain: true,
        ..CardDefinition::new(
            CardKind::SteadyWard,
            "머무는 가호",
            1,
            "기물 하나에 가호를 부여합니다. 쓰지 않은 카드는 손에 보존합니다.",
            ward::effect,
        )
        .with_targets(ward::targets)
    }
}
```

이 파일은 양측 어느 기물에나 가호를 줄 수 있고, 사용하지 않으면 손에 남는다. 왕도 대상이 될 수 있다. 현재 `fleeting_ward.rs`도 같은 효과 재사용 방식을 사용한다.

## 2. 필수 속성과 선택 속성

`CardDefinition::new(kind, name, cost, description, effect)`의 인자는 모두 필수다. 복수 선택 카드는 기존 효과 자리에 실행되지 않는 함수를 전달하고 `.with_selection()`으로 실제 효과를 등록한다.

| 속성 | 의미 |
| --- | --- |
| `kind: CardKind` | 등록 enum 값. `SteadyWard`는 JSON에서 `steady_ward`가 된다 |
| `name`, `description: &'static str` | UI와 사용 기록에 전달하는 고정 문자열 |
| `cost: u32` | 마나 비용. 효과 성공 후 차감. `Upgrade` 선택 모드에서는 실제 선택 수를 비용으로 사용 |
| `effect: CardEffect` | `fn(&mut Battle, Option<PieceId>) -> Result<Option<usize>, String>` |

다음 속성은 기본값을 그대로 두거나 필요한 것만 구조체 갱신 문법으로 덮어쓴다.

| 선택 설정 | 기본값 | 공통 처리 |
| --- | --- | --- |
| `.with_targets(targets)` | 대상 없음 | `targeted = true`와 `target_candidates`를 함께 설정. 후보 함수는 `fn(&Battle) -> Vec<PieceId>` |
| `exhaust` | `false` | 사용 후 버림 더미 대신 소멸 더미 |
| `retain` | `false` | 사용하지 않고 턴을 마치면 손패에 유지 |
| `innate` | `false` | 시작 드로우의 우선순위. 선천성 카드가 많아도 1장 드로우·10장 손패 제한은 유지 |
| `ethereal` | `false` | 사용하지 않고 턴을 마치면 소멸. 보존보다 먼저 처리 |

`targeted`만 직접 바꾸지 말고 `.with_targets()`를 사용한다. 키워드는 효과 함수에서 처리하지 않는다. 효과 함수와 대상 함수는 `#[serde(skip)]`이므로 WASM 스냅샷에는 포함되지 않는다.

## 3. 등록하기 — 필수

`crates/chesslayer/src/content/cards/mod.rs`의 세 곳에 추가한다. 기존 항목은 유지한다.

모듈 선언은 파일 첫 줄의 `//!` 문서 주석 아래, 기존 `mod` 선언들과 함께 둔다.

```rust
mod steady_ward;
```

`CardKind` enum 안:

```rust
SteadyWard,
```

`CardKind::definition()`의 `match self` 안:

```rust
Self::SteadyWard => steady_ward::definition(),
```

등록은 이 파일에서 끝난다. `game.rs`의 실행 분기나 화면 대상 필터에 카드 이름을 추가하지 않는다. 기존 `chesslayer::cards::{CardKind, CardDefinition}` 경로도 계속 사용할 수 있다.

## 4. 획득 경로 설정하기

등록만으로 기존 덱이나 보상에 들어가지는 않는다. 다음 중 원하는 경로를 같은 `content/cards/mod.rs`에서 정한다.

**보상에 추가:** 아래 후보 풀에 등록한다. Rust가 원정 RNG로 실제 보상 3칸을 생성하고 보관한다. 화면과 `choose_reward`는 이 생성된 목록의 슬롯을 사용한다.

```rust
pub const REWARD_POOL: &[CardKind] = &[
    CardKind::Ward,
    CardKind::Jeans,
    CardKind::Gun,
    CardKind::SteadyWard,
];
```

**시작 덱은 비워 둔다:** `starter_deck()`은 빈 목록을 반환한다. 새 카드 효과는 Rust 테스트의 명시적인 손패 또는 정상 보상 획득 경로로 검증한다. 카드 ID는 획득 시 공통 할당기가 발급하며 영구 덱 장수에는 제한이 없다.

후보 풀이나 추첨 규칙을 바꾸면 RNG 소비와 과거 명령의 결과가 달라질 수 있다. 저장 버전과 기보를 함께 검토한다. 현재 `choose_reward.index`는 고정 카드 풀 인덱스가 아니다. 휘발성은 매 턴 종료 시 소멸하고 보존은 유지한다. 일반 카드는 종료 처리 전 손패가 5장 이상일 때만 버린다.

## 5. 새 효과·대상 조건 작성하기

`Game::play_card()`가 플레이어 전투 턴, 손패 ID, 마나를 먼저 검사한다. 그다음 효과 함수를 실행하고, `Ok`일 때만 비용을 차감하고 카드를 버림/소멸시킨다. `Err(String)`은 사용자에게 표시된다.

대상 후보는 **UI 안내**다. 명령은 직접 전달될 수도 있으므로 효과 함수에서도 대상을 검증해야 한다. 대상 함수를 공통 판정으로 재사용할 수 있다. 예를 들어 위 카드의 대상을 아군으로 제한하려면 파일에 아래 함수를 추가하고, 생성자의 `ward::effect`를 `effect`, `.with_targets(ward::targets)`를 `.with_targets(targets)`로 바꾼다.

```rust
use crate::{Battle, PieceId, PLAYER};

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle
        .board
        .cells()
        .iter()
        .flatten()
        .filter(|piece| piece.owner == PLAYER)
        .map(|piece| piece.id)
        .collect()
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    let id = target.ok_or("가호를 받을 아군을 선택하세요.")?;
    if !targets(battle).contains(&id) {
        return Err("살아 있는 아군만 선택할 수 있습니다.".into());
    }
    ward::effect(battle, Some(id))
}
```

대상이 없는 카드도 불필요한 `Some(id)`를 허용할지 효과에서 결정한다. 기존 청바지는 거부한다. 특수 진형 예시는 `gun.rs`를 참고한다. `front()`를 대상 목록과 효과 양쪽에서 재사용하며, 총의 범위 상수도 같은 파일에 있다.

### 기존 효과 처리 재사용

| 필요한 동작 | 실제 API / 주의점 |
| --- | --- |
| 기존 가호 부여 | `ward::effect(battle, target)` 또는 `battle.grant_ward(id)`. 지속 상태와 포획 방어는 코어가 처리 |
| 범위의 비로얄 제거 | 보드에서 ID 목록을 모아 `battle.remove_nonroyals(&ids)`. 아군 킹·적 로얄은 자동 보호 |
| 해결된 이동·교환 등 카드 액션 | `battle.apply_resolved_card_effect(&action)`. `DefaultAction`을 적용하되 양측 로얄 제거를 원자적으로 거부하고 턴 유지 |
| 코어 오류를 사용자 오류로 변환 | crate 내부의 `crate::game::engine_error`를 `.map_err(engine_error)`로 사용 |

효과 반환값 `Ok(None)`은 일반 사용 기록, `Ok(Some(n))`은 “n기 제거 (로얄 보호)” 접미사를 만든다. 제거 효과 예시는 `jeans.rs`와 `gun.rs`에 완전한 코드가 있다.

**검증을 끝낸 뒤 상태를 변경한다.** `Game::resolve_selection`은 Battle 복제본에서 효과를 실행하므로 효과 실패 시 실제 Battle은 바뀌지 않는다. 그래도 효과 안에서 각 단계의 적법성을 확인하고, 보드 이동·제거에는 공통 Battle 메서드를 사용한다. `apply_resolved_card_effect()`는 단일 액션 자체를 원자적으로 처리한다.

## 6. 정의 파일만으로 가능한 범위

기존 전투 상태를 읽는 대상 선정, 가호, 비포획 제거, 기존 액션을 조합한 효과, 비용·키워드 변경은 정의 파일에서 작성할 수 있다. 복수 기물과 강화 선택은 `CardSelection`으로 지원한다. 빈 칸 선택, 원정 자원 또는 RNG 변경은 별도 인터페이스 확장이 필요하다.

새 카드의 제목·설명·비용·키워드는 화면에 자동 반영된다. 장식 기호는 `web/src/main.tsx::CardFace`에서 청바지/총 이외에는 `◇`를 사용한다. 고유 그림이 필요할 때만 해당 UI를 수정한다.

## 신규 카드 확인 시나리오

`cargo test -p chesslayer --test new_cards`는 폰 스톰 복수 이동과 실패 시 무차감, 양측 마이너 계열 제거, 비숍 희생과 가호, 일시 강화 분기·동적 비용·영구 로스터 보존·전투 종료 후 원위치 복귀, 리미터의 아군·적 AI 3회 행동과 로얄 사망을 검증한다. `cargo test --workspace`는 기존 카드와 시드 원정의 회귀를 확인한다. `npm run test:e2e --prefix web`은 실제 WASM 손패에서 일시 강화 경로 선택, 취소 무소모, 확정 적용까지 검사한다. 화면에서는 두 폰을 고른 뒤 취소하여 손패·마나·보드가 그대로인지 보고, 다시 선택해 확인한다. 리미터 사용 직후 남은 횟수 3과 이동 후 2→1을 확인하고, 세 번째 이동 직후 대상이 사망하며 턴이 넘어가는지 확인한다. 새 보상 풀은 시드 추첨을 바꾸므로 저장 기록 버전을 7로 올렸다.

## 7. 확인하기

저장소 루트에서:

```sh
cargo fmt --all --check
cargo test -p chesslayer --test content
cargo test -p chesslayer --test prototype
cargo clippy --workspace --all-targets -- -D warnings
npm run wasm --prefix web
npm run build --prefix web
npm run dev --prefix web
```

보상으로 획득한 뒤 이름·비용·대상 표시를 확인한다. 유효한 대상 사용 후 마나 1 감소, 가호 표시, 이동 행동 수 유지, 미사용 시 보존을 확인한다. 대상 제한을 추가했다면 적 대상 거부 시 손패·마나·보드가 바뀌지 않는지도 확인한다. 기존 카드 동작을 보존할 때는 `cargo test --workspace`와 [브라우저 E2E](../TESTING.md)도 실행한다. 새 카드의 새 규칙은 기존 테스트만으로 검증되지 않으므로 실제 행동으로 확인한다.
