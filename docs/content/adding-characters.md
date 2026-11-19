# 부대 추가 (v0.4.0)

부대의 등록 단위는 `crates/chesslayer/src/content/characters/<id>.rs`의 `SQUAD: SquadDefinition`입니다. 기존 부대 파일을 복사하면 ID·이름·설명, 시작 기물 진형, 시작 덱, 카드 보상 풀, 유물 보상 풀, 턴 방식, 자원을 한곳에서 볼 수 있습니다. `Character`는 웹 카탈로그에 사용하는 기존 자료형으로 유지합니다.

1. `content/characters/<id>.rs`에 `DEFINITION: Character`와 `SQUAD: SquadDefinition`을 만듭니다. `formation`은 8개의 후열 기물과 선택적 전열 기물 종류를 지정합니다. `None`이면 기존 표준 16기 진형을 씁니다. 배치 순서가 기물 ID와 영구 강화 기록의 기준이므로 출시 후 재정렬하지 마세요.
2. `starting_deck`은 카드 종류를 **인스턴스 ID 순서**로 나열합니다. 카드 종류는 `content/cards/mod.rs`의 `CardKind`에 등록하고, 카드 설명·비용·대상·효과는 각 카드 파일에 둡니다. `card_pool`과 `relic_pool`은 원정의 보상과 상점에서 사용됩니다. 배열 순서를 바꾸면 같은 시드의 결과가 달라질 수 있습니다.
3. `turn_rule`은 현재 `Standard` 또는 `FreshFive`를 선택합니다. `resource`는 일반 마나와 포병대대의 지속 탄약을 구분합니다. `card_movement`는 복수귀의 카드 기반 행마권 규칙을 켭니다. 새로운 자원/턴 모델이 필요하면 `ResourceRule`/`CardTurnRule`과 해당 전투 경계만 확장하고, 기존 ID의 동작은 유지하세요.
4. `content/characters/mod.rs`에 모듈을 선언하고 `ALL`과 `SQUADS`에 같은 위치로 추가합니다. `ALL` 순서가 선택 화면 순서입니다. `characters::definition(id)`가 원정 초기화, 진형, 카드·유물 풀을 연결합니다.
5. 새 부대 전용 카드가 필요하면 `content/cards/<카드>.rs`의 `CardDefinition` 함수 포인터로 대상과 효과를 연결하세요. 기존 전투 효과 계열에 속하면 `battle/mechanics/*_cards.rs`의 해당 계열에 실행을 추가합니다. 새 부대 고유 전투 규칙은 `battle`의 작은 전용 모듈에서 구현하고, 시작 경계에서만 연결하세요.

기존 저장은 부대 문자열 ID와 명령 기록을 재생합니다. 기존 부대의 ID, 시작 덱 순서, 진형의 기물 ID, 보상 후보 순서, 난수 호출 시점을 바꾸지 마세요. `cargo test --workspace`와 `chesslayer-web`의 저장 복원 테스트로 확인합니다.
