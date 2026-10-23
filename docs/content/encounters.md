# 1층 몬스터 편성

> 최신 저장 버전은 전투 경계·호신용품 변경에 따른 **4**입니다. 아래 버전 3 설명은 편성 변경 당시 기록이며 [최신 호환성 안내](../BATTLE_BOUNDARIES.md)를 참고하세요.

`crates/chesslayer/src/content/encounters.rs`의 `ENCOUNTERS`가 ID·명칭·설명을 관리하고 `encounter(index)`가 초기 편성을 만든다. 기존 지도는 숫자 인덱스를 유지한다. 다음은 실제 등록 목록이며 일반전 네 종류의 좌표·종류·인스턴스 ID 순서는 이전과 같다.

| 인덱스 | 콘텐츠 ID | 이름 | 기본 초기 배치 | 로얄 |
| --- | --- | --- | --- | --- |
| 0 | 1F-N01 | 홀로 선 왕 | 킹 e8 | 킹 e8 |
| 1 | 1F-N02 | 왕관을 쓴 기사 | 나이트 e8 | 나이트 e8 |
| 2 | 1F-N03 | 쌍둥이 주교 | 킹 e8, 비숍 c8·f8 | 킹 e8 |
| 3 | 1F-N04 | 폰의 성벽 | 킹 e8, 폰 a7~h7 | 킹 e8 |
| 4 | 1F-E01 | 마하라자 | 아마존 e8 한 기 | 아마존 e8 |
| 5 | 1F-E02 | 호드 | 5~8랭크 전체 32칸과 b4·c4·f4·g4의 폰, 총 36기 | 폰 e8 |
| 6 | 1F-E03 | 건어물 | 흑 표준 초기 배치 16기 | 킹 e8 |
| 7 | 1F-E04 | 런던 | 아래 고정 좌표 16기 | 킹 e8 |
| 8 | 1F-B01 | 구행 | 구행 e8 한 기 | 구행 e8 |

런던은 킹 e8, 퀸 d8, 룩 a8·h8, 비숍 d6·f5, 나이트 d7·f6, 폰 a7·b7·c6·d5·e6·f7·g7·h7이다. 파일을 유지하고 랭크만 뒤집은 고정 좌표이며 포지션 생성 알고리즘은 없다. 모든 편성의 로얄 인스턴스 ID는 100이다. `왕관의 근위대`는 등록 및 출현 풀에서 제거했다.

`ActMap::generate(seed)`는 `ELITES`의 네 종류를 별도의 결정적 난수 흐름으로 선택한다. 같은 시드에는 같은 엘리트가 배정되며, 기존 지도 그래프·일반 몬스터·유물의 난수 흐름을 소비하지 않는다. 보스는 `HOOK_BOSS` 한 종류다. 카드 선택과 유물/중복 유물 대체 카드, 공통 AI, 로얄 직접 포획, 가호, 역습 및 종료 규칙은 유지한다. 위 좌표는 기본 편성이며 기존 소용돌이 유물은 적의 위치를 섞고, 기존 역습/호신용품은 계승된 아군과 겹치는 칸을 처리한다.

구행은 `content/pieces/hook.rs`에 종류 ID 17, key `hook`으로 독립 등록했다. 각 직교 방향의 `do … while`은 직선 경로를 진행하고, 매 빈 칸의 두 자식 블록은 수직 방향으로 `take-move … repeat(1)`을 실행한다. 자식 블록을 마치면 부모 좌표로 복귀하므로 추가 회전은 생기지 않는다. `take-move`가 적을 포획하거나 아군에 막히면 해당 경로가 끝나며, 포획한 지점에서는 분기하지 않는다. 엔진이나 Chesssembly 문법, WASM 인터페이스, 공통 AI의 변경은 없다. 별도 페이즈·호위·기믹도 없다. 이미지는 사용자가 지정한 쇼기 구행 SVG를 사용하며, 아군은 위·적군은 아래를 향하도록 같은 그림을 180도 회전한다. 색은 그대로 유지한다. [에셋 안내](../ASSETS.md)에 원본과 방향을 기록했다.

브라우저 저장은 상태 스냅샷이 아닌 명령 재생이므로 같은 시드라도 이전 엘리트/보스에서의 명령은 다른 결과를 만들 수 있다. 저장 payload 버전을 2에서 3으로 올렸고 기존 localStorage 키 `chesslayer.run.v1`은 유지한다. 버전 1·2는 명령 역직렬화와 재생 전에 새 원정 필요 안내를 표시한다. 이어하기 실패만으로 원본 저장을 덮어쓰지 않으며, 사용자가 새 원정을 시작할 때 교체한다. 기존 기록의 자동 이관은 지원하지 않는다. 새 버전 저장/복원은 정상 명령 재생으로 검증한다.

검증 명령과 예상 결과는 [TESTING.md의 1층 편성 절](../TESTING.md#1층-엘리트-4종과-구행-보스--2026-10-23)에 있다.

## 변경 파일 목록

- **엔진 연결 및 콘텐츠 소스 (7개)**: [crates/chesslayer-web/src/lib.rs](../../crates/chesslayer-web/src/lib.rs), [crates/chesslayer/src/content/mod.rs](../../crates/chesslayer/src/content/mod.rs), [crates/chesslayer/src/content/pieces/mod.rs](../../crates/chesslayer/src/content/pieces/mod.rs), [crates/chesslayer/src/game.rs](../../crates/chesslayer/src/game.rs), [crates/chesslayer/src/map.rs](../../crates/chesslayer/src/map.rs), [crates/chesslayer/src/content/encounters.rs](../../crates/chesslayer/src/content/encounters.rs), [crates/chesslayer/src/content/pieces/hook.rs](../../crates/chesslayer/src/content/pieces/hook.rs).
- **React 및 이미지 (5개)**: [web/src/ActMap.tsx](../../web/src/ActMap.tsx), [web/src/assets/pieces/provenance.json](../../web/src/assets/pieces/provenance.json), [web/src/main.tsx](../../web/src/main.tsx), [web/src/assets/pieces/hook-black.svg](../../web/src/assets/pieces/hook-black.svg), [web/src/assets/pieces/hook-white.svg](../../web/src/assets/pieces/hook-white.svg).
- **테스트 및 기보 (10개)**: [crates/chesslayer/tests/act_map.rs](../../crates/chesslayer/tests/act_map.rs), [crates/chesslayer/tests/fixtures/bouncing-promotion.json](../../crates/chesslayer/tests/fixtures/bouncing-promotion.json), [crates/chesslayer/tests/fixtures/tempest-promotion.json](../../crates/chesslayer/tests/fixtures/tempest-promotion.json), [crates/chesslayer/tests/fixtures/winning-run.json](../../crates/chesslayer/tests/fixtures/winning-run.json), [crates/chesslayer/tests/prototype.rs](../../crates/chesslayer/tests/prototype.rs), [crates/chesslayer/tests/variants.rs](../../crates/chesslayer/tests/variants.rs), [web/tests/game.spec.ts](../../web/tests/game.spec.ts), [web/tests/rest.spec.ts](../../web/tests/rest.spec.ts), [web/tests/variants.spec.ts](../../web/tests/variants.spec.ts), [crates/chesslayer/tests/encounters.rs](../../crates/chesslayer/tests/encounters.rs).
- **문서 (10개)**: [README.md](../../README.md), [docs/ACT_MAP.md](../ACT_MAP.md), [docs/ASSETS.md](../ASSETS.md), [docs/POLISHING.md](../POLISHING.md), [docs/PROTOTYPE.md](../PROTOTYPE.md), [docs/TESTING.md](../TESTING.md), [docs/VARIANTS.md](../VARIANTS.md), [docs/content/README.md](README.md), [docs/content/adding-pieces.md](adding-pieces.md), [docs/content/encounters.md](encounters.md).
