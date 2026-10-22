# Chesslayer

Chesslayer의 Rust 엔진 워크스페이스입니다. 저장소 루트의 `Cargo.toml`이 세 crate를 관리합니다.

| crate | 역할 |
| --- | --- |
| `chesslayer-core` | 보드, 기물 식별자, 공통 액션 효과, 룰 모듈 인터페이스 |
| `chessembly` | Chesssembly 파서와 보드를 변경하지 않는 액션 후보 생성기 |
| `chesslayer-rules` | 현재의 8×8 Chesslayer 전투 설정, 기본 기물 이동, 턴·승패·플레이어 기물 지속 상태 |

## 검증

저장소 루트에서 실행합니다.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --target wasm32-unknown-unknown
```

마지막 명령에는 `wasm32-unknown-unknown` Rust 타깃이 필요합니다. 설치 명령은 `rustup target add wasm32-unknown-unknown`입니다.

## 현재 범위

`Battle::new(BattleStart { players, encounter })`로 전투를 시작합니다. `Battle::standard_roster()`는 초기 플레이어 기물 16개를 제공합니다. `legal_actions`는 순서가 있는 이동 후보를 반환하고, `apply_piece_action`은 선택한 후보를 검증해 적용합니다. `outcome`은 다음 전투로 넘길 플레이어 기물의 상태를 반환합니다. `apply_resolved_card_effect`는 다른 시스템에서 이미 확정한 효과를 받아들이며 기물 이동 턴을 넘기지 않습니다.

기본 이동은 킹의 한 칸 이동, 룩·비숍·퀸의 직선 이동, 나이트 점프, 폰의 전진·대각 포획 및 조건을 만족하는 첫 두 칸 전진을 포함합니다. 캐슬링, 앙파상, 승격, 체크 제한과 체크메이트는 현재 Chesslayer 전투 규칙에 없습니다. 카드 비용·덱·대상 선택, 적 AI, 전투 이후 진행은 아직 정의되지 않았습니다.

- [Chesssembly 문법과 공통 액션 계약](docs/CHESSEMBLY.md)
- [Rust 엔진 구조와 전투 규칙 경계](docs/ARCHITECTURE.md)
