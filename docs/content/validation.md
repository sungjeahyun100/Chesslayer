# 콘텐츠 분리 검증 — 2026-10-23

기존 테스트와 빌드 절차를 유지했다. 새 프레임워크나 런타임 디버그 기능은 없다. 저장소에 추가한 테스트는 아래 통합 테스트 한 개다.

## 추가 테스트의 목적과 필요성

- 파일: `crates/chesslayer/tests/content.rs`
- 이름: `catalog_entries_are_unique_and_connected`
- 목적: 수동 등록 시 발생할 수 있는 중복 ID, 잘못된 정의 연결, 미사용 기물의 DSL 오류를 조기에 확인한다.
- 필요한 이유: 기존 플레이 시나리오는 특정 덱·편성·캐릭터를 사용한다. 새 항목이 목록에 잘못 연결되거나 기존 기물 ID를 덮어써도 그 항목을 사용하지 않는 시나리오는 통과할 수 있다.

| 대상 | 검사 | 예상 결과 |
| --- | --- | --- |
| `pieces::ALL` | ID 0 금지, ID·DSL key의 유일성/비어 있지 않음, 실제 적 편성 초기화, 보드 카탈로그·표시 이름 조회, 적의 행마 실행 | 모든 등록 기물이 기존 Battle/Chesssembly 경로로 처리됨 |
| 시작 덱·보상 카드 | `kind.definition().kind == kind` | 등록 분기가 다른 카드 정의를 반환하지 않음 |
| `characters::ALL` | ID 유일성/비어 있지 않음, `Game::new(id, 2026)` 실행, 이름과 시작 유물 확인 | 모든 등록 캐릭터가 정상 초기화됨 |

수행 방법:

```sh
cargo test -p chesslayer --test content
```

예상 출력은 `catalog_entries_are_unique_and_connected ... ok`, `1 passed; 0 failed`와 종료 코드 0이다. 새 콘텐츠의 구체적인 효과나 기획상 올바른 이동 범위를 모두 보장하는 테스트는 아니다. 새 규칙 자체는 각 추가 가이드의 확인 절차를 따른다.

## 기존 동작 비교

변경 전 `cargo test --workspace`에서 기존 32개가 통과했다. 분리 전 임시 비교 프로그램으로 다음 출력을 보관하고 분리 후 같은 입력으로 대조했다.

- 기존 캐릭터 3종 및 카드 4종의 직렬화된 정의.
- 기존 기물 8종 × 양측 × 중앙/초기 랭크 × 이동 이력 설정의 64가지 조합에서 `legal_actions()`의 전체 후보와 순서.
- 캐릭터별 새 런 스냅샷과 기존 `winning-run.json` 명령·각 적 행동 직후의 전체 스냅샷. 기존 필드만 비교하도록 새 표시 필드 `glyph`, `badge`를 제외했다.

결과는 모두 일치했다. 이 일회성 비교 프로그램은 제품 코드나 디버그 인터페이스로 남기지 않았다. 지속적인 회귀 확인은 기존 Rust 테스트, 명령 기보, 브라우저 E2E가 담당한다.

## 문서 예제 검증

`adding-cards.md`, `adding-pieces.md`, `adding-characters.md`의 Rust 코드 블록을 임시 소스 복사본에 적용했다. 실제 파일을 생성하고 안내된 모듈·ID/enum·목록 등록 및 적 편성 코드를 추가해 오프라인 Cargo 테스트로 확인했다.

- `SteadyWard`: 정의·등록·보상 노출, 보존, 사용 후 가호/마나 1 소비/행동 수 유지.
- 같은 카드의 아군 대상 제한 변형: 문서의 함수로 교체 후 컴파일·사용 확인.
- `Dancer`: ID 9 등록, 적 편성 생성, 스냅샷 이름 카탈로그 연결과 기호·배지 출력.
- `Tactician`: 선택 ID로 런 생성, 시작 유물 `Haste` 연결.

예시 콘텐츠와 예시 전용 실행 코드는 실제 게임 콘텐츠나 상시 테스트에 추가하지 않았다. 문서의 예제는 새 콘텐츠를 만드는 템플릿이다.

## 실행 결과

| 검사 | 결과 |
| --- | --- |
| `cargo fmt --all --check` | 통과 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 통과 |
| `cargo test --workspace` | 33개 통과: 기존 32 + 등록 연결 1 |
| `cargo check --workspace --target wasm32-unknown-unknown` | 통과 |
| `npm run wasm --prefix web` | release WASM 생성 성공 |
| `npm run build --prefix web` | TypeScript 검사·프로덕션 번들 통과 |
| `CHESSLAYER_CHROME=/usr/bin/google-chrome npm run test:e2e --prefix web` | 기존 브라우저 시나리오 5개 통과 (27.1초) |
| 문서 예제 | 기본/대상 제한 카드, 기물, 캐릭터의 등록·컴파일·실행 통과 |

WASM 빌드는 기존 `/tmp/chesslayer-tools/bin`의 wasm-bindgen을 PATH에 추가해 실행했다. E2E의 첫 시도는 샌드박스의 `127.0.0.1:5173` 바인딩 제한(`EPERM`)으로 서버가 시작되지 않았다. localhost·Chrome 실행 권한으로 다시 실행하여 모두 통과했다. 일반 개발 환경의 전체 준비 절차는 [기존 테스트 안내](../TESTING.md)를 참고한다.
