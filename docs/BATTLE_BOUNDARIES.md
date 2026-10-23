# 전투 시작 칸 복귀와 호신용품 — 2026-10-24

## 적용 규칙과 구현

`PlayerPiece.origin`은 고유 ID에 연결된 영구 시작 칸이다. 표준 백 진형 a1~h1/a2~h2로 생성하며 종류나 현재 좌표에서 추측하지 않는다. 원래 b1 나이트가 영구 진화해도 b1에 복귀한다. 등록된 다른 ID·시작 칸도 같은 방식으로 부활하고 전투를 시작할 수 있다. 편성 편집이나 기물 획득 UI는 추가하지 않았다.

`PlayerPiece.kind`, `upgrades`, `persistent`에는 영구 종류·강화·성장만 유지한다. 전투 승격은 실제 보드의 종류만 바꾼다. 전투 종료 판정 직후 `Game::check_finish → Battle::reset_after_battle`에서 생존한 등록 기물은 `origin`, 죽은 기물은 `position: None`으로 정리한다. 이동 이력·가호·호신용품 대상은 초기화한다. 보드는 영구 목록으로 다시 구성하므로 전투에만 추가된 기물은 남지 않는다. 영구 승격 효과는 현재 콘텐츠에 없으며, 추후 구현 시 영구 종류를 명시적으로 갱신하면 된다.

휴식 UI·실제 부활·다음 전투는 같은 목록을 사용한다. 살아 있는 기물을 선택하면 영구 진화 상태의 트리를, 죽은 기물의 시작 칸을 선택하면 확대된 부활 미리보기를 표시한다. 다른 빈칸은 반응하지 않는다. 부활은 기존대로 비킹만 가능하고 쉼터 행동 1회를 사용한다. ID·영구 성장·시작 칸을 보존하며 중복 생성하지 않는다.

호신용품은 이름·획득·분류를 유지하고 효과만 교체했다. 전투 배치 후 살아 있는 아군(킹 포함) 한 기를 런 RNG로 선택한다. 난수 인덱스는 불균등한 나머지 구간을 거부해 동일 확률로 고른다. 같은 시드·명령은 같은 대상을 재현하며, 매 전투 새로 뽑기 때문에 우연히 같은 기물이 다시 선택될 수도 있다.

Rust의 직접 기물 액션 처리에서 각 단계의 실제 포획자·피해자 ID를 확인한다. 보호 대상이 포획되면 적 포획자도 즉시 제거하고 남은 연속 이동을 중단한다. 로얄·엘리트·보스에도 적용하고 연쇄 사망이 끝난 뒤 승패를 판정한다. 양측 로얄 동시 사망은 기존대로 아군 킹 사망 우선으로 패배한다. `ResultReason::SelfDefense`로 승리 로그를 구분한다. 가호가 포획을 막으면 발동하지 않는다. 카드·환경 제거·아군 제거·위치 교환·전투 정리에도 발동하지 않는다.

전투 화면은 기존 기물 상태 표시 옆에 ◆를 표시한다. 칸의 접근성 이름·툴팁과 기물 선택 안내에서 효과를 확인할 수 있다. UI가 사망이나 승패를 계산하지 않는다. 적 배치·행마·AI·기존 카드의 로얄 보호 규칙은 유지했다.

## 저장 호환성

명령 재생 방식은 위치·승격·가호·난수 소비 순서 변경의 영향을 받는다. 저장 payload 버전을 **4**로 올렸으며 1~3은 재생 전에 변경 사유를 안내한다. localStorage 슬롯 `chesslayer.run.v1`과 기존 원본은 유지하고 사용자가 새 원정을 시작할 때만 교체한다. 자동 이관은 제공하지 않는다.

## 집중 검증의 목적과 기대 결과

추가한 Rust 시나리오는 `crates/chesslayer/tests/battle_boundaries.rs`의 4개다. 기존 테스트는 변경된 규칙의 기대값과 정상 기보만 갱신했다.

| 테스트 | 목적·실행 방법 | 기대 결과 |
| --- | --- | --- |
| `battle_boundary_rest_revive_and_next_formation_share_permanent_roster` | 영구 진화·성장, 이동, 사망, 가호, 보드에만 있는 임시 기물, 승격과 승리 포획을 함께 실행하고 보상→휴식→부활→다음 전투 진행 | 생존 기물은 시작 칸, 사망 칸은 빈칸. 영구 종류·성장은 유지, 전투 승격·이동 이력·가호·임시 기물은 제거. 표준 목록에 없는 ID도 지정한 시작 칸으로 부활. 휴식과 다음 편성 일치, 폰 초기 이동 복구 |
| `direct_capture_retaliates_against_normal_elite_boss_and_royal_after_ward` | 룩·아마존·구행 각각 일반/로얄 및 보호 대상 폰/킹 조합에서 가호 방어 후 실제 재포획 | 첫 포획은 취소, 다음 포획은 쌍방 사망. 적 로얄만 사망하면 승리, 아군 킹도 죽으면 패배. 종료 후 보호 지정 제거 |
| `self_defense_ignores_other_victims_removals_cards_and_swaps` | 자동 제거·해결된 카드 포획·위치 교환·이동·비보호 기물 직접 포획 | 호신용품 반격 없음, 생존 보호 대상 지정 유지 |
| `each_battle_selects_one_living_ally_including_king_with_seeded_replay` | 여러 시드의 실제 선택 및 동일 RNG 재생, 실제 Game에서 보호 폰 포획→반격 승리→보상→다음 전투 | 살아 있는 아군 한 기만 선택, 킹 포함·죽은 기물 제외. 재생 일치. 종료 시 표시 제거 후 다음 전투에서는 남은 킹을 새로 선택 |

브라우저에서는 기존 휴식·부활·7개 일반 기물 강화·7개 승격 선택·완주 시나리오를 유지했다. 휴식 칸을 e1/c1 등 원래 칸으로 갱신하고, 승격 직후에는 승격 종류가 보이지만 다음 전투에는 강화된 폰으로 돌아오는 것을 검사한다. `game.spec.ts`의 새 호신용품 시나리오 1개는 실제 WASM의 수호자 원정 저장을 재생해 ◆ 한 개, 대상 ID, 접근성 이름, 툴팁, 선택 설명을 확인한다. 이전 저장 버전 검사는 3도 포함한다.

기보는 임시 외부 도구에서 실제 `Game::dispatch`와 공통 AI로 생성한 정상 명령이다. 생성 도구·디버그 API는 제품에 추가하지 않았다. `winning-run.json`의 경로는 기존 `2→6→8→10→14→17→18`을 유지한다. 두 승격 기보는 휴식 10→일반전 13에서 승격·승리→일반전 15를 유지한다. `rest-revive.json`은 실제 청바지 사용으로 아군 룩을 잃고 쉼터에 도달하는 부활 UI 검증용 기록이다.

## 실행

저장소 루트에서 실행한다. Rust·npm 의존성·설치된 Chrome과 wasm-bindgen을 사용한다.

```sh
# 핵심 규칙만 빠르게 검사
cargo test -p chesslayer --test battle_boundaries

# 기존 편성·행마·카드·원정 회귀와 정적 검사
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 최신 실제 WASM 및 TypeScript/프로덕션 번들
cargo build -p chesslayer-web --target wasm32-unknown-unknown --release
/tmp/chesslayer-tools/bin/wasm-bindgen --target web --out-dir web/src/wasm \
  target/wasm32-unknown-unknown/release/chesslayer_web.wasm
npm run build --prefix web

# 실제 브라우저 전체 회귀
CHESSLAYER_CHROME=/usr/bin/google-chrome npm run test:e2e --prefix web
git diff --check
```

일반 환경에서는 WASM 생성 두 명령 대신 `npm run wasm --prefix web`을 사용할 수 있다. 브라우저 실행에는 localhost 서버와 Chrome 실행 권한이 필요하다.

## 변경 파일

- 엔진: `crates/chesslayer/src/battle.rs`, `game.rs`, `lib.rs`, `content/mod.rs`, `content/cards/ward.rs`
- 저장 어댑터: `crates/chesslayer-web/src/lib.rs`
- UI: `web/src/main.tsx`, `types.ts`, `style.css`, `RestPanel.tsx`
- Rust 검증: `crates/chesslayer/tests/battle_boundaries.rs`, `battle.rs`, `act_map.rs`, `prototype.rs`, `variants.rs`
- 정상 기보: `crates/chesslayer/tests/fixtures/winning-run.json`, `tempest-promotion.json`, `bouncing-promotion.json`, `rest-revive.json`
- 브라우저 검증: `web/tests/game.spec.ts`, `rest.spec.ts`, `variants.spec.ts`
- 문서: `README.md`, `docs/ARCHITECTURE.md`, `PROTOTYPE.md`, `VARIANTS.md`, `TESTING.md`, `BATTLE_BOUNDARIES.md`, `content/encounters.md`

## 검증 결과와 제약

Rust **47개**, Playwright **13개**(실제 Chrome 12개 + 트리 배치 1개)가 통과했다. fmt·Clippy·WASM release 빌드/바인딩·TypeScript/Vite 빌드·`git diff --check`도 통과했다. 브라우저 전체 실행은 약 1분 17초였다. `web/test-results/self-defense.png`와 `rest-revive-mobile.png`를 직접 열어 보호 표시·설명, 모바일 시작 칸·확대 미리보기·부활 문구와 버튼을 확인했다.

저장 버전 1~3의 자동 이관, 기물 획득·진형 편집 UI, 새로운 영구 승격 효과는 구현 범위에 포함하지 않았다. 현행 콘텐츠의 일반·엘리트·보스 배치/행마와 공통 AI 회귀는 통과했으며, 모든 시드·캐릭터 조합의 장기 난이도 균형은 평가하지 않았다.
