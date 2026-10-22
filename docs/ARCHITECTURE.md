# 프로토타입 구조

## 통합 결정

기존 `chesslayer-core`와 `chesslayer-rules` 패키지를 없애고 게임 상태와 규칙을 `chesslayer` 하나로 통합했습니다. `RuleModule`, 범용 `GameState`, 사용하지 않는 `GameAction`/`Move` 래퍼와 해당 인터페이스만 검증하던 테스트 1개는 제거했습니다.

Chesssembly와 게임 엔진이 서로 의존하는 순환을 피하기 위해, 기존 `Board`, `Piece`, `Position`, `DefaultAction`, 액션 적용 구현은 **코드를 그대로 이동해** `chessembly`가 소유합니다. Chesssembly에 카드나 전투 규칙을 넣지 않았습니다. 게임 crate는 이 자료형을 재노출합니다. 보드 마스크, 비표준 크기, DSL 문법과 기존 액션 스냅샷 테스트는 유지합니다.

```text
React UI
  ↓ JSON 의도 / ↑ 표시용 스냅샷
chesslayer-web (WebGame: WASM 객체 하나가 Game 하나를 소유)
  ↓
chesslayer
  battle.rs   기존 Battle 기반 이동·턴·가호·로얄 보호·역습·outcome
  cards.rs    카드 정의, 영구 덱과 별개인 전투 더미
  content.rs  몬스터, 캐릭터, 임시 규칙 상수, 결정적 RNG
  map.rs      한 번 생성하는 제1막 그래프, 노드 콘텐츠와 표시 자료형
  game.rs     명령 검증, AI, 보상, 휴식, 원정 상태와 UI 스냅샷
  ↓
chessembly (보드, 액션, 파서와 이동 후보 계산)
```

## 한 번의 명령

1. UI는 `move`, `play_card`, `end_turn`, `enemy_step`, `choose_reward`, `select_node`, `claim_treasure`, 휴식 명령 중 하나를 보냅니다.
2. Rust가 현재 단계·연결된 다음 노드·진영·합법 수·손패·비용·대상·진형을 검증합니다. 프론트엔드의 disabled 표시는 편의일 뿐입니다.
3. 기존 `Battle::legal_actions`와 `apply_piece_action`으로 행동합니다. 카드 비용은 효과 검증과 실행이 성공한 뒤 차감합니다.
4. Rust가 새 스냅샷을 반환합니다. 이동할 수 있는 칸과 카드 대상 ID도 여기서 전달합니다. TypeScript에는 체스 행마나 승패 판정이 없습니다.
5. 적 턴에서는 UI가 애니메이션을 위한 650ms 간격으로 `enemy_step`만 요청합니다. 어떤 수를 둘지는 Rust가 정합니다. 전투 종료 후 명령과 잘못된 진영의 명령은 Rust가 거부합니다.

## 맵과 진행 상태

`Game::new()`에서 `ActMap::generate(seed)`를 한 번 호출합니다. 노드 유형·전투 편성·유물·연결 목록은 원정 중 바꾸지 않습니다. 지도 RNG는 전투/덱 RNG와 별도 스트림을 사용합니다. 브라우저는 새 런마다 난수 시드 하나를 전달하며 지도 생성과 이동 허가는 Rust가 담당합니다.

`Game.current_node`, `path`, `completed`가 탐험 상태입니다. `available_nodes()`는 맵 단계에서만 시작점 또는 완료된 현재 노드의 직접 후속 노드를 반환합니다. 잘못된 선택은 상태 변경 전에 거부합니다. UI는 스냅샷의 `available/visited/current/skipped`를 표시하며 별도의 진행 규칙을 계산하지 않습니다.

```text
새 런 → Map → select_node
  일반/엘리트 → Combat → Reward → choose_reward → Map
  휴식 → Rest → 강화/부활/건너뛰기 → Map
  보물 → Treasure → claim_treasure → Map
  보스 → Combat → Victory
  전투 중 킹 사망 → Defeat
```

`web/src/ActMap.tsx`가 SVG 연결선과 HTML 노드 버튼을 그립니다. 전투 중 지도 열기는 보기 전환일 뿐이며 현재 콘텐츠를 건너뛸 수 없습니다. 기존 직선 경로 상수와 `continue` 명령, 전투 사이 자동 순번 증가를 제거했습니다. 구체적인 생성 방식과 임시 콘텐츠는 [ACT_MAP.md](ACT_MAP.md)에 있습니다.

## 상태 계승

`Battle::outcome()`의 플레이어 목록은 죽은 기물도 `position: None`으로 보존합니다. 새 전투는 같은 목록으로 시작하며, 기존 역습 배치 절차를 사용합니다. 영구 덱은 `Game.deck`에, 전투 중 상태는 `CardCombat`에 분리합니다. 종료 시 `CardCombat`을 비우므로 소멸한 카드도 다음 전투에서는 영구 덱에서 다시 만들어집니다.

카드와 유물의 제거는 `remove_nonroyals()`를 통하며 양측 로얄을 보호합니다. 기존 `apply_resolved_card_effect()`에도 결과 보드를 검사하는 원자적 보호를 넣었습니다. 로얄을 없애는 효과라면 보드를 적용하지 않습니다. 직접 기물 포획은 `RoyalCaptured`, 시작 배치의 킹 역습 사망은 `Counterattack`으로 구분합니다.

AI는 기존 합법 후보에서 킹 포획을 최우선으로, 그다음 포획할 기물의 가치로 점수를 매깁니다. 동점은 시드 RNG로 선택합니다. 학습·검색 트리·외부 서비스가 없습니다. 합법 수가 없으면 승리 판정 없이 턴을 넘깁니다.

## 로컬의 기존 벤치마크

작업 시작 전 미추적 상태였던 `README-BENCH.md`, `bench-results/`, `scripts/`, 두 crate의 `examples/`는 보존했습니다. 과거 crate 이름에 의존하는 이 로컬 자료는 통합 프로토타입 빌드에 포함하지 않습니다(`chessembly`의 `autoexamples = false`). 정식 검증은 [TESTING.md](TESTING.md)를 따릅니다. 과거 벤치마크를 다시 쓰려면 새 import/패키지 경로로 별도 이관해야 합니다.
