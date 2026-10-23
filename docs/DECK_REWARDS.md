# 작업 1/6 — 빈 덱·손패 유지·혼합 전투 보상

기준은 `develop`의 `fd1ee22`이며 Rust가 게임 규칙을 소유하는 기존 Rust → WASM → React 경계를 유지한다.

## 적용 규칙

- 모든 캐릭터는 영구 덱 0장과 기존 시작 유물 1개로 출발한다. 빈 덱·드로우·버림 더미에서도 이동과 턴 종료가 가능하며 카드를 자동 지급하지 않는다.
- 첫 턴을 포함해 플레이어 턴 시작마다 1장을 뽑는다. 영구 덱 크기에 제한을 두지 않고 손패 상한 10장을 유지한다. 마나·선천성 우선순위·셔플·버림 재셔플·소멸 더미는 기존 방식이다.
- 턴 종료 **처리 전 전체 손패 수**가 0~4장이면 일반 카드를 유지하고 5장 이상이면 일반 카드를 버린다. 휘발성은 항상 소멸하고 보존은 유지한다. 예를 들어 일반 3장+보존 1장+휘발성 1장은 총 5장이므로 일반 3장은 버림, 보존은 손패, 휘발성은 소멸로 이동한다.
- 일반전·엘리트·보스전 승리마다 정확히 3개를 생성하며 한 개만 선택한다. 미보유 유물이 있으면 카드 1개와 유물 1개를 먼저 추첨하고, 남은 카드·유물 후보에서 세 번째를 추첨한 뒤 슬롯 순서를 섞는다. 같은 제시 목록에는 같은 카드 종류나 유물이 중복되지 않는다. 유물을 모두 보유하면 기존 카드 후보 3종으로 채운다.
- 선택한 카드는 영구 덱, 유물은 영구 유물 목록에 추가한다. 보물과 엘리트 추가 유물은 유지하며 중복 유물은 가호 카드 1장으로 대체한다. 엘리트 선택 보상으로 추가 유물과 같은 유물을 먼저 받았을 때도 추가 유물은 가호가 된다. 모든 카드 획득은 동일한 ID 할당기를 사용한다.
- 보스도 보상 선택을 마친 뒤 `Victory`로 전환한다. 선택 전에는 원정을 완료하거나 다른 노드로 이동할 수 없다.

## 보상과 저장 재현성

`Reward::Card(CardKind)` / `Reward::Relic(Relic)`으로 구분한 실제 보상을 `Game.rewards`에 보관한다. 추첨은 기존 원정 RNG를 사용하며 지도 RNG는 기존처럼 별개다. `snapshot()`은 저장된 목록만 읽으므로 화면 갱신은 재추첨이나 RNG 소비를 일으키지 않는다.

WASM 스냅샷은 `type: "card" | "relic"`과 해당 `definition`을 보낸다. React는 카드 비용·키워드와 유물 이름·설명을 타입에 따라 표시하고 클릭한 슬롯 인덱스를 전달한다. `choose_reward.index`는 **생성된 목록의 정확한 슬롯**이다. 고정 `REWARD_POOL` 인덱스로 해석하지 않는다. 범위 밖 인덱스와 재수령은 상태 변경 없이 거부한다.

저장 payload 버전은 **4 → 5**로 올렸다. 저장 구조는 `version + character + seed + commands`를 유지한다. 저장 시점이 보상 화면이어도 동일한 승리 명령을 재생하면서 슬롯과 이후 RNG를 재현하므로 보상 필드를 별도로 저장하지 않는다. 구버전은 초기 덱·드로우·RNG 소비·보상 인덱스 의미가 달라 자동 이관하지 않는다. v1~4는 명령 재생 전에 새 원정 필요 안내를 표시하며 기존 localStorage 문자열은 새 원정을 시작하기 전까지 보존한다. 저장 키는 `chesslayer.run.v1` 그대로다.

## 변경 파일

| 파일 | 변경 |
| --- | --- |
| `crates/chesslayer/src/cards.rs` | 종료 직전 손패 수를 기준으로 일반 카드 버림 분기 |
| `crates/chesslayer/src/content/cards/mod.rs` | 초기 10장 제거, 빈 시작 덱과 카드 후보 풀 |
| `crates/chesslayer/src/content/mod.rs` | 턴당 드로우 1장, 유물 후보 목록 |
| `crates/chesslayer/src/game.rs` | 보상 타입·생성·보관·슬롯 선택, 공통 획득 처리, 보스 보상 |
| `crates/chesslayer/src/map.rs` | 일반·엘리트·보스 노드의 보상 설명 |
| `crates/chesslayer-web/src/lib.rs` | 저장 v5와 구버전 안내 |
| `web/src/types.ts`, `web/src/main.tsx`, `web/src/style.css` | 타입별 보상 UI, 시작 덱·손패 설명, 모바일 유물 표시 |
| `crates/chesslayer/tests/prototype.rs`, `act_map.rs`, `content.rs` | 기존 빈 덱·더미·획득·등록 기대값 갱신 |
| `crates/chesslayer/tests/rewards.rs` | 핵심 보상 회귀 2개 |
| `crates/chesslayer/tests/fixtures/*.json` | 완주·패배·부활·템페스트/바운싱 승격 기보 5개 갱신 |
| `web/tests/game.spec.ts`, `variants.spec.ts`, `rewards.spec.ts` | 빈 덱·슬롯 클릭·저장 버전 기대값, 혼합 보상 E2E 1개 |
| `README.md`, `docs/TESTING.md`, `docs/ARCHITECTURE.md`, `docs/PROTOTYPE.md`, `docs/ACT_MAP.md`, `docs/DECK_REWARDS.md` | 현재 규칙·검증·구조 설명 |
| `docs/content/README.md`, `adding-cards.md`, `adding-characters.md` | 빈 시작 덱과 생성된 보상 슬롯에 맞춘 확장 가이드 |

기보는 시드 2026에서 실제 `Game::dispatch`와 적 AI로 만들어진 합법 명령만 담는다. 초기 덱 셔플 제거와 새 보상 추첨이 RNG 순서를 바꿔 기존 기보를 재생할 수 없으므로 함께 갱신했다. 기보 생성용 임시 도구는 제품/저장소에 추가하지 않았다.

## 핵심 테스트와 목적

새 테스트는 Rust 2개, 브라우저 1개로 제한하고 나머지는 기존 테스트를 갱신했다.

| 테스트 | 검증 목적 |
| --- | --- |
| 기존 `initial_position_moves_ai_and_seed_replay` | 빈 초기 덱에서 실제 이동·적 행동·다음 턴·마나 회복, 자동 카드 지급 없음 |
| 기존 `piles_reshuffle_keywords_and_hand_limit` | 첫/다음 턴 1장, 0~10장 경계, 휘발성 처리 전 5장 판정, 보존·소멸·재셔플, 더미 사이 ID 누락/중복 없음, 12장 영구 덱과 손패 상한 10장 |
| 새 `mixed_slots_are_stable_seeded_and_grant_exactly_the_displayed_reward` | 32개 시드의 혼합 3칸, 스냅샷 불변, 각 슬롯 실제 획득, 잘못된 인덱스·재수령 거부, 이후 보상 RNG 재현 |
| 새 `exhausted_relic_pool_elite_duplicates_and_boss_keep_valid_rewards_and_ids` | 유물 소진 시 카드 3종, 10장을 넘는 덱과 연속 고유 ID, 보물·엘리트 중복 대체, 선택 유물과 추가 유물 충돌, 보스 보상 후 완료 |
| 새 브라우저 `혼합 보상: 표시 슬롯 획득·다음 전투 드로우·저장 재현` | 실제 WASM 명령으로 첫 승리, 명령별 저장 복원 일치, 카드/유물 각각 UI 슬롯 클릭 및 획득, 다음 전투 1장과 카드 사용, 이후 전투 RNG 재현, 데스크톱/390px 모바일 표시 |
| 기존 완주·휴식·승격·구버전 브라우저 테스트 | 보스까지 정상 완료, 추가 보상과 기존 휴식·부활·승격 유지, 저장 v1~4 거부와 원본 보존 |

빠른 규칙 검증은 저장소 루트에서 실행한다.

```sh
cargo test -p chesslayer --test prototype --test rewards --test act_map --test content
```

전체 검증 순서:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --target wasm32-unknown-unknown
npm run wasm --prefix web
npm run build --prefix web
CHESSLAYER_CHROME=/usr/bin/google-chrome npm run test:e2e --prefix web
git diff --check
```

Playwright Chromium을 설치한 환경은 `CHESSLAYER_CHROME`을 생략한다. 이 작업 환경에서는 기존에 설치된 동일 버전 `wasm-bindgen`을 사용했다. 동일한 생성 명령은 다음과 같다.

```sh
cargo build -p chesslayer-web --target wasm32-unknown-unknown --release
/tmp/chesslayer-tools/bin/wasm-bindgen --target web --out-dir web/src/wasm \
  target/wasm32-unknown-unknown/release/chesslayer_web.wasm
```

## 직접 플레이 확인

1. WASM을 빌드하고 `npm run dev --prefix web` 실행 후 새 원정을 시작한다. 첫 전투에서 전체 덱·손패·드로우가 모두 0인지 확인하고 이동 및 턴 종료 후에도 전투가 계속되는지 확인한다.
2. 승리 후 선택지가 3개인지, 카드와 유물이 구분되는지 확인한다. 보상 화면에서 새로고침하고 이어하기를 눌렀을 때 종류·순서가 같은지 확인한다.
3. 카드 선택 후 지도 덱 수가 1 증가하는지, 유물 선택 후 유물 수가 1 증가하는지 확인한다. 다음 전투에서 카드가 있으면 첫 손패가 1장인지 확인한다. 일반 카드를 쓰지 않고 턴을 끝내면 4장 이하 손패는 유지된다.
4. 일반·보존·휘발성이 섞인 4/5장 경계와 유물 풀 소진은 위 Rust 집중 테스트로 바로 재현한다. 테스트용 상태 편집 UI는 없다.

## 결과와 남은 범위

2026-10-24 검증: Rust **49개 통과**. fmt, Clippy(`-D warnings`), wasm32 검사, release WASM 생성, TypeScript/Vite 빌드, `git diff --check` 통과. 실제 Chrome 전체 **14개 중 13개가 처음 통과**했고 RNG 변경으로 패배까지 이어지지 않던 기보를 갱신한 뒤 해당 패배·보스 완주·혼합 보상 **3개를 재실행해 모두 통과**했다. 전체 14개 시나리오가 검증되었다. 데스크톱과 390px 모바일 보상 스크린샷도 직접 확인했다. 새 라이브러리·범용 프레임워크·디버깅 UI는 추가하지 않았다.

구버전 저장의 자동 이관은 제공하지 않는다. 모든 시드·캐릭터의 장기 난이도와 카드/유물 보상 확률의 밸런스 평가는 이번 작업 범위 밖이다. 찰나의 가호는 기존과 같이 보상 후보 풀에 없으므로 초기 덱 제거 후 정상 원정 획득 경로가 없지만, 정의와 휘발성 처리·테스트는 유지한다.
