# 콘텐츠 구조

선택 가능한 부대는 수호자(`guardian`) 하나다. 부대 추가 절차와 변경 파일은 [부대 추가](adding-squads.md)에 정리했다. 카드와 기물의 엔진 정의는 등록만으로 플레이어에게 제공되지 않는다. 각 부대가 시작 카드·보상 카드·사용 기물·강화 경로를 직접 지정한다.

| 경로 | 역할 |
| --- | --- |
| `crates/chesslayer/src/content/squads/` | 부대 등록, 시작 편성·덱·유물, 보상 카드, 사용 기물·강화 경로, 규칙 |
| `crates/chesslayer/src/content/cards/` | 카드 9종의 효과·키워드·소속. 현재 모두 수호자 전용 |
| `crates/chesslayer/src/content/pieces/` | 몬스터와 플레이어가 참조하는 공통 엔진 기물 정의·행마·강화 경로 원본 |
| `crates/chesslayer/src/content/encounters.rs` | 공용 몬스터와 보스 |
| `crates/chesslayer/src/content/relics.rs` | 공용 유물 8종 |
| `crates/chesslayer/src/map.rs` | 공용 갈림길과 노드 구조 |

기존 기물 행마 계산은 Chesssembly가 담당한다. 플레이어의 강화 가능 여부는 기물 정의의 경로와 소속 부대의 `upgrade_paths`를 모두 확인한다. 카드 획득은 소속 부대의 `reward_cards`와 공용 보상 목록을 확인한다. 현재 공용 카드 목록은 비어 있다.

기존 정의 파일 작성 방식은 [카드 추가](adding-cards.md)와 [기물 추가](adding-pieces.md)를 참고한다. 이 문서들의 등록 예제에 더해 위의 부대 귀속 목록도 갱신해야 실제 원정에서 사용할 수 있다.
