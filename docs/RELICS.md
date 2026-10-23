# 작업 2/6 — 신규 유물

기준은 원격 `develop`의 `fd1ee22`와 작업 1/6의 빈 덱·혼합 보상 변경이다. 유물 정의는 `crates/chesslayer/src/content/relics.rs`, 원정 획득·배치·스냅샷은 `game.rs`, 합법 수·포획·전투 종료는 `battle.rs`가 담당한다. React는 Rust가 반환한 `RelicView.id/name/description/status`, `mana_rule`, 코스트와 합법 수만 표시한다.

## 정의와 획득

| 이름 | 직렬화 식별자 | 효과 |
| --- | --- | --- |
| 인디언 킹의 전략책 | `kings_indian` | 다음 전투 시작 시 KIA 프리셋 |
| 인디언 퀸의 전략책 | `queens_indian` | 다음 전투 시작 시 백 방향 QID 프리셋 |
| 고슴도치의 지혜 | `hedgehog` | 다음 전투 시작 시 백 방향 Hedgehog 프리셋 |
| 선공권 | `initiative` | 적 턴 10회 동안 적의 일반 포획 금지. 아군의 일반 포획 성공 시 즉시 해제 |
| 아이스크림 | `ice_cream` | 플레이어 턴 시작마다 남은 코스트 +3 |
| 하드디스크 | `hard_disk` | 직전 전투의 생존 아군 종료 위치 유지. 겹친 적 제거 |

기존 `haste`, `whirlwind`, `self_defense`를 포함한 9종을 `RELIC_REWARD_POOL`에 명시적으로 등록했다. 일반·엘리트·보스의 혼합 선택 보상, 보물, 엘리트 추가 유물이 같은 후보 풀을 사용한다. 혼합 보상은 미보유 유물만 추첨한다. 보물/엘리트에서 중복 획득하면 기존처럼 가호 카드로 대체한다.

`Game.relics`는 시작 유물을 포함한 **최초 획득 순서**다. 추가 획득 때 뒤에 붙이고 기존 유물을 삭제하거나 재정렬하지 않는다. 시작 진형 유물은 뒤에서 처음 발견한 하나만 활성이다. 중복 유물은 새 획득으로 기록하지 않으므로 기존 진형을 다시 활성화하지 않는다. 엘리트에서 선택 보상을 먼저 받고 추가 유물을 나중에 받으므로 추가 유물이 새 진형이면 그쪽이 우선한다.

## 시작 진형 좌표

아래는 **Chesslayer의 게임용 프리셋**이다. KIA, QID, Hedgehog는 각각 하나의 고정 FEN으로 정의되는 체계가 아니며, 아래 좌표를 체스의 유일한 공식 배치로 간주하지 않는다. 교환 없이 원래 16개 ID를 모두 배치할 수 있도록 구성했다.

엔진 좌표는 `a8=(0,0)`, `h1=(7,7)`, 화면 칸 번호는 `rank*8+file`이다. 아래 칸은 `KIA`, `QID`, `HEDGEHOG` 상수에 명시적인 좌표 쌍으로 저장한다. QID와 Hedgehog는 흑 구조를 **파일은 유지하고 랭크만 반전**해 백 방향으로 구성했다. 대응하는 흑 좌표는 `(file, 7-rank)`이며, 예를 들어 흑 b7 비숍/b6 폰은 백 b2/b3이 된다.

| 영구 ID | 원래 기물 | 영구 origin | KIA | QID | Hedgehog |
| --- | --- | --- | --- | --- | --- |
| 1 | 퀸사이드 룩 | a1 | a1 | a1 | c1 |
| 2 | 퀸사이드 나이트 | b1 | d2 | d2 | d2 |
| 3 | 퀸사이드 비숍 | c1 | c1 | b2 | b2 |
| 4 | 퀸 | d1 | e2 | d1 | c2 |
| 5 | 킹 | e1 | g1 | g1 | g1 |
| 6 | 킹사이드 비숍 | f1 | g2 | e2 | e2 |
| 7 | 킹사이드 나이트 | g1 | f3 | f3 | f3 |
| 8 | 킹사이드 룩 | h1 | f1 | f1 | e1 |
| 9 | a폰 | a2 | a2 | a2 | a3 |
| 10 | b폰 | b2 | b2 | b3 | b3 |
| 11 | c폰 | c2 | c2 | c2 | c3 |
| 12 | d폰 | d2 | d3 | d4 | d3 |
| 13 | e폰 | e2 | e4 | e3 | e3 |
| 14 | f폰 | f2 | f2 | f2 | f2 |
| 15 | g폰 | g2 | g3 | g2 | g2 |
| 16 | h폰 | h2 | h2 | h2 | h2 |

- KIA: g2 비숍/g3 폰, f3·d2 나이트, d3·e4 중심 구조와 g1 킹을 사용한다.
- QID: b2 비숍/b3 폰, e3·d4 중심 구조를 사용한다.
- Hedgehog: a3·b3·d3·e3 폰, b2·e2 비숍, d2·f3 나이트를 사용한다. 교환된 c폰을 전제로 하지 않도록 c폰도 c3에 보존한다.

배치는 **원래 ID**에 대응한다. 예를 들어 ID 2가 템페스트 나이트로 강화돼도 d2에 배치한다. ID·영구 종류·`origin`·강화 이력·성장은 변경하지 않는다. `position: None`인 사망 기물은 건너뛰며 전투 중 획득 처리로 보드를 다시 배치하지 않는다.

## 전투 경계와 하드디스크

전투 시작 우선순위:

1. 하드디스크를 보유하고 직전 전투에서 살아남은 현재 ID에 대한 유효한 위치 기록이 있으면 복원한다.
2. 그렇지 않으면 가장 최근에 획득한 시작 진형을 적용한다.
3. 둘 다 없으면 각 기물의 영구 `origin`을 사용한다.

`last_battle_positions`는 ID→위치만 보관한다. **유물 보유 여부와 관계없이**, 승패 확정 직후 보드를 정리하기 전에 매 전투 종료 위치를 수집한다. 따라서 승리 보상에서 처음 하드디스크를 받아도 방금 종료한 실제 배치가 남는다. 스냅샷을 만들거나 유물을 획득하는 행위는 RNG를 소비하지 않는다.

하드디스크 보유 시 전투 종료 정리에서도 생존 위치를 유지한다. 미보유 시 기존처럼 `origin`으로 복귀하되 별도 종료 기록은 남긴다. 두 경우 모두 전투 중 승격 종류·가호·이동 이력·호신용품·선공권·코스트를 초기화한다. 영구 강화 종류와 이력은 유지하며 임시로 보드에만 존재하는 기물은 다음 전투에 생성하지 않는다.

복원은 현재 살아 있는 ID만 대상으로 한다. 위치의 보드 범위와 중복을 검사하며 유효한 생존 기록이 없으면 진형/표준 배치로 돌아간다. 휴식에서 부활했거나 새로 추가되어 기록에 없는 ID는 `origin`을 사용한다. 기록된 생존 기물이 그 칸을 점유했다면 기록을 우선하고 새 기물은 `a1..h1, a2..h2, …` 순서의 첫 빈 칸에 배치한다. 유효한 아군을 삭제하거나 밀어내지 않는다. 일반 프리셋에 없는 추가 ID도 같은 빈 칸 정책을 사용한다.

휴식은 계속 각 ID의 영구 `origin`을 사용한다. 그 칸이 실제로 점유됐으면 기존 부활 거부 규칙을 유지한다. 처음 하드디스크를 받은 시점에 보드는 이미 표준 배치로 복귀했을 수 있지만 종료 기록은 별개로 유지되므로 이후 휴식/강화가 기록을 덮어쓰지 않는다.

소용돌이가 적 시작 칸을 먼저 섞은 뒤 아군을 배치한다. 유효한 하드디스크 복원이 적용된 전투에서는 겹친 적을 로얄까지 제거하고 아군을 보존한다. ID/좌표 중복 등 적 편성 유효성 검사는 제거 전에도 수행한다. 로얄이 제거되면 `ResultReason::HardDisk`로 정상 승리·보상 처리한다. 즉시 끝난 전투는 AI가 행동할 수 없고, 진행 중인 전투는 충돌 처리가 끝난 같은 보드에서 호신용품 대상과 AI 합법 수를 계산한다. 하드디스크 기록이 없는 첫 전투에는 기존 역습 규칙을 사용한다.

## 선공권과 코스트

선공권은 전투 시작마다 10회로 설정하고 **적 턴 종료 시** 1회 차감한다. 적에게 합법 수가 없어 패스해도 한 턴이다. 가속의 두 이동은 한 턴으로 계산하며 열 번째 적 턴의 마지막 행동까지 보호한다. `Battle::legal_actions`는 각 액션 단계의 중간 보드를 확인해 아군을 포획하는 적 후보를 제외한다. 직접 행동 제출도 동일 검증을 통과해야 하므로 AI와 UI 모두 금지된 포획을 실행할 수 없다.

아군의 `PieceMove`가 실제로 적을 포획하면 즉시 0으로 바꾼다. 가호는 액션 전체를 취소하므로 해제하지 않는다. `remove_nonroyals`와 `ResolvedCardEffect`는 일반 포획으로 세지 않는다. 호신용품은 보호 기간 중 포획될 일이 없으며 선공권 종료 후 기존대로 동작한다. 전투 종료 시 카운터는 0, 다음 전투 시작 시 다시 10이다. Rust가 보호 중/종료/다음 전투 대기 문구와 남은 적 턴 수를 생성한다.

아이스크림은 첫 턴 코스트 3, 이후 실제 플레이어 턴 시작에서만 현재 코스트에 3을 더한다. 적 행동·카드 사용·가속의 첫 이동에서는 증가하지 않는다. 예: 3에서 1 사용→2, 다음 플레이어 턴→5, 미사용 후 다음 턴→8. 미보유 시 매 턴 3으로 복구한다. 코스트 잔액·정의 비용·스냅샷을 `u32`로 통일했다. 누적은 `saturating_add`로 4,294,967,295에서 포화하여 오버플로/래핑을 방지하며 JS number에서도 정확하다. 라운드 카운터도 포화 증가한다. 비용 검증은 현재 잔액만 비교하므로 6 이상 비용도 지원한다. 현재 등록 카드 비용은 0~3이며 6코스트 정의를 공통 카드 해결·결제 경로에 넣는 집중 테스트로 확장 가능성을 검증한다.

## 저장과 재현성

저장 버전은 **5→6**이다. `version + character + seed + commands` 구조와 `chesslayer.run.v1` 키를 유지한다. 유물 목록 순서·직전 종료 위치·선공권 카운터·누적 코스트는 명령을 재생해 동일하게 복원한다. 화면 조회로 상태/RNG를 바꾸지 않는다. 새 후보 풀 때문에 같은 시드도 v5와 보상·보물 결과가 달라질 수 있어 v1~5를 재생 전에 거부하고 원본 저장 문자열을 보존한다. 자동 이관은 제공하지 않는다.

## 검증 방법

| 검증 | 실행/관찰 |
| --- | --- |
| KIA/QID/Hedgehog | `formations_use_original_ids_keep_all_relics_and_apply_only_at_start`: 16칸 표, 최신 순서, 중복 대체, 죽은 ID, 영구 강화·origin 유지, 기록 없는 하드디스크 fallback |
| 선공권 | `initiative_filters_engine_and_ai_counts_enemy_turns_including_haste_and_passes`: 직접 불법 포획 제출 거부, AI 생존, 적 9/10/11턴 경계, 가속 2회, 무행동 패스, UI 카운터 |
| 포획 분류 | `initiative_ends_only_on_successful_piece_capture_not_ward_or_card_removal`: 가호 취소 후 유지→실제 포획 후 해제, 카드 제거와 해결된 카드 액션 유지 |
| 아이스크림 | `ice_cream_accumulates_only_on_player_turns_and_saturates_safely`: 3→2→5→8, 미보유 3 복구, 적 턴 미증가, 전투 경계, u32 상한 |
| 고비용 카드 | `six_cost_card_uses_the_same_resolution_and_payment_path`: 5코스트에서 6비용 거부·무변경, 6에서 실제 가호 적용 후 0 |
| 하드디스크 첫 획득 | `hard_disk_captures_before_first_acquisition_and_overrides_formations_and_enemy_royal`: 영구 폰/일시 승격 분리, 보상 전 스냅샷, 진형 우선순위, 일반 적·로얄 충돌, 즉시 보상과 AI 거부 |
| 휴식·기존 유물 | `hard_disk_rest_revival_missing_ids_and_shuffled_collisions_stay_consistent`: 점유 origin 부활 거부, 기록 없는 부활 ID·강화, 소용돌이 후 충돌, 호신용품 대상·AI 상태, 다음 전투 선공권 재설정 |
| 실제 획득 풀 | `every_new_relic_is_in_seeded_combat_and_treasure_pools`: 64개 시드의 전투 보상·보물/엘리트 풀에서 신규 6종 확인 |
| 브라우저 | `web/tests/relics.spec.ts`: 실제 명령 기보로 여섯 유물 획득, 명령마다 WASM 저장 복원 일치, 백 진형·origin 분리, 하드디스크 위치 복원, 선공권 10→9 표시와 코스트 3→6, 390px 모바일 가로 넘침 검사 |

```sh
cargo test -p chesslayer --test relics --lib
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --target wasm32-unknown-unknown
npm run wasm --prefix web
npm run build --prefix web
CHESSLAYER_CHROME=/usr/bin/google-chrome npm run test:e2e --prefix web
git diff --check
```

이 환경에서는 기존 설치된 도구로 WASM을 생성했다. `cargo build -p chesslayer-web --target wasm32-unknown-unknown --release` 후 `/tmp/chesslayer-tools/bin/wasm-bindgen --target web --out-dir web/src/wasm target/wasm32-unknown-unknown/release/chesslayer_web.wasm`을 실행했다. Playwright는 localhost 서버와 Chrome 실행 권한이 필요하다.

`fixtures/relic-runs.json`의 6개 시나리오는 모두 `vanguard`로 실제 전투를 승리한 뒤 혼합 보상에서 해당 유물을 선택한다. 시드는 QID/Hedgehog=0, 선공권=1, 아이스크림=2, 하드디스크=5, KIA=6이다. 기존 완주·부활·승격 기보도 새 풀에서 생성한 합법 명령으로 갱신했다. 사용자용 상태 주입이나 디버그 UI는 추가하지 않았다.

## 실행 결과

2026-10-24 기준 Rust 전체 **57개 통과**. fmt, Clippy(`-D warnings`), wasm32 검사, release WASM/바인딩 생성, TypeScript/Vite 빌드, `git diff --check` 통과. 기존 E2E 14개와 신규 유물 E2E 1개를 합친 **15개 시나리오를 검증**했다. 전체 실행에서 14개가 통과했고, 완주 기보의 실제 승격 종류를 UI에서 선택하도록 테스트를 수정한 뒤 해당 시나리오를 재실행해 통과했다. 하드디스크 첫 획득 후 부활 origin 충돌 사례도 추가한 뒤 집중 테스트·Clippy를 재검증했다.

`web/test-results/relic-initiative-mobile.png`와 `relic-kia-desktop.png`를 직접 열어 390px 보호 상태·남은 적 턴 표시와 KIA의 g2/g3/f3/d2 배치를 확인했다. 별도 라이브러리나 사용자용 디버그 API는 추가하지 않았다. 구버전 자동 이관과 모든 시드의 장기 밸런스 평가는 범위 밖이다.
