# 행마 구조 수정 검증

실행 명령: `cargo test --workspace --no-fail-fast`

| 검증 | 방법 | 기대 결과 |
| --- | --- | --- |
| 반사벽 진영 판정 | `chessembly/tests/commands.rs`, `chesslayer/tests/variants.rs`, `new_movements.rs` | 아군 바운싱 폰 앞에서 반사하고 해당 칸에 진입하지 않는다. 적군 바운싱 폰은 포획하고 경로가 종료된다. `barrier`도 아군 벽에서만 참이다. |
| 추기경 반복 반사·순환 | `new_movements.rs`의 두 반사 도착과 양측 64칸 시작 위치 검사 | 두 번째 반사 뒤 후보를 만들고 순환 없이 종료한다. 생성한 액션은 적용 가능하다. |
| DSL과 정의 독립성 | `content.rs`의 전체 등록 기물 검사와 소스 참조 검사 | 추기경 외 모든 기물 식이 파싱되고 전투에서 합법 수를 생성한다. 다른 기물의 `movement()`를 호출하지 않는다. |
| 명령 제거 | `chessembly/tests/commands.rs`의 파서 검사 | `reflect-ray`는 알 수 없는 식으로 거부된다. |
| 전투·강화·승격·미리보기 회귀 | 전체 Rust 워크스페이스 테스트 중 `battle`, `composite_pieces`, `variants`, `new_movements`, `content` 등 | 기존 액션 검증, 포획, 강화·승격, 적 행마 미리보기 테스트가 통과한다. |

추기경의 DSL 식은 비어 있으며 `PieceDefinition.native_movement`가 [`cardinal.rs`](../crates/chesslayer/src/content/pieces/cardinal.rs)의 계산기를 지정한다. 다른 기물의 DSL 식은 각 기물 파일의 `movement()`에서 확인한다.

최종 생성 DSL 확인: `(pieces::definition(kind).unwrap().movement)()`를 호출해 반환된 문자열을 출력한다. 추기경은 이 문자열이 비어 있고 `native_movement`가 설정되어 있다.

## 수정 파일

- `crates/chessembly/src/lib.rs`
- `crates/chessembly/tests/commands.rs`
- `crates/chesslayer/src/battle.rs`
- `crates/chesslayer/src/content/pieces/bouncing_bishop.rs`
- `crates/chesslayer/src/content/pieces/bouncing_knightrider.rs`
- `crates/chesslayer/src/content/pieces/bouncing_pawn.rs`
- `crates/chesslayer/src/content/pieces/bouncing_queen.rs`
- `crates/chesslayer/src/content/pieces/bouncing_rook.rs`
- `crates/chesslayer/src/content/pieces/bouncing_tempest_bishop.rs`
- `crates/chesslayer/src/content/pieces/bouncing_tempest_queen.rs`
- `crates/chesslayer/src/content/pieces/bouncing_tempest_rook.rs`
- `crates/chesslayer/src/content/pieces/cardinal.rs`
- `crates/chesslayer/src/content/pieces/mod.rs`
- `crates/chesslayer/src/content/pieces/movement.rs`
- `crates/chesslayer/src/content/pieces/movement_god.rs`
- `crates/chesslayer/src/content/pieces/pawn.rs`
- `crates/chesslayer/src/content/pieces/tempest_bishop.rs`
- `crates/chesslayer/src/content/pieces/tempest_queen.rs`
- `crates/chesslayer/src/content/pieces/tempest_rook.rs`
- `crates/chesslayer/src/content/pieces/typhoon_bishop.rs`
- `crates/chesslayer/src/content/pieces/typhoon_knight.rs`
- `crates/chesslayer/src/content/pieces/typhoon_queen.rs`
- `crates/chesslayer/src/content/pieces/typhoon_rook.rs`
- `crates/chesslayer/tests/composite_pieces.rs`
- `crates/chesslayer/tests/content.rs`
- `crates/chesslayer/tests/new_movements.rs`
- `crates/chesslayer/tests/variants.rs`
- `docs/ADVANCED_MOVEMENT.md`
- `docs/CHESSEMBLY.md`
- `docs/COMPOSITE_PIECES.md`
- `docs/TESTING.md`
- `docs/VARIANTS.md`
- `docs/content/README.md`
- `docs/content/adding-pieces.md`
- `docs/MOVEMENT_REFACTOR_TESTS.md`
