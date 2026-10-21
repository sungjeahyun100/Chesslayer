# 변형체스 공통 코어와 Chesssembly 실행기

C++20 프로젝트입니다. `Core`는 보드·기물 식별자·결과 및 룰 모듈의 최소 진입점만 정의합니다. `Chessembly`는 독립적인 파서와 순수 실행기입니다. `StandardChess`는 일반 체스의 초기 포지션, 기물 정의, 보드 경계, 승리 판정과 특수 상태를 소유하는 하나의 룰 모듈입니다. CLI는 이 모듈을 선택해 실행합니다.

## 빌드 및 검증

```bash
cmake -S . -B build
cmake --build build -j
ctest --test-dir build --output-on-failure
./build/chess_engine
```

기존 `tests.cpp`는 시작 포지션과 추가 포지션의 Perft, FEN, 캐슬링, 앙파상, 승격, 체크 및 무승부 판정, Undo를 검증합니다. `interpreter_tests.cpp`는 8×8 이외 보드, 경계 마스크, 색상, 파서 오류, 액션의 종류와 적용, 반복 호출의 결정성을 독립적으로 검사합니다.

## 모듈 경계

- `core.hpp`: `Board`, `Piece`, `GameResult`, `RuleModule` 콘셉트. 기물은 숫자 ID·소유자·개별 ID만 공통이며, 이름과 상대 방향은 해당 모듈이 보드에 제공합니다. 기물 정의의 나머지 스키마는 자유롭습니다. 보드는 가로·세로 크기 및 선택적인 유효 칸 마스크를 가집니다.
- `chessembly.hpp/.cpp`: `Program`으로 파싱한 코드를 재사용할 수 있습니다. `Program::execute(const Board&, Coord)`는 보드를 복사하지 않습니다. 요청한 `chessemblyInterpreter(Board, pair<int,int>, string)`도 제공합니다. 외부 상태, 보드, 정적 가변 상태를 변경하지 않습니다.
- `standard_chess.hpp`: 독립된 `Rules` 모듈. 일반 기물의 기본 행마는 공통 실행기에서 얻습니다. 폰의 특수 행마, 킹 안전, 캐슬링, 앙파상, 승격, FEN, 승리·무승부와 이력은 이 모듈 안에서 처리합니다. 일반 체스는 실행기에 내장되어 있지 않습니다.
- `main.cpp`: 일반 체스 CLI만 담당합니다.

새 모듈은 `RuleModule`의 네 진입점인 `pieceDefinitions()`, `boardBoundary()`, `initialPosition()`, `victoryCondition(State&)`를 제공하고, 동일한 `Board`와 `defaultAction`을 사용하면 됩니다. 기물 정의의 반환 타입이나 모듈별 추가 상태의 형태는 강제하지 않습니다.

## 공통 액션 계약

`defaultAction`은 첫 단계와 선택적 후속 단계로 이루어진 순서 있는 액션입니다. 단일 단계는 추가 할당이 없고, 후속 단계는 연속 행마를 의미 손실 없이 담습니다. 원본 JIT의 연속 명령은 각각 별도 후보를 출력하므로 인터프리터도 그 출력 순서를 유지합니다. 한 단계의 `from`, `take`, `moveTo`, `effect`, 선택적 `transition`은 모든 모듈에서 같은 의미입니다.

| 효과 | 의미 |
|---|---|
| `Move` | 빈 `moveTo`로 이동 |
| `TakeMove` | `take == moveTo`로 이동하고 적이 있으면 포획 |
| `Take` | 적이 있는 `take == moveTo`로 이동하고 포획; 원본 `take` 명령의 구별 유지 |
| `TakeJump` | 별도 `take`에서 포획하고 빈 `moveTo`로 이동 |
| `Catch` | `take`의 적을 포획하고 `from`에 머무름 |
| `Shift` | `from`과 `moveTo`의 기물을 교환 |

`applyDefaultAction`은 이 효과를 한곳에서 적용하며, 잘못된 연속 단계는 이미 적용한 단계를 롤백합니다. 모듈은 액션 후보를 거부하거나 자신의 상태·승리 규칙을 갱신할 수 있습니다. 액션의 위 효과를 재정의해서는 안 됩니다. 실행기는 액션 후보만 만들며 적용하지 않습니다. `transition`은 적용 시 모듈이 등록한 기물 이름으로 종류를 변경합니다.

## 문법과 원본의 모호한 부분

원본 참조는 작업 환경의 `chessembly_origin/chessembly-bot/src/chessembly.rs`, `chessembly/behavior.rs`, `chessembly/jit_compiler.rs`와 `jit_compiler/helper.rs`입니다. 다음 원본 명령을 파싱합니다: `move`, `take`, `take-move`, `catch`, `shift`, `jump`, `peek`, `observe`, `anchor`, `absolute`, `piece`, `color`, `piece-on`, `color-on`, `enemy`, `friendly`, 경계·모서리 조건, `transition`, 블록, `not`, `do`/`while`, `repeat`, `label`/`jmp`/`jne`, `end`. `state` 관련 명령과 `check`는 명시적으로 거부합니다. `color(name)`은 **현재 실행 중인 기물**의 색상 이름과 비교하며 턴은 참조하지 않습니다.

상대 좌표는 원본처럼 `(dx,dy)`의 `dy`가 행 번호에서 빠집니다. 모듈이 색상별 `colorDirections`를 지정하면 두 성분 모두 그 방향으로 반전됩니다. 파싱한 `Program`은 호출 간 상태를 보유하지 않습니다.

원본 JIT에는 `true`/`false`, `write`/`read` 계열, `danger`, `absolute-x`/`absolute-y`의 실행 분기가 없습니다. 이 구문들은 원본 실행 경로와 같이 효과 없는 명령으로 유지합니다. 이를 유효한 새 의미로 간주하지 않습니다. 원본 `jump` 헬퍼는 보드 밖일 때만 안쪽 목적지 검사를 하는 모순된 분기를 포함합니다. 현재 구현은 `take` 후 빈 유효 칸으로 뛰어 `TakeJump`를 만드는 해석을 사용하며, 이 한 동작은 원본의 실행 결과와 완전 일치한다고 주장하지 않습니다. 원본의 `take` 결과가 이미 출력된 뒤 `jump`에서 제거되는 순서는 보존합니다. 원본 JIT의 무한 루프 가능성을 막기 위해 한 체인에서 100만 명령을 넘거나 블록 중첩이 32단계를 넘으면 오류를 냅니다.

체섬블리 실행기는 일반 체스의 체크·캐슬링·앙파상·승격을 모르며, 액션 실행 이후의 합법성은 룰 모듈이 판정합니다.
