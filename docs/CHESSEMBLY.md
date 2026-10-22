# Chesssembly 문법과 공통 액션 계약

이 문서는 현재 Rust crate `chessembly`와 `chesslayer-core`의 계약을 설명합니다. `Program::parse(source)`는 문법을 검사하고 재사용 가능한 프로그램을 만듭니다. `Program::execute(&board, origin)` 및 `interpret(&board, origin, source)`는 입력 보드를 변경하지 않고 순서가 있는 `Vec<DefaultAction>` 후보를 반환합니다. 실행기는 후보를 적용하지 않습니다.

## 문법

공백으로 명령을 구분하고 `;`로 독립된 체인을 구분합니다. `#`부터 줄 끝까지는 주석입니다. `{ ... }`는 조건부 블록이며, `do`/`while`, `repeat(n)`, `label(n)`/`jmp(n)`/`jne(n)`로 반복과 분기를 표현합니다. `not`은 현재 조건 상태를 뒤집고 `end`는 현재 체인을 끝냅니다.

| 분류 | 명령 |
| --- | --- |
| 액션·탐색 | `move(dx,dy)`, `take-move(dx,dy)`, `take(dx,dy)`, `catch(dx,dy)`, `shift(dx,dy)`, `jump(dx,dy)`, `peek(dx,dy)`, `observe(dx,dy)`, `anchor(dx,dy)`, `absolute(x,y)` |
| 기물·색상 조건 | `piece(name)`, `color(name)`, `piece-on(name,dx,dy)`, `color-on(name,dx,dy)`, `enemy(dx,dy)`, `friendly(dx,dy)` |
| 경계 조건 | `bound(dx,dy)`, `edge(dx,dy)`, `corner(dx,dy)`, `edge-top`, `edge-bottom`, `edge-left`, `edge-right`, `corner-top-left`, `corner-top-right`, `corner-bottom-left`, `corner-bottom-right` (방향별 명령에도 `(dx,dy)` 필요) |
| 변환 | `transition(name)` 또는 인자 없는 `transition` |
| 제어 | `{`, `}`, `not`, `do`, `while`, `repeat(n)`, `label(n)`, `jmp(n)`, `jne(n)`, `end` |

`color(name)`은 실행 기물의 색상 이름을 검사하며 현재 턴을 읽지 않습니다. 상대 좌표 `(dx,dy)`는 파일에 `dx`를 더하고 랭크에서 `dy`를 뺍니다. 보드 카탈로그에 색상별 방향이 지정되면 두 성분에 해당 방향을 곱합니다. `absolute(x,y)`는 절대 칸을 가리키며 방향이 음수일 때 보드 양축으로 뒤집습니다. 경계 조건의 방향도 기물 방향을 따릅니다.

`transition`은 이후 생성한 액션에 이름을 기록합니다. 이름은 액션을 적용할 때 보드의 기물 카탈로그에서 찾습니다. 인자 없는 `transition`은 후속 액션의 변환을 해제합니다.

`true`, `false`, `write`, `read`, `read-and`, `read-or`, `read-xor`, `write-anchor`, `read-anchor`, `danger`, `absolute-x`, `absolute-y`는 현재 파싱되지만 실행 효과가 없습니다. `check` 및 `state` 관련 명령은 지원하지 않아 파싱 오류가 납니다. 한 체인에서 100만 명령을 넘으면 `ExecutionLimit`, 블록 깊이가 한계에 닿으면 `BlockLimit`을 반환합니다.

## 공통 액션

`DefaultAction`은 `first: ActionStep`과 선택적 `continuation: Vec<ActionStep>`으로 구성됩니다. 각 단계는 `from`, `take`, `move_to`, `effect`, 선택적 `transition`을 가집니다. 후보의 순서는 Chesssembly 프로그램의 체인과 실행 순서를 따릅니다.

| `Effect` | 적용 의미 |
| --- | --- |
| `Move` | 빈 `move_to`로 이동 |
| `TakeMove` | `take == move_to`로 이동; 대상이 적이면 포획하고 비어 있으면 그냥 이동 |
| `Take` | 적이 있는 `take == move_to`로 이동하며 포획 |
| `TakeJump` | `take`의 적을 포획하고 별도의 빈 `move_to`로 이동 |
| `Catch` | `take`의 적을 포획하고 `from`에 머무름 |
| `Shift` | `from`과 `move_to`의 기물을 교환 |

`chesslayer_core::apply_default_action`은 단계를 순서대로 적용하고 실패한 연속 단계가 있으면 이미 적용된 변경을 되돌립니다. 룰 모듈은 후보를 거부하거나 모듈 고유 상태와 승패를 갱신할 수 있으며, 위 효과의 의미는 공유합니다. 보드, 기물 카탈로그, 액션 효과는 `chesslayer-core`에 정의되어 있습니다.

`crates/chessembly/tests/fixtures/action_snapshots.txt`에는 14개 프로그램의 액션 순서·좌표·효과·변환과 적용 후 보드 셀이 고정돼 있습니다. `cargo test --workspace`가 이 fixture와 현재 Rust 실행 결과를 비교합니다.
