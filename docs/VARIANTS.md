# 템페스트·바운싱 구현 — 2026-10-23

`develop`의 1차 폴리싱(`bf27a4a`) 위에 두 계열을 연결했다. 기물 정의 → Chesssembly → Battle → Game → WASM → React의 기존 구조를 유지한다. `POLISHING.md`의 임시 템페스트 행마와 저장 버전 1 설명은 이 문서로 대체한다.

## 기물과 획득

| ID / key | 행마 | 획득 |
| --- | --- | --- |
| 8 / tempest_pawn | 앞·좌우 빈칸 1칸, 앞 대각 포획, 전방 2칸 포획 | 일반 폰의 휴식 강화 |
| 9 / tempest_rook | 대각 한 칸 후 그 대각을 구성하는 두 직교 방향으로 슬라이딩 | 템페스트 폰의 최종 랭크 승격 |
| 10 / tempest_bishop | 직교 한 칸 후 같은 진행 방향의 두 대각선으로 슬라이딩 | 동일 |
| 11 / tempest_knight | 대각 한 칸 후 `(3,2)`·`(2,3)`까지 추가 도약, 별도로 직교 3칸 도약 | 동일 |
| 12 / tempest_queen | 템페스트 룩+비숍 정의 재사용 | 동일 |
| 13 / bouncing_pawn | 일반 폰 행마+진영에 관계없는 반사벽 | 일반 폰의 휴식 강화 |
| 14 / bouncing_rook | 직교 진행 후 첫 벽에서 양쪽 직교 방향으로 꺾음 | 바운싱 폰의 최종 랭크 승격 |
| 15 / bouncing_bishop | 대각 진행 후 첫 벽에서 x 또는 y 방향 성분 반전 | 동일 |
| 16 / bouncing_queen | 바운싱 룩+비숍 정의 재사용 | 동일 |

모든 종류는 백·흑에 같은 정의를 사용한다. Chesssembly의 진영 방향 변환으로 흑을 처리하며, 별도 JavaScript 합법 수 계산은 없다. 이전 ID 1~8과 key는 유지한다. 폰 1의 휴식 경로는 8/13 두 개뿐이며 상위 기물로의 휴식 강화는 없다.

## 원본 템페스트의 의미

로컬 덱체스 저장소 `var-chess-brainfuck/engine/src/pieces/default_pieces/tempest_*.rs`의 정의를 확인했다. `take-move`는 중간 칸에도 후보를 만들기 때문에 룩·비숍·나이트는 첫 경유 칸에서 멈추거나 포획할 수 있다. 그 칸에 아군이 있으면 진입과 이후 분기를 차단하고, 적을 포획하면 그 이후 경로를 생성하지 않는다. 이후 직선 경로도 같은 차단을 적용한다.

나이트의 대각 경유 뒤 `(2,1)`/`(1,2)` 추가 단계와 독립적인 `take-move(3,0)` 등은 원본처럼 그 단계의 목적지만 검사한다. 따라서 첫 대각 경유는 반드시 열려 있어야 하지만, 추가 도약의 중간 칸이나 직교 3칸 도약의 중간 칸은 검사하지 않는다. 전체를 일반 나이트처럼 뛰어넘도록 바꾸지 않았다.

템페스트 폰은 `move(0,1); move(1,0); move(-1,0); take(0,2); take(1,1); take(-1,1)`를 사용한다. 옆 포획·후진·초기 두 칸 전진을 제거했다. `take(0,2)`는 일반 폰의 두 번 `move(0,1)`과 달리 중간 칸을 보지 않으며, 두 칸 앞에 적이 있어야 후보를 생성한다. 아군이나 빈칸에는 두 칸 전진할 수 없다.

## 반사의 실제 해석

원본 `bouncing_rook.rs`, `bouncing_bishop.rs`, `bouncing_queen.rs`는 **접근 직선 반복 → 첫 벽 검사 → 두 갈래 직선 반복**이다. 원본 `interaction.rs`의 바운싱 폰 처리도 첫 벽 직전에서 반사 후 직선 진행이며 두 번째 폰 벽에서 멈춘다. 이 경로 구조를 그대로 채택했다. 재귀 반사나 별도의 반사 횟수 카운터·쿨다운은 추가하지 않았다.

- 접근 중 도달한 모든 빈칸에서 멈출 수 있다. 보드 밖 또는 바운싱 폰으로 향하면 **마지막 합법 칸**에서 분기한다. 시작 칸 바로 옆이 벽인 경우 시작 칸에서도 분기할 수 있다.
- 룩은 진행 방향과 직교하는 양방향, 비숍은 x만 반전한 방향과 y만 반전한 방향을 각각 평가한다. 보드 가장자리에서는 보드 안으로 향하는 분기만 살아남는다. 대각으로 모서리 바깥에 닿을 때 두 분기가 모두 보드 밖이면 그 경로는 끝난다. 원본에 없는 180도 되돌아가기는 추가하지 않았다.
- 바운싱 폰은 백·흑 모두 벽이다. 진입하거나 포획하지 않고 직전 칸에서 양쪽 분기를 탐색한다. 일반 아군은 차단, 일반 적은 포획 후 종료이며 벽으로 취급하지 않는다.
- 반사 후에는 직선으로만 진행한다. 두 번째 경계·바운싱 폰·아군에서 정지, 일반 적에서 포획 후 정지한다. 원점에 놓인 자기 기물도 아군 차단 규칙을 따른다.
- `enemy(0,0) not`으로 접근 중 이미 포획한 후보에서 반사를 이어 만들지 못하게 한다. `do…while`이 조건을 참으로 복구하더라도 포획 후 추가 진행은 허용하지 않는다.
- 각 직선은 유한 보드에서 한 방향으로만 전진한다. 별도의 실행 제한에 도달하지 않고 끝나며, Battle은 효과·목적지·승격이 동일한 액션만 중복 제거한다. 서로 다른 승격 선택은 유지한다.

Chesssembly에 체인 범위의 `wall-piece(key)`와 `barrier(dx,dy)` 두 식만 추가했다. 전자는 이후 `take-move`가 특정 종류 앞에서 정지하게 하고, 후자는 보드 경계 또는 설정한 종류의 점유를 검사한다. 기물 ID나 바운싱 전용 전투 엔진을 계산기에 넣지 않았다. 기존 `edge`/`corner`/`bound` 의미는 그대로다. 일반 기물은 `wall-piece`를 설정하지 않으므로 바운싱 폰도 보통 적처럼 포획한다.

## 강화·승격·지속 상태

`PieceDefinition.upgrades: &[UpgradeDefinition]`가 휴식 분기 전체를 제공한다. 보드에서 일반 폰을 선택하면 두 SVG 경로와 효과·요구 조건을 표시한다. 분기를 바꾸면 해당 결과만 강조하며, 보드와 상세 이미지도 함께 미리보기한다. 확정에서 `{type:"rest_upgrade",id,target}`를 보내고 Rust가 살아 있는 기물·현재 종류의 경로·쉼터 상태를 검증한다. 다른 유효 기물 클릭·닫기·Esc·부활 및 행동 1회 제한은 기존 방식이다.

`PieceDefinition.promotions`는 휴식 경로와 별개다. Battle은 최종 랭크로 향하는 후보를 각 승격 대상의 `ActionStep.transition`을 포함한 후보로 바꾼다. Game은 `{type:"move",from,to,promotion}`과 정확히 일치하는 후보만 실행한다. 선택 누락·다른 계열·등록되지 않은 종류·승격 아닌 이동에 승격을 붙인 요청은 거부한다. 적 AI도 같은 승격 후보를 선택하고 같은 검증으로 실행한다.

React는 Rust 스냅샷의 후보만 사용한다. 최종 랭크를 클릭하면 보드 위에 4개/3개 SVG 선택지가 나타나며 선택 즉시 이동·포획·승격을 한 액션으로 확정한다. 선택 전은 미리보기이므로 보드·행동·저장은 바뀌지 않고 Esc/취소로 돌아갈 수 있다. 확정 뒤 기존 하이라이트로 새 기물의 합법 칸을 확인한다. 이 구조에는 중간 승격 대기 턴이나 별도 페이지가 없다.

기존 액션 적용과 `sync_players`가 ID·위치·종류를 동기화한다. 종류만 바뀌고 `moved`, `upgrades`, `persistent`, 가호 ID는 유지된다. 다음 전투의 역습은 기존대로 적용되며, 사망 후 부활도 현재 종류와 효과를 유지한다. 승격 자체는 휴식 강화 이력에 새 항목을 쓰지 않는다. 가호로 포획이 취소되면 이동과 승격도 함께 취소되고 행동만 소모한다.

## 저장

WASM 저장 JSON은 버전 **2**이다. 강화 대상은 필수 `target`, 승격 대상은 이동의 `promotion`에 명시한다. 일반 이동에서는 `promotion`을 생략한다. 성공한 명령과 적 행동을 저장하고 동일 엔진으로 재생하는 방식은 유지한다.

localStorage 슬롯 `chesslayer.run.v1`은 이전 기록을 찾아 안내하기 위해 유지한다. 슬롯 이름과 JSON 스키마 버전은 다르다. 버전 1은 옆 포획 등 제거된 규칙을 포함할 수 있어 자동 이관하지 않는다. 헤더를 먼저 검사하여, 명령 파싱 실패 대신 규칙 변경과 새 원정 필요를 설명한다. 이어하기 실패는 기존 저장을 덮어쓰지 않으며 새 원정을 시작할 때만 교체된다.

## 범위와 남은 항목

두 계열의 모든 9종은 실제 원정에서 강화·승격으로 획득하고 이동할 수 있다. 일반 폰 승격, 성벽 경로, 상위 기물로의 휴식 강화, 새 적 편성, 체크메이트·캐슬링·앙파상은 이번 범위에 넣지 않았다. 적 편성에 새 종류를 배치했을 때의 행마·승격은 지원하지만 기존 전투 편성은 유지한다. 제공 이미지의 원본 라이선스는 여전히 미확인이다.

검증 목적·실행 명령·브라우저 기보는 [TESTING.md](TESTING.md), SVG 원본과 수정은 [ASSETS.md](ASSETS.md)에 기록한다.

## 변경 파일 안내

- `crates/chesslayer/src/content/pieces/`: `tempest_pawn.rs` 교정, `tempest_rook.rs`, `tempest_bishop.rs`, `tempest_knight.rs`, `tempest_queen.rs`, `bouncing_pawn.rs`, `bouncing_rook.rs`, `bouncing_bishop.rs`, `bouncing_queen.rs` 정의. `pawn.rs` 분기, `mod.rs` 등록/승격 목록, `movement.rs` 공통 반사 DSL.
- `crates/chessembly/src/lib.rs`: 체인 범위의 반사벽 식 2개.
- `crates/chesslayer/src/battle.rs`, `game.rs`: 승격 후보·정확한 선택 검증·실행·스냅샷·강화 대상.
- `crates/chesslayer-web/src/lib.rs`: 저장 버전 2와 이전 기록 안내.
- `web/src/RestPanel.tsx`, `PromotionPicker.tsx`, `main.tsx`, `types.ts`, `style.css`: 분기 미리보기와 보드 승격 선택. `PieceImage.tsx`의 기존 자동 key 매핑을 그대로 사용.
- `web/src/assets/pieces/`: 16개 SVG 추가 및 `provenance.json` 보강. 기존 템페스트 폰 SVG 2개는 유지.
- `crates/chesslayer/tests/variants.rs`, `prototype.rs`, `content.rs`, `act_map.rs`, `fixtures/`: 4개 핵심 시나리오·기존 규칙 기대값·명령 스키마·정상 원정 기보.
- `web/tests/variants.spec.ts`, `rest.spec.ts`: 실제 브라우저의 7가지 승격 선택·강화·이동·복원 및 기존 쉼터 검증.
- `README.md`, `docs/TESTING.md`, `ASSETS.md`, `CHESSEMBLY.md`, `ARCHITECTURE.md`, `content/README.md`, `content/adding-pieces.md`: 현재 규칙과 작성법. `PROTOTYPE.md`, `POLISHING.md`에는 최신 문서로의 안내를 추가.
