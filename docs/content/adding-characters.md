# 캐릭터 추가

현재 캐릭터의 차이는 **설명과 시작 유물 한 개**다. 파일 위치는 `crates/chesslayer/src/content/characters/`이며, 기존 `vanguard.rs`, `wanderer.rs`, `guardian.rs`와 같은 형태로 정의한다.

## 1. 완전한 정의 파일 만들기

아래는 기존 가속 유물을 사용하는 **새 캐릭터 예시**다. 현재 등록된 캐릭터가 아니며 2단계 등록을 함께 적용해야 선택할 수 있다.

`crates/chesslayer/src/content/characters/tactician.rs` 전체:

```rust
use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "tactician",
    name: "전술가",
    title: "다음 한 수까지",
    description: "가속으로 두 수를 연속해서 둡니다. 적의 두 수까지 고려하세요.",
    relic: Relic::Haste,
};
```

필수 속성은 다음 다섯 개다. 현재 캐릭터 구조에는 추가 선택 속성이 없다.

| 속성 | 설명 |
| --- | --- |
| `id: &'static str` | 다른 캐릭터와 겹치지 않는 문자열. UI 선택값과 `Game::new(id, seed)`의 조회 키 |
| `name: &'static str` | 선택 화면과 런 스냅샷의 이름 |
| `title: &'static str` | 선택 화면의 짧은 제목 |
| `description: &'static str` | 현재 지원하는 기능을 설명하는 문장 |
| `relic: Relic` | 시작 유물 한 개. 아래 기존 enum 중 하나 |

기존 문자열 ID를 바꾸지 않는다. 파일명과 ID를 같게 유지하면 찾기 쉽지만, 실제 조회는 파일명이 아닌 `id`를 사용한다.

## 2. 등록하기 — 필수

`crates/chesslayer/src/content/characters/mod.rs`에 모듈을 선언한다.

모듈 선언은 파일 첫 줄의 `//!` 문서 주석 아래, 기존 `mod` 선언들과 함께 둔다.

```rust
mod tactician;
```

같은 파일의 `ALL` 끝에 추가한다.

```rust
tactician::DEFINITION,
```

`ALL`의 순서가 선택 화면의 순서다. `characters()`가 이 목록을 반환하고 `chesslayer-web::catalog()`가 JSON으로 전달한다. `Game::new()`도 같은 목록을 사용하므로 UI나 별도 ID `match`를 수정하지 않는다. 같은 시작 유물을 쓰는 캐릭터가 여러 개 있어도 된다.

## 3. 고유 능력과 콘텐츠 연결

현재는 `relic` 선택으로 기존 유물 기능을 연결한다.

| 유물 | 실제 효과 | 구현 위치 |
| --- | --- | --- |
| `Relic::Haste` | 양 진영이 턴마다 최대 2행동 | `content/mod.rs::HASTE_ACTIONS`, `game.rs::start_battle()` → `BattleOptions.actions_per_turn` |
| `Relic::Whirlwind` | 전투 시작 시 적들끼리 시작 위치 교환 | `game.rs::start_battle()`, 공통 시드 RNG |
| `Relic::SelfDefense` | 역습으로 아군이 제거되면 겹친 적도 제거하되 적 로얄 제외 | `game.rs::start_battle()` → `BattleOptions.self_defense`, `battle.rs::with_options()` |

유물의 이름·설명은 `content/mod.rs`의 `Relic`에 있다. 유물은 보물·엘리트 보상에서도 획득하므로 캐릭터 정의에 실행 코드를 복제하지 않는다. UI의 캐릭터 아이콘은 기존 유물 종류에서 결정된다.

### 초기 구성

`Game::new()`의 현재 순서는 다음과 같다.

1. 캐릭터 ID를 `characters()`에서 찾는다.
2. `content/cards/mod.rs::starter_deck()`으로 공통 시작 덱을 만든다.
3. `Battle::standard_roster()`로 공통 백 16기를 만든다.
4. `relics`에 선택 캐릭터의 `relic` 하나를 넣는다.
5. 시드로 맵을 만들고 지도 단계에서 시작한다.

시작 유물의 전투 효과는 첫 전투 노드를 선택할 때 적용된다. 모든 캐릭터의 공통 덱을 바꾸려면 `content/cards/mod.rs`, 공통 진형을 바꾸려면 `battle.rs::standard_roster()`를 수정한다. 캐릭터 정의에 카드/기물 ID를 넣는 필드는 현재 없다.

## 4. 현재 미지원 기능

다음은 정의 파일에 임의의 필드를 추가해도 실행되지 않는다.

- 캐릭터별 시작 덱·진형·마나
- 여러 시작 유물 또는 시작 유물 없음
- 유물과 독립적인 캐릭터 전용 패시브·액티브 스킬
- 레벨, 경험치, 성장 트리, 해금 조건
- 캐릭터 전용 카드 보상 풀

이 작업에서는 이런 기능을 위한 추상화나 이벤트 훅을 만들지 않았다. 요구가 생기면 `Character`와 `Game::new()` 등 실제 사용부를 함께 확장해야 한다. 특히 진형을 다르게 만들려면 `RestRevive`와 `rest_options`가 현재 공통 표준 진형에서 부활 원점을 조회하는 점도 검토해야 한다.

**새 유물 효과**는 기존 유물 재사용과 다르다. `content/mod.rs::Relic`의 종류·이름·설명, 실제 발동 시점인 `game.rs`/`battle.rs`, 필요 시 `map.rs`의 유물 공급 목록을 수정해야 한다. 새 유물을 화면에 표시하려면 `web/src/types.ts`의 유물 문자열 타입과 `web/src/main.tsx`의 `relicIcons`도 추가한다. 캐릭터 파일 하나만으로 새 전투 규칙이 생기는 것으로 취급하지 않는다.

## 5. 확인하기

```sh
cargo fmt --all --check
cargo test -p chesslayer --test content
cargo test -p chesslayer --test prototype
cargo clippy --workspace --all-targets -- -D warnings
npm run wasm --prefix web
npm run build --prefix web
npm run dev --prefix web
```

등록 테스트는 문자열 ID 중복을 검사하고 모든 등록 캐릭터에 대해 `Game::new()`가 성공하며 이름과 시작 유물이 연결되는지 확인한다.

브라우저에서 새 게임을 시작해 전술가의 이름·제목·설명·가속 아이콘을 확인한다. 선택 후 지도에 진입하고 첫 전투를 시작한다. 가속 기록, 양측 2행동, 공통 16기 진형과 10장 덱이 유지되는지 확인한다. 유물 재사용이므로 기존 가속 시나리오도 계속 통과해야 한다. 전체 회귀 검증은 [TESTING.md](../TESTING.md)를 따른다.
