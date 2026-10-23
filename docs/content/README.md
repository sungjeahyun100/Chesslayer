# 콘텐츠 정의와 확장 안내

카드 4종, 기물 8종, 캐릭터 3종은 `crates/chesslayer/src/content/`에서 각각 한 파일로 관리한다. 정의 파일을 만들고 같은 디렉터리의 `mod.rs`에 등록하면 기존 조회·실행 경로에 연결된다. 카드와 기물은 **등록과 실제 획득·등장 경로 지정이 별도**다. 캐릭터는 등록하면 선택 화면에 바로 나온다.

모든 경로는 저장소 루트 기준이다.

- [카드 추가](adding-cards.md): 정의, 키워드, 대상 판정, 효과, 시작 덱·보상
- [기물 추가](adding-pieces.md): ID, Chesssembly 행마, 표시, AI 가치, 전투 편성
- [캐릭터 추가](adding-characters.md): 기본 정보, 시작 유물, 지원 범위
- [검증 기록과 최소 등록 테스트](validation.md)

## 조사 결과와 변경 전후

변경 전에는 다음 위치를 함께 수정해야 했다.

```text
crates/chesslayer/src/
├── cards.rs     CardKind, 카드 설명·비용·키워드, 시작 덱, 전투 더미
├── battle.rs    기물 ID, 영문 카탈로그, 한글 이름, 행마 생성, 폰 첫 두 칸
├── content.rs   캐릭터 3종 목록, 유물 정보, 적 편성, 상수, RNG
└── game.rs      카드별 실행 분기·총 진형·대상 필터, AI 기물 가치,
                두 곳의 카드 보상 목록, 캐릭터 선택·유물 실행
web/src/main.tsx 기물 종류별 기호·배지
```

카드 추가는 정의와 실행·대상 분기를, 기물 추가는 ID·카탈로그·행마 벡터·이름·유효 범위·AI·화면을 각각 맞춰야 했다. 캐릭터는 이미 데이터 목록으로 선택 화면과 연결되지만 한 파일에 모여 있었다. 유물은 캐릭터 외에도 맵 보상에서 획득하므로 캐릭터 전용 효과가 아니다.

변경 후:

```text
crates/chesslayer/src/
├── content/
│   ├── mod.rs                 공통 상수, 유물 정보, 적 편성, RNG
│   ├── cards/
│   │   ├── mod.rs             CardKind, CardDefinition, 등록, 시작 덱·보상
│   │   ├── jeans.rs           청바지
│   │   ├── ward.rs            가호, 재사용 가능한 대상·효과 함수
│   │   ├── gun.rs             알레킨의 총, 진형 검사·사격 범위 상수
│   │   └── fleeting_ward.rs   찰나의 가호 (ward의 효과 재사용)
│   ├── pieces/
│   │   ├── mod.rs             PieceDefinition, ID 재노출, ALL, 조회·카탈로그
│   │   ├── movement.rs        공통 방향·DSL 조합, 첫 두 칸 DSL
│   │   ├── pawn.rs
│   │   ├── knight.rs
│   │   ├── bishop.rs
│   │   ├── rook.rs
│   │   ├── queen.rs
│   │   ├── king.rs
│   │   ├── amazon.rs
│   │   └── tempest_pawn.rs
│   └── characters/
│       ├── mod.rs             Character, ALL, characters()
│       ├── vanguard.rs
│       ├── wanderer.rs
│       └── guardian.rs
├── cards.rs                   공통 손패·더미·마나·키워드 처리
├── battle.rs                  보드·턴·합법 행동·가호·로얄·역습·지속 상태
└── game.rs                    명령 검증, AI, 원정·보상·휴식, 스냅샷
```

`chessembly`는 기존 공통 보드·액션·DSL 계산을 계속 담당한다. 코어/룰셋 패키지를 다시 나누지 않았다. 파일 탐색, 자동 등록 매크로, 플러그인 로더, 별도 디버그 API도 없다.

## 정의 파일과 코어의 경계

| 종류 | 정의 파일이 소유하는 것 | 공통 실행부가 소유하는 것 |
| --- | --- | --- |
| 카드 | 이름·설명·비용·키워드, 대상 후보, 대상 유효성 검사, 효과 함수 | 플레이어 턴·손패·비용 검사, 성공 후 마나/더미 처리, 사용 기록 |
| 기물 | 숫자 ID·DSL 이름·표시 이름, 행마 DSL 생성, AI 가치, 능력 설명·단일 강화 경로, 첫 두 칸 사용 여부 | Chesssembly 후보 계산·적용, 이동 이력, 턴 소비, 가호, 로얄 판정, 계승 |
| 캐릭터 | 문자열 ID·이름·제목·설명, 시작 유물 한 개 | 공통 덱·16기 진형 초기화, 유물 획득·발동, 원정 진행 |

각 종류는 같은 자료형과 등록 경로를 사용한다. 카드와 기물의 선택 속성은 생성자 기본값을 덮어쓰므로 특수 속성을 매번 나열할 필요가 없다. 캐릭터는 현재 데이터만으로 정의되며 별도 실행 콜백이 없다.

가호의 지속 처리·로얄 보호·이동 적용을 콘텐츠에서 복사하지 않는다. 가호 변형은 `ward::effect`, 기물 변형은 `movement`의 DSL 조합 및 `Program`, 비포획 제거는 `Battle::remove_nonroyals()`를 재사용한다. 총의 진형 검사는 `gun.rs`에만 있으며 화면 대상 후보와 실제 사용 검증이 같은 함수를 호출한다.

## 등록·초기화·조회·실행

1. **카드**: `CardKind::definition()`의 명시적 `match`가 각 파일의 `definition()`을 호출한다. `starter_deck()`와 `REWARD_POOL`은 획득 경로다. `Game::play_card()`는 공통 검증 뒤 `definition.effect`를 호출하고 성공한 경우에만 비용과 더미를 처리한다. 스냅샷은 `target_candidates`를 호출한다. 함수 포인터는 직렬화하지 않으므로 브라우저는 데이터만 받는다.
2. **기물**: `pieces::ALL` → 숫자 ID로 인덱싱한 보드 이름 카탈로그 → 전투 시작 시 모든 행마를 `Program::parse()` → ID별 프로그램 보관 → `Battle::legal_actions()` → 공통 규칙 적용 순서다. `pieces::definition(kind)`는 이름·AI·표시에도 사용한다. React는 Rust가 제공하는 `key`와 진영으로 공통 `PieceImage`를 통해 SVG를 그린다. 강화 UI와 실행은 같은 `upgrades` 정의를 사용한다.
3. **캐릭터**: `characters::ALL` → `characters()` → WASM `catalog()` → 선택 화면이다. `Game::new(id, seed)`도 같은 목록에서 ID를 찾고 `relic`을 런에 넣는다. 기존 유물 발동은 `Game::start_battle()`와 `BattleOptions`를 통해 유지한다.

## 추가할 때 수정하는 파일

| 작업 | 필수 | 게임에 등장시키려면 | 특수한 경우만 |
| --- | --- | --- | --- |
| 카드 | `content/cards/<이름>.rs`, `content/cards/mod.rs`의 모듈·enum·match | 같은 `mod.rs`의 시작 덱 또는 보상 목록 | 새 턴 이벤트/덱 조작 등 현재 효과 인자로 할 수 없는 기능은 코어 수정 |
| 기물 | `content/pieces/<이름>.rs`, `content/pieces/mod.rs`의 모듈·ID 재노출·ALL | 적은 `content/mod.rs::encounter()`, 아군 초기 배치는 `battle.rs::standard_roster()` 등 실제 공급처 | SVG 두 진영 파일과 출처 등록. 새로운 지속 능력·분기 강화·로얄 규칙은 해당 코어 수정 |
| 캐릭터 | `content/characters/<이름>.rs`, `content/characters/mod.rs`의 모듈·ALL | 추가 작업 없이 선택 화면 노출 | 새 유물 또는 캐릭터별 덱·진형은 현재 정의 규격 밖 |

등록 이후 기존 콘텐츠의 수치·행마·효과를 수정하는 일은 대부분 해당 정의 파일 하나에서 끝난다. 신규 콘텐츠는 Rust 모듈 등록 때문에 `mod.rs` 수정도 필요하다.

## 유지한 ID와 의존 관계

- 카드 직렬화 키: `jeans`, `ward`, `gun`, `fleeting_ward`. 인스턴스 `Card.id`와 카드 종류는 별개다. 시작 덱의 기존 순서와 ID 1~10을 유지했다. 이후 획득 ID는 시작 덱의 최댓값 다음부터 할당한다.
- 기물 종류: 폰 1, 나이트 2, 비숍 3, 룩 4, 퀸 5, 킹 6, 아마존 7, 템페스트 폰 8, 템페스트 룩/비숍/나이트/퀸 9~12, 바운싱 폰/룩/비숍/퀸 13~16. 기존 DSL 이름은 유지한다. 0은 예약되어 있다.
- 캐릭터 ID와 순서: `vanguard`, `wanderer`, `guardian`. 시작 유물: `Haste`, `Whirlwind`, `SelfDefense`.
- 보상 인덱스: 가호 0, 청바지 1, 총 2. 기존 명령 기보가 이 순서를 참조한다. 새 보상은 보통 뒤에 추가한다.
- 청바지는 정확히 `ROOK | QUEEN`, 총 진형은 정확히 `ROOK, ROOK, QUEEN`을 검사한다. 퀸 행마를 가진 아마존이 자동으로 퀸 취급을 받지 않는다.
- 바운싱 폰은 일반 폰 DSL을 재사용한다. 템페스트 폰은 독립 DSL이며 첫 두 칸 전진을 하지 않는다. 일반 폰/바운싱 폰의 첫 두 칸은 이동 이력과 시작 랭크 조건을 공통 실행부에서 검사한다.

## 현재 제약

보드는 8×8, 진영은 플레이어 0/적 1이다. 플레이어 로얄은 기존 `KING` 종류의 기물 하나이며 적 로얄은 편성에서 지정한 인스턴스 ID다. 체크메이트·캐슬링·앙파상은 지원하지 않는다. 강화된 폰의 최종 랭크 승격은 `promotions` 목록과 명시적인 선택 명령으로 처리한다. [계열 구현](../VARIANTS.md)을 참고한다.

모든 캐릭터는 같은 시작 덱과 표준 16기 진형을 사용하고 유물 한 개만 다르다. 유물 자체는 캐릭터 정의로 옮기지 않았다. 새로운 능력의 이벤트 구독 시스템은 없다. 카드 효과에는 `Battle`과 선택 기물 ID만 전달하며 덱·마나·원정 RNG를 직접 조작하는 인터페이스는 없다.

1차 폴리싱에서 로컬 저장/이어하기를 추가했다. 기존 JSON 명령·ID·기보 참조는 유지했으며 스냅샷의 `glyph`, `badge`는 이미지 조회용 `key`와 능력 `description`으로 교체했다. 새로 추가한 콘텐츠가 결정적 재생 결과를 바꾸면 저장 버전 호환성도 검토한다. 시작 덱 장수를 바꾸면 UI의 고정 문구와 장수에 의존하는 기존 테스트도 검토해야 한다. 기물 등록 누락이나 중복 ID는 [등록 테스트](validation.md)로 확인한다.
