# v0.4.0 엔진 및 콘텐츠 경계

- `content/encounters/definition.rs`: 고정 인덱스, 콘텐츠 ID, 층·분류, 배치 순서, 로얄 목록, 로얄 추첨 및 전투 시작 규칙의 정의.
- `content/encounters/floor*/encounter_XX.rs`, `final_boss.rs`: 몬스터 하나당 한 파일. `mod.rs`가 기존 `ENCOUNTERS`, `encounter(index)`, 층별 후보 목록을 유지합니다. `encounter()`는 난수를 소비하지 않으며 로얄 추첨은 전투 진입에서만 수행합니다.
- `content/characters/<id>.rs`: `DEFINITION`과 `SQUAD`에 부대 메타데이터·진형·덱·보상 풀·턴·자원 설정을 함께 둡니다.
- `battle.rs`: 전투 상태, 초기화, 턴 및 행동 적용. `battle/movement.rs`가 행마 후보와 강화, `battle/effects.rs`가 피해·제거·승패를 담당합니다. `battle/boss.rs`, `avenger.rs`, `combat_state.rs`는 기존 고유 행동과 전투 수치를 유지합니다.
- `battle/mechanics.rs`: 효과 상태와 공통 실행 경계. 하위 `shooting.rs`, `statuses.rs`, `combat.rs`, `choices.rs`, `relics.rs`가 각 효과를 담당합니다. `cards.rs`가 카드 대상·공통 검증을 수행하고 부대별 `*_cards.rs`로 효과를 보냅니다.
- `cards.rs`: 카드 비용·선택·효과 실행·카드 이동을 성공 시에만 확정합니다. `content/cards/<카드>.rs`가 메타데이터와 실행 함수 포인터를 연결합니다.
- `game.rs`: 원정 상태·명령 디스패치. `game/progression.rs`, `battle_flow.rs`, `rewards.rs`, `shop.rs`, `card_play.rs`, `snapshot.rs`가 각 경계를 담당합니다. 기존 `events.rs`, `expansion.rs`의 이벤트·복수귀 규칙도 유지합니다.

유물의 시작 설정과 현재 상태 표시는 `battle/mechanics/relics.rs`에, 명중률·탄약 관련 발동은 `shooting.rs`에, 포획·사망 발동은 `combat.rs`와 `statuses.rs`에 있습니다. 원정 보상과 자원 변경은 `game/rewards.rs`·`events.rs`에서 적용합니다. 효과를 한꺼번에 재실행하는 별도 유물 루프는 없으며, 각 사건의 기존 실행 지점에서 발동합니다.

## 몬스터 추가

해당 층에 `encounter_XX.rs`를 만들고 `EncounterDefinition`을 채웁니다. `placements` 배열은 적 기물 ID 100부터 순서대로 할당되므로 기존 항목은 재정렬하지 않습니다. 고정 로얄은 `royal_id`와 `additional_royal_ids`에, 추첨은 `RoyalSelection::UniformDeployed`에 지정합니다. 시작 규칙은 `start_rules`에 표시하고 기존에 없는 규칙이라면 전투 진입 코드에 구현을 연결합니다. 층 `mod.rs`에 모듈을 선언하고 상위 `ENCOUNTERS`의 **끝**에 추가한 뒤 해당 층 후보 배열에 인덱스를 넣습니다. 지도 선택 RNG와 기존 인덱스는 그대로 둡니다.

`tests/fixtures/encounters_v033.txt`와 `all_legacy_rosters_and_royals_are_unchanged` 테스트는 기존 37개 정의의 기물 ID·종류·좌표·로얄을 고정합니다. 기존 콘텐츠를 바꿀 때 이 기준 파일을 자동 갱신하지 마세요.

## 호환성 경계

`Encounter`, `BattleStart`, `Command`, `Snapshot`, `CombatEvent`의 공개 데이터 형식은 유지합니다. 브라우저 저장은 상태가 아니라 시드와 명령 기록을 재생하므로 카드·몬스터 공급 순서와 RNG 호출 위치도 호환성의 일부입니다. 새로운 저장 필드는 이번 리팩터링에 추가하지 않았습니다.

기존 최종 보스 강제 제거는 `remove_pieces`에서 `boss_hit`을 거친 뒤 제거되며, 복수귀 턴은 `card_movement` 분기에서 수동 종료를 유지합니다. 두 경로 모두 규칙 변경 없이 옮겼습니다. 새로운 강제 제거/턴 종료 효과를 추가할 때 이 경계의 HP·턴 소모를 별도 검증해야 합니다.

검토 중 발견한 잠재적 충돌: `remove_for_limiter`는 최종 보스가 HP를 잃고 살아남아도 기물 사망 로그를 기록할 수 있습니다. 또한 `Battle::complete_action`의 복수귀 분기는 `forced_turn_end`를 지우므로 새로운 강제 턴 종료 효과가 이 경로를 거치면 수동 종료 규칙과 충돌할 수 있습니다. 이번 변경에서는 두 동작을 수정하지 않았습니다.
