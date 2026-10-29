# 부대 추가

v0.3.0에는 `guardian`, `vanguard`, `heretic`, `artillery`, `special`이 등록되어 있다. 현재 상태와 임시 규칙은 [v0.3.0 안내](../V03_PROTOTYPE.md)를 따른다. `crates/chesslayer/src/content/squads/guardian.rs`가 수호자의 시작 유물(호신용품), 표준 16기 편성, 빈 시작 덱, 전투 보상 카드 풀, 사용 가능한 플레이어 기물과 강화 경로를 명시한다. 수호자의 카드 9종은 모두 수호자 전용이다. 보상 풀에는 기존과 같이 찰나의 가호를 제외한 8종만 들어간다.

새 부대를 추가할 때는 다음 순서로 작업한다.

1. `crates/chesslayer/src/content/squads/<id>.rs`에 `Squad` 정의를 만든다. `id`, 표시 정보, 시작 유물, `starting_roster`, `starting_deck`, `reward_cards`, `available_pieces`, `upgrade_paths`, 중복 유물 대체 카드, `rules`, `unlock_floor`, `reward_relics`를 모두 지정한다. 수호자의 배열을 기본값으로 복사하지 않는다. 초기 기물의 `origin`은 휴식 부활과 다음 전투 복귀에 사용된다.
2. `squads/mod.rs`에 모듈을 선언하고 `ALL`에 등록한다. `squads()`가 WASM `catalog()`와 `Game::new(id, seed)`에 함께 사용되므로 선택 UI에 자동으로 나타난다. `Game::new`는 카드 소속, 보상 풀 크기, 시작 기물과 강화 경로를 검증한다.
3. 전용 카드는 `content/cards/`에 정의하고 `CardKind::affiliation()`에서 새 부대 ID를 지정한다. 시작 카드와 보상 카드는 해당 부대의 두 배열에 각각 넣는다. 향후 공용 카드는 `CardAffiliation::Shared`로 지정하고, 보상에 넣으려면 비어 있는 `SHARED_REWARD_POOL`에 명시적으로 등록한다. 현재 공용 카드는 없다.
4. 새 플레이어 기물은 `content/pieces/`에 기존 방식대로 정의한 뒤 새 부대의 `available_pieces`에 넣는다. 영구 강화는 엔진의 기물 정의에 경로를 추가하고 부대의 `upgrade_paths`에도 `(출발 종류, 목표 종류)`를 넣는다. 승격 목표도 `available_pieces`에 넣는다. 몬스터는 같은 엔진 기물 정의를 사용할 수 있지만 플레이어의 사용 권한은 부대 목록으로 제한된다.
5. 독자적인 능력이나 규칙이 필요하면 `SquadRules`에 변형을 추가하고 해당 이벤트의 `Game` 처리 코드에 구현한다. 유물 효과는 부대 코드에 복제하지 않는다. 전투 엔진과 Chesssembly의 행마 정의는 필요한 경우가 아니면 변경하지 않는다.

유물은 각 부대의 `reward_relics`로 제한되며 기존 8종은 수호자 전용이다. 현재 공용 콘텐츠는 `content/encounters.rs`의 몬스터·보스, `map.rs`의 갈림길·노드 구조다. 가속과 소용돌이는 수호자 유물 보상 풀에 남아 있다.

## 변경 파일과 확인

- `content/squads/`: 선택 목록과 부대별 시작 상태·보상·기물·강화 권한
- `content/cards/`, `game.rs`: 카드 소속 및 부대별 획득·강화 검증
- `chesslayer-web/src/lib.rs`, `web/src/`: 저장 데이터와 선택 화면의 부대 명칭
- `crates/chesslayer/tests/content.rs`: 등록·귀속·기존 수호자 상태 검사

`cargo test --workspace`로 Rust 회귀 테스트를 실행한다. `npm run wasm --prefix web && npm run build --prefix web`로 WASM과 화면 타입 연결을 확인한다. 웹 조작은 `npm run test:e2e --prefix web`로 검사한다. 저장 payload는 `squad` 필드와 버전 15를 사용한다. 이전 버전 저장은 기존 안내와 함께 재생 전에 거부된다.
