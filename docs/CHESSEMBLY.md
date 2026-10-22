# Chesssembly 문법과 공통 액션 계약

이 문서는 현재 Rust crate `chessembly`와 `chesslayer-core`의 계약을 설명합니다. `Program::parse(source)`는 문법을 검사하고 재사용 가능한 프로그램을 만듭니다. `Program::execute(&board, origin)` 및 `interpret(&board, origin, source)`는 입력 보드를 변경하지 않고 순서가 있는 `Vec<DefaultAction>` 후보를 반환합니다. 실행기는 후보를 적용하지 않습니다.

## 문법

공백으로 명령을 구분하고 `;`로 독립된 체인을 구분합니다. `#`부터 줄 끝까지는 주석입니다. `{ ... }`는 조건부 블록이며, `do`/`while`, `repeat(n)`, `label(n)`/`jmp(n)`/`jne(n)`로 반복과 분기를 표현합니다. `not`은 현재 조건 상태를 뒤집고 `end`는 현재 체인을 끝냅니다.

| 분류 | 명령 |
| --- | --- |
| 액션·탐색 | `move(dx,dy)`, `take-move(dx,dy)`, `take(dx,dy)`, `catch(dx,dy)`, `shift(dx,dy)`, `jump(dx,dy)`, `peek(dx,dy)`, `observe(dx,dy)`, `anchor(dx,dy)`, `absolute(x,y)`, `absolute-x(x)`, `absolute-y(y)`, `write-anchor(n)`, `read-anchor(n)` |
| 기물·색상 조건 | `piece(name)`, `color(name)`, `piece-on(name,dx,dy)`, `color-on(name,dx,dy)`, `enemy(dx,dy)`, `friendly(dx,dy)` |
| 경계 조건 | `bound(dx,dy)`, `edge(dx,dy)`, `corner(dx,dy)`, `edge-top`, `edge-bottom`, `edge-left`, `edge-right`, `corner-top-left`, `corner-top-right`, `corner-bottom-left`, `corner-bottom-right` (방향별 명령에도 `(dx,dy)` 필요) |
| 변환 | `transition(name)` 또는 인자 없는 `transition` |
| 제어 | `{`, `}`, `not`, `true`, `false`, `write(n)`, `read(n)`, `read-and(n)`, `read-or(n)`, `read-xor(n)`, `do`, `while`, `repeat(n)`, `label(n)`, `jmp(n)`, `jne(n)`, `end` |

`color(name)`은 실행 기물의 색상 이름을 검사하며 현재 턴을 읽지 않습니다. 상대 좌표 `(dx,dy)`는 파일에 `dx`를 더하고 랭크에서 `dy`를 뺍니다. 보드 카탈로그에 색상별 방향이 지정되면 두 성분에 해당 방향을 곱합니다. `absolute(x,y)`는 절대 칸을 가리키며 방향이 음수일 때 보드 양축으로 뒤집습니다. 경계 조건의 방향도 기물 방향을 따릅니다.

`transition`은 이후 생성한 액션에 이름을 기록합니다. 이름은 액션을 적용할 때 보드의 기물 카탈로그에서 찾습니다. 인자 없는 `transition`은 후속 액션의 변환을 해제합니다.

## 조건 상태와 실행 컨텍스트

각 `;` 체인은 새로운 실행 컨텍스트로 시작합니다. 처음 탐색 좌표는 `origin`, 조건 상태는 참이며, 레지스터와 앵커 슬롯은 모두 초기화되지 않은 상태입니다. 슬롯, 탐색 좌표, 블록 프레임, 변환 이름, 액션 후보는 실행 중에만 유지됩니다. 같은 `Program`을 다시 실행해도 저장 상태가 이어지지 않습니다. 보드나 게임 상태에 슬롯 값을 쓰지 않습니다.

조건이 거짓이면 일반 명령은 실행하지 않습니다. 최상위 프레임에서는 그 체인을 끝내고, 조건부 블록 안에서는 해당 블록의 닫는 괄호로 이동합니다. `not`, `true`, `false`, `write`, `read` 계열, `while`, `jmp`, `jne`, `label`은 거짓 조건에서도 실행할 수 있습니다. 따라서 `false true move(-1,0)`은 이동 후보를 만들지만 `false move(-1,0) true`는 이동 명령에서 체인이 끝납니다. `{ ... }`는 부모 조건이 참일 때만 들어가며 자식 프레임의 조건·탐색 좌표 변경은 닫는 괄호에서 사라집니다. 슬롯 값은 같은 체인 전체에서 공유되어 블록을 벗어나도 남습니다. `while`은 거짓 조건에서 반복을 끝내고 조건을 참으로 되돌립니다. `jmp`와 `jne`는 기존 조건 분기 규칙을 사용합니다.

| 명령 | 인자 및 실행 의미 | 변경 상태·오류 | 예제 |
| --- | --- | --- | --- |
| `true` | 인자 없음. 현재 프레임의 조건을 참으로 설정 | 조건만 변경; 인자를 주면 파싱 오류 | `false true move(-1,0)` |
| `false` | 인자 없음. 현재 프레임의 조건을 거짓으로 설정 | 조건만 변경; 인자를 주면 파싱 오류 | `false not move(-1,0)` |
| `write(n)` | 현재 조건 값을 슬롯 `n`에 저장 | 해당 레지스터를 덮어씀; 슬롯 범위 밖은 파싱 오류 | `false write(0) true` |
| `read(n)` | 슬롯 `n`의 값을 현재 조건으로 설정 | 조건 변경; 초기화 전 읽기는 `UninitializedRegister(n)` | `true write(0) false read(0) move(-1,0)` |
| `read-and(n)` | 현재 조건과 슬롯 값의 논리 AND | 조건 변경; 초기화 전 읽기는 실행 오류 | `false write(0) true read-and(0)` |
| `read-or(n)` | 현재 조건과 슬롯 값의 논리 OR | 조건 변경; 초기화 전 읽기는 실행 오류 | `true write(0) false read-or(0) move(-1,0)` |
| `read-xor(n)` | 현재 조건과 슬롯 값의 논리 XOR | 조건 변경; 초기화 전 읽기는 실행 오류 | `true write(0) false read-xor(0) move(-1,0)` |

슬롯 번호 `n`은 10진 정수 `0..=255`입니다. 음수, 256 이상, 정수가 아닌 인자, 잘못된 인자 개수는 `Program::parse`에서 `Error::Parse`가 됩니다. 읽기 명령은 조건이 거짓이어도 슬롯을 확인하므로 초기화되지 않은 슬롯을 조용히 건너뛰지 않습니다. `write`도 거짓 조건에서 실행되어 `false`를 저장할 수 있습니다. 각 명령은 즉시 액션 후보를 만들지 않습니다.

## 탐색 좌표 저장과 절대 좌표

| 명령 | 인자 및 실행 의미 | 변경 상태·오류 | 예제 |
| --- | --- | --- | --- |
| `write-anchor(n)` | 현재 탐색 좌표를 앵커 슬롯 `n`에 저장 | 해당 앵커를 덮어씀; 슬롯 범위 밖은 파싱 오류 | `write-anchor(0) anchor(-1,0)` |
| `read-anchor(n)` | 앵커 슬롯 `n`의 좌표를 현재 탐색 위치로 복원 | 탐색 좌표 변경; 초기화 전 읽기는 `UninitializedAnchor(n)` | `write-anchor(0) peek(-1,0) read-anchor(0) move(-1,0)` |
| `absolute-x(x)` | 현재 탐색 좌표의 X만 절대 파일 `x`로 변경 | 유효 칸이면 좌표 변경; 범위 밖·마스크 제외 칸이면 조건만 거짓 | `absolute-x(0) move(1,0)` |
| `absolute-y(y)` | 현재 탐색 좌표의 Y만 절대 랭크 `y`로 변경 | 유효 칸이면 좌표 변경; 범위 밖·마스크 제외 칸이면 조건만 거짓 | `absolute-y(0) move(0,-1)` |

앵커도 레지스터와 같이 체인별로 분리되며 `n`은 `0..=255`입니다. 앵커 명령과 절대 좌표 명령은 일반 명령이므로 조건이 거짓일 때 실행되지 않습니다. `anchor(dx,dy)`와 `peek(dx,dy)`로 탐색 좌표를 바꾼 뒤 저장하거나, 저장한 좌표를 `absolute(x,y)` 이후 복원할 수 있습니다. 블록 안에서 저장한 앵커는 같은 체인의 바깥에서도 읽을 수 있지만, 복원한 좌표 자체는 현재 프레임에만 적용됩니다. 어느 명령도 입력 보드의 기물을 변경하지 않습니다.

`absolute-x`와 `absolute-y`는 `absolute(x,y)`와 같이 기물 방향이 음수일 때 해당 축을 보드 끝에서 뒤집습니다. X는 `width - 1 - x`, Y는 `height - 1 - y`로 계산하고, 지정하지 않은 축은 현재 좌표 그대로 둡니다. 결과 좌표가 보드 밖이거나 플레이 가능 마스크에서 제외되면 기존 좌표를 유지하고 조건을 거짓으로 만듭니다. 좌표 인자가 정수가 아니거나 개수가 틀리면 파싱 오류가 납니다. `false`가 된 뒤 `true`나 `not`으로 조건을 복구하면 이전 탐색 좌표에서 이어집니다.

## 지원하지 않는 명령

| 명령 | 처리 |
| --- | --- |
| `danger(dx,dy)` | 지원하지 않음. 사용 즉시 `Error::Parse("unsupported expression: danger")` |
| `check` | 지원하지 않음. 파싱 오류 |
| `state` | 지원하지 않음. 파싱 오류 |
| `set-state` | 지원하지 않음. 파싱 오류 |
| `if-state` | 지원하지 않음. 파싱 오류 |

과거 C++ 구현과 현재 저장소의 이전 문서에는 레지스터·앵커·절대축 명령의 실행 규격이 없습니다. 과거 C++ 실행기는 이 구문을 파싱만 하고 아무 효과 없이 지나갔습니다. 위 슬롯 의미와 범위, 미초기화 오류, 절대축의 한 축 변경은 이번에 정의한 계약입니다. 기존 `absolute`의 방향 변환·유효 칸 검사와 기존 제어 흐름 규칙은 유지했습니다. 한 체인에서 100만 명령을 넘으면 `ExecutionLimit`, 블록 깊이가 한계에 닿으면 `BlockLimit`을 반환합니다.

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

`crates/chessembly/tests/fixtures/action_snapshots.txt`에는 21개 프로그램의 액션 순서·좌표·효과·변환과 적용 후 보드 셀이 고정돼 있습니다. `cargo test --workspace`가 이 fixture와 현재 Rust 실행 결과를 비교합니다.
