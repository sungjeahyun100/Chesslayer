# 기물 추가

정의 파일은 `crates/chesslayer/src/content/pieces/`에 만든다. 기존 8종도 각 파일에서 숫자 ID, Chesssembly 이름, 표시 이름·능력 설명, 행마, AI 가치를 정의한다. 합법 수 계산은 기존 `chessembly::Program`과 `Battle`을 그대로 사용한다.

## 1. 완전한 정의 파일 만들기

아래는 대각선 두 칸을 뛰는 **새 기물 예시**다. 중간 칸은 검사하지 않고 도착 칸의 아군/적 여부를 공통 계산기가 판정한다. 아직 등록된 콘텐츠가 아니므로 2단계 등록을 함께 적용한다.

`crates/chesslayer/src/content/pieces/dancer.rs` 전체:

```rust
use super::{movement, PieceDefinition};
use crate::PieceType;

pub const DANCER: PieceType = 9;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    description: "대각선 두 칸을 뛰어 이동·포획합니다. 중간 기물은 뛰어넘습니다.",
    ..PieceDefinition::new(DANCER, "dancer", "무희", movement)
};

fn movement() -> String {
    movement::steps(&[(-2, -2), (-2, 2), (2, -2), (2, 2)])
}
```

9는 현재 비어 있는 다음 ID다. 다른 기물을 먼저 추가했다면 사용하지 않는 ID를 선택한다. 종류 ID와 실제 보드 위 `Piece.id`는 다른 값이다.

### 기본 속성

`PieceDefinition::new(kind, key, name, movement)`의 네 인자는 필수다.

| 속성 | 설명 |
| --- | --- |
| `kind: PieceType` | `u16` 숫자 종류 ID. 0은 예약, 기존 1~8 재사용·변경 금지 |
| `key: &'static str` | 보드 카탈로그/DSL의 고유 이름. `piece(dancer)`, `transition(dancer)` 등이 참조하므로 단순한 영문 식별자를 권장 |
| `name: &'static str` | UI·기록의 한글 이름 |
| `movement: fn() -> String` | Chesssembly 소스를 반환. 각 전투 시작 때 파싱 |

다음은 필요한 경우에만 덮어쓴다.

| 선택 속성 | 기본값 | 설명 |
| --- | --- | --- |
| `value: i32` | `10` | 적 AI의 포획 점수. 기존 킹 ID 포획 최우선과 가호 점수 처리는 코어에 남음 |
| `description: &'static str` | `""` | 실제 능력 설명. 등록 시 반드시 작성하며 UI가 그대로 표시 |
| `upgrade: Option<UpgradeDefinition>` | `None` | 강화 대상 ID·이력 key·능력 변화·선행 조건. 현재는 종류당 단일 경로 |
| `initial_double_step: bool` | `false` | 기존 폰 첫 두 칸 규칙 재사용. 일반 행마는 별도로 정의 |

이미지는 `web/src/assets/pieces/<key>-white.svg`, `<key>-black.svg` 두 파일로 등록한다. `PieceImage.tsx`가 Rust의 key·진영으로 조회하므로 화면마다 매핑하지 않는다. 누락 파일은 오류 표식으로 표시하고 콘텐츠 등록 테스트도 실패한다. `provenance.json`에 원본 경로·변경 내용·라이선스 확인 상태를 기록한다. [에셋 안내](../ASSETS.md)를 참고한다.

## 2. 등록하기 — 필수

`crates/chesslayer/src/content/pieces/mod.rs`에서 세 곳에 추가한다.

모듈 선언은 파일 첫 줄의 `//!` 문서 주석 아래, 기존 `mod` 선언들과 함께 둔다.

```rust
mod dancer;
pub use dancer::DANCER;
```

`ALL` 배열 끝에:

```rust
dancer::DEFINITION,
```

이후 `crate::content::pieces::DANCER`로 종류 ID를 참조한다. 기존 기물의 crate 루트 재노출은 호환성을 위해 유지하며 새 ID를 루트에 재노출할 필요는 없다. 등록 목록 순서 대신 각 정의의 `kind`로 프로그램을 조회하므로 별도의 `NAMES`, 유효 범위, 행마 배열 인덱스를 수정하지 않는다. ID·key 중복은 허용하지 않는다.

## 3. 행마와 Chesssembly 연결

`Battle::with_options()`가 `ALL`의 `key`들로 보드 카탈로그를 만들고 `(definition.movement)()`를 `Program::parse()`로 파싱한다. 턴 중 `Battle::legal_actions(from)`가 해당 종류의 프로그램을 실행한다. 반환된 후보 중 하나를 `Battle::apply_piece_action()`으로 적용하면 행동 수, 이동 이력, 가호, 지속 상태, 로얄 판정까지 기존 경로를 따른다.

행마 정의에서 직접 보드를 변경하거나 별도 합법 수 계산기를 만들지 않는다.

### 공통 조합 함수

`content/pieces/movement.rs`:

- `steps(directions)`: 각 방향의 `take-move(dx,dy)`를 `;`로 연결. 나이트, 킹처럼 지정한 변위 한 번의 이동/포획에 사용한다.
- `slides(directions)`: 각 방향의 `take-move(dx,dy) repeat(1)`. 빈 칸을 따라 반복하고 첫 적 포획에서 멈추며 아군을 통과하지 않는다.
- `DIAGONAL`, `ORTHOGONAL`, `KNIGHT`: 기존 방향 순서.
- `all_directions()`: 대각선 다음 직선 순서. 퀸·킹·아마존이 공유한다.

퀸은 `movement::slides(&movement::all_directions())`, 아마존은 이 소스와 `movement::steps(movement::KNIGHT)`를 `;`로 합친다. 템페스트는 `pawn::PAWN_MOVEMENT`를 재사용한다. 공통 동작을 복사하지 않고 이 조합을 사용한다. **후보 순서는 동점 AI의 시드 선택에 영향을 준다.** 기존 기물에서 의미 없이 방향/체인 순서를 바꾸지 않는다.

### 직접 DSL 반환

`movement()`는 다음처럼 기존 DSL을 그대로 반환할 수도 있다.

```rust
fn movement() -> String {
    "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1)".into()
}
```

각 `;`는 원점에서 새 체인을 시작한다. 방향 `+1`인 플레이어의 `(dx,dy)`는 `(file + dx, rank - dy)`이며, 적은 두 축이 반전된다. `move`는 빈 칸으로 이동, `take-move`는 빈 칸 이동 또는 적 포획이다. `repeat(1)`은 “한 번 반복”이 아니라 한 명령 뒤로 돌아가는 명령이다. 전체 문법과 `catch`, `shift`, `transition`의 정확한 조건은 [CHESSEMBLY.md](../CHESSEMBLY.md)에 있다.

## 4. 실제 게임에 등장시키기

등록은 **사용 가능한 종류를 만드는 작업**이다. 자동으로 전투에 배치하지 않는다. 현재 공급처 중 필요한 곳만 바꾼다.

### 적 편성 — 선택한 전투에 출현시키려면

`crates/chesslayer/src/content/mod.rs::encounter(index)`에서 최종 `Encounter { enemies, royal_id: 100 }`를 반환하기 전에 다음을 추가하면 첫 편성에 무희가 생긴다.

```rust
if index == 0 {
    enemies.push(EnemyPiece {
        id: 120,
        kind: pieces::DANCER,
        position: Position::new(2, 1), // c7
    });
}
```

이 함수에는 이미 `EnemyPiece`와 `Position`이 임포트되어 있고 `pieces` 모듈도 선언되어 있다. 같은 전투의 다른 적/아군과 인스턴스 ID가 겹치면 안 된다. 적끼리 위치가 겹쳐도 안 된다. 적 로얄은 종류가 아니라 `royal_id`로 지정하며, 예시에서는 기존 100을 유지한다. 기존 편성 인덱스·맵 구조를 바꿀 필요는 없다.

### 아군 시작 진형 — 필요할 때만

`crates/chesslayer/src/battle.rs::Battle::standard_roster()`에서 원하는 종류를 `crate::content::pieces::DANCER`로 바꾼다. 기존 ID와 위치를 유지하고 `KING` 한 개를 남겨야 한다. 모든 캐릭터에 공통 적용되며 부활 원점도 이 함수에서 읽는다. `PlayerPiece`의 `moved`, `upgrades`, `persistent`는 개별 인스턴스 상태이며 종류 정의에 넣지 않는다.

### 휴식 강화·다른 콘텐츠의 참조 — 필요할 때만

현재 휴식의 폰 → 템페스트 폰 경로는 `pawn.rs`의 `upgrade`에 정의한다. `RestUpgrade`, `rest_options`, `upgrade_paths`는 같은 정의를 읽는다. 새 단일 경로는 해당 기물 정의의 `upgrade`에 추가하며 대상 ID도 등록해야 한다. 여러 분기나 자원 비용을 도입하려면 명령의 대상 선택과 실제 비용 검증도 함께 구현해야 한다. 카드는 기존 종류 ID를 정확히 검사하므로, 새 기물에 룩 행마를 주더라도 청바지의 제거 대상이나 총의 진형으로 자동 편입되지 않는다.

## 5. 특수 능력의 경계

- 기존 DSL로 표현 가능한 조건부 행마·제자리 포획·교환·종류 변환은 정의의 `movement()`에서 작성할 수 있다. 기존 액션 적용기가 실행한다.
- `initial_double_step: true`는 폰/템페스트의 현재 첫 두 칸 규칙을 재사용한다. 이동한 적이 없고 플레이어 rank 6 / 적 rank 1일 때, 두 칸 모두 비어 있는 후보만 일반 후보 뒤에 붙인다. DSL은 `movement.rs`, 이동 이력 검사는 `Battle`이 담당한다.
- 가호·역습·로얄 보호는 종류별 행마가 아니라 공통 전투 규칙이다. 새 기물에도 기존 규칙이 적용된다.
- 매 턴 발동, 포획 후 자원 획득, 새로운 지속 상태 효과 등은 현재 정의에 콜백이 없다. `battle.rs` 또는 `game.rs`의 해당 실행 시점에 실제 규칙을 추가해야 한다. `persistent`에 값을 저장하는 것만으로 능력이 실행되지 않는다.
- `transition(name)`은 등록된 `key`를 사용한다. 자동 승격을 뜻하지 않으며 어떤 경우에 변환할지는 DSL에 명시한다. 로얄 인스턴스는 전투 중 ID로 추적하고 다음 전투의 아군 킹 검사는 `KING` 종류를 요구하므로 아군 킹을 다른 종류로 변환하는 규칙은 별도 검토가 필요하다.
- `danger`, `check`, `state`, `set-state`, `if-state`는 현재 DSL에서 지원하지 않는다. 체스의 체크메이트·캐슬링·앙파상도 새로 구현되지 않는다.

## 6. 확인하기

```sh
cargo fmt --all --check
cargo test -p chesslayer --test content
cargo test -p chesslayer --test battle
cargo test -p chessembly
cargo clippy --workspace --all-targets -- -D warnings
npm run wasm --prefix web
npm run build --prefix web
npm run dev --prefix web
```

등록 테스트는 모든 기물의 ID/key 중복, 보드 카탈로그 연결, DSL 파싱·후보 실행을 확인한다. 정의에 문법 오류가 있으면 그 종류가 아직 보드에 없어도 전투 초기화가 실패한다.

예시 무희를 실제 편성/초기 진형에 놓고 이미지·이름과 대각선 두 칸 후보를 확인한다. 양측 방향, 보드 경계, 도착 칸의 아군 차단/적 포획, 중간 기물 점프를 확인한다. 첫 두 칸을 켰다면 시작 랭크·이동 이력·중간 점유도 확인한다. 이동/포획은 `Battle::legal_actions()`에서 받은 후보로 수행하고 다음 전투의 위치·종류 계승도 확인한다. 기존 회귀 검증 전체 명령은 [TESTING.md](../TESTING.md)에 있다.
