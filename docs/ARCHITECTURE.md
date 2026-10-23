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
  cards.rs    영구 덱과 별개인 전투 더미, 마나·키워드 처리
  content/    카드·기물·캐릭터별 정의, 명시적 등록 목록
    mod.rs    콘텐츠 재노출, 임시 규칙 상수, 결정적 RNG
    relics.rs 유물 식별자·설명·후보 풀·ID별 명시적 진형 좌표
  map.rs      한 번 생성하는 제1막 그래프, 노드 콘텐츠와 표시 자료형
  game.rs     명령 검증, AI, 보상, 휴식, 원정 상태와 UI 스냅샷
  ↓
chessembly (보드, 액션, 파서와 이동 후보 계산)
```

카드별 효과·대상 검증은 `content/cards/`, 기물의 ID·이름·행마·AI 가치·표시는 `content/pieces/`, 캐릭터 기본 정보·시작 유물은 `content/characters/`에 있다. 각 디렉터리의 `mod.rs`에서 명시적으로 등록한다. 카드 효과는 공통 `Battle` 메서드를 호출하고, 기물 행마는 기존 Chesssembly 계산기를 사용한다. [콘텐츠 추가 가이드](content/README.md)에 전체 파일 구조와 복사 가능한 예제가 있다.

## 한 번의 명령

1. UI는 `move`, `play_card`, `end_turn`, `enemy_step`, `choose_reward`, `select_node`, `claim_treasure`, 휴식 명령 중 하나를 보냅니다.
2. Rust가 현재 단계·연결된 다음 노드·진영·합법 수·손패·비용·대상·진형을 검증합니다. 프론트엔드의 disabled 표시는 편의일 뿐입니다.
3. 기존 `Battle::legal_actions`와 `apply_piece_action`으로 행동합니다. 카드 비용은 효과 검증과 실행이 성공한 뒤 차감합니다.
4. Rust가 새 스냅샷을 반환합니다. 이동할 수 있는 칸과 카드 대상 ID도 여기서 전달합니다. TypeScript에는 체스 행마나 승패 판정이 없습니다.
5. 적 턴에서는 UI가 애니메이션을 위한 650ms 간격으로 `enemy_step`만 요청합니다. 어떤 수를 둘지는 Rust가 정합니다. 전투 종료 후 명령과 잘못된 진영의 명령은 Rust가 거부합니다.

## 맵과 진행 상태

`Game::new()`에서 `ActMap::generate(seed)`를 한 번 호출합니다. 노드 유형·전투 편성·유물·연결 목록은 원정 중 바꾸지 않습니다. 지도 RNG는 전투/덱/보상 RNG와 별도 스트림을 사용합니다. 브라우저는 새 런마다 난수 시드 하나를 전달하며 지도 생성과 이동 허가는 Rust가 담당합니다.

`Game.current_node`, `path`, `completed`가 탐험 상태입니다. `available_nodes()`는 맵 단계에서만 시작점 또는 완료된 현재 노드의 직접 후속 노드를 반환합니다. 잘못된 선택은 상태 변경 전에 거부합니다. UI는 스냅샷의 `available/visited/current/skipped`를 표시하며 별도의 진행 규칙을 계산하지 않습니다.

```text
새 런 → Map → select_node
  일반/엘리트 → Combat → Reward → choose_reward → Map
  휴식 → Rest → 강화/부활/건너뛰기 → Map
  보물 → Treasure → claim_treasure → Map
  보스 → Combat → Reward → choose_reward → Victory
  전투 중 킹 사망 → Defeat
```

`web/src/ActMap.tsx`가 SVG 연결선과 HTML 노드 버튼을 그립니다. 전투 중 지도 열기는 보기 전환일 뿐이며 현재 콘텐츠를 건너뛸 수 없습니다. 기존 직선 경로 상수와 `continue` 명령, 전투 사이 자동 순번 증가를 제거했습니다. 구체적인 생성 방식과 임시 콘텐츠는 [ACT_MAP.md](ACT_MAP.md)에 있습니다.

## 상태 계승

`PlayerPiece`의 ID·`origin`은 영구 시작 칸을, `kind`·`upgrades`·`persistent`는 영구 진화·성장을 나타냅니다. 전투 보드의 종류는 일시 승격을 포함하며 `sync_players()`는 영구 종류를 덮어쓰지 않습니다. `Game::check_finish()`에서 `Battle::reset_after_battle()`을 한 번 호출해 종료 위치를 별도 기록한 뒤 생존 기물은 `origin`으로 복귀시키고(하드디스크 보유 시 위치 유지) 사망 기물은 `position: None`으로 보존합니다. `moved`·가호·호신용품 대상·선공권은 초기화하며 보드에만 있는 임시 기물은 제거합니다. 휴식·보상·맵 및 다음 전투는 이 동일한 목록과 보드를 사용합니다. 부활도 표준 진형을 재조회하지 않고 해당 ID의 `origin`을 사용합니다. 새 전투는 적의 기존 고유 편성과 역습 배치 절차를 사용합니다. 영구 덱은 `Game.deck`에, 전투 중 상태는 `CardCombat`에 분리합니다. 종료 시 `CardCombat`을 비우므로 소멸한 카드도 다음 전투에서는 영구 덱에서 다시 만들어집니다.

카드의 비포획 제거는 `remove_nonroyals()`를 통하며 양측 로얄을 보호합니다. 호신용품은 전투 배치 후 살아 있는 아군(킹 포함) 한 기를 런 RNG로 선택합니다. `apply_effect()`가 직접 기물 액션의 각 단계에서 실제 포획자·피해자를 확인하고, 보호 대상의 사망 직후 포획자를 로얄 여부와 무관하게 제거합니다. 모든 연쇄 사망 후 승패를 판정하며 양측 로얄 동시 사망은 기존 킹 사망 우선 규칙으로 패배합니다. 결과 원인은 `SelfDefense`로 구분합니다. 이 임시 지정은 영구 강화에 저장하지 않습니다. 기존 `apply_resolved_card_effect()`에도 결과 보드를 검사하는 원자적 보호를 넣었습니다. 로얄을 없애는 효과라면 보드를 적용하지 않습니다. 직접 기물 포획은 `RoyalCaptured`, 시작 배치의 킹 역습 사망은 `Counterattack`으로 구분합니다.

AI는 기존 합법 후보에서 킹 포획을 최우선으로, 그다음 포획할 기물의 가치로 점수를 매깁니다. 동점은 시드 RNG로 선택합니다. 학습·검색 트리·외부 서비스가 없습니다. 합법 수가 없으면 승리 판정 없이 턴을 넘깁니다.

## 1차 폴리싱 연결

- 기물 정의의 `description`, `upgrades`, `promotions`가 스냅샷과 실제 강화 명령을 함께 구동한다. 행마는 공통 Chesssembly를 사용하며, 체인별 반사벽 설정만 확장한다. [템페스트·바운싱 연결](VARIANTS.md)을 참고한다.
- `PieceImage.tsx`가 key·진영으로 사용 중인 SVG만 조회한다. `RestPanel.tsx`는 로컬 선택/미리보기만 관리하고 확정할 때 기존 명령을 전송한다.
- `WebGame.save/restore`는 버전·캐릭터 ID·시드·성공한 명령(적 행동 포함)을 JSON으로 저장/재생한다. UI는 매 확정 행동 후 localStorage에 쓴다. 별도 보드 복제나 임의 상태 주입 API가 아니다. 게임 규칙 변경으로 재생 결과가 달라지면 저장 버전도 변경해야 한다.

## 덱·혼합 보상 (저장 v6)

시작 영구 덱은 비어 있다. `CardCombat`이 첫 턴을 포함해 턴당 1장 드로우, 손패 10장 상한, 종료 직전 손패 5장 기준의 일반 카드 버리기를 처리한다. 휘발성과 보존은 기존 우선순위를 유지한다.

`Game::check_finish()`는 모든 전투 승리 때 `Reward::Card(CardKind)` / `Reward::Relic(Relic)` 세 칸을 원정 RNG로 생성하고 `Game.rewards`에 보관한다. `snapshot()`은 이를 태그가 있는 `RewardView`로 변환할 뿐 추첨하지 않는다. `choose_reward.index`는 이 보관 목록의 슬롯이며 선택 후 목록을 비운다. 카드·유물 획득과 중복 유물의 가호 대체는 Rust에서 처리한다. React는 타입별 화면과 슬롯 명령만 담당한다.

저장은 시드·성공한 명령의 재생 방식을 유지한다. v6 재개 시 같은 승리 명령에서 같은 보상을 생성하며, 규칙과 RNG 소비가 다른 v1~5는 재생 전에 거부한다. 자세한 검증은 [덱·보상 개편](DECK_REWARDS.md)에 있다.

## 시작 진형과 전투 유물

`Game.relics`의 획득 순서와 `last_battle_positions`를 분리한다. 다음 전투에서 유효한 하드디스크 기록 → 최신 시작 진형 → 영구 origin 순서로 살아 있는 ID만 배치한다. 소용돌이 후 하드디스크 충돌을 처리하고 승패/호신용품 대상을 확정한다. 선공권은 `Battle::legal_actions`와 실제 포획 해결에서, 아이스크림은 공통 플레이어 턴 시작에서 처리한다. 코스트는 포화 증가하는 u32다. React가 소비하는 상태 문구도 Rust에서 생성한다. 좌표·경계 규칙·저장 재현성은 [신규 유물](RELICS.md)에 정리했다.
