# Chesslayer · v0.2.1

템페스트 5종·바운싱 4종의 휴식 강화, 전투 승격, 저장 연결은 [구현 안내](docs/VARIANTS.md)에 있습니다.
나이트라이더·유니콘·바운싱 나이트라이더·구행·텐쿠·추기경의 행마와 강화 경로는 [고급 행마 안내](docs/ADVANCED_MOVEMENT.md)에 있습니다.
행마의 신·바운싱 템페스트·태풍 8종의 전체 강화 트리와 해금 조건은 [상위 복합 기물 안내](docs/COMPOSITE_PIECES.md)에 있습니다.

Rust 엔진이 실제 게임 상태를 관리하는 브라우저 체스 로그라이크입니다. React 화면은 Rust/WASM이 반환한 기물·합법 수·카드 대상을 표시하고 플레이어의 명령을 전달합니다.

**캐릭터 선택 → 1막 분기 맵 → 노드 선택 → 전투·보상·휴식·보물 → 1막 보스 → 2막 분기 맵과 보스**를 플레이할 수 있습니다. 새 원정과 다음 막 진입에는 ACT 전환 연출이 있으며 건너뛰기와 모션 감소 설정을 지원합니다. 적의 지정 로얄을 직접 포획하거나 호신용품·하드디스크 충돌로 제거하면 승리하고, 킹을 잃으면 원정이 끝납니다. 생존 기물은 각자의 시작 칸으로 복귀하며(하드디스크 보유 시 종료 위치 유지) 사망·영구 강화·진화·성장과 보상 카드를 계승합니다. 다음 막으로 올라갈 때는 사망한 아군 전원이 부활합니다. 전투 승격·가호·이동 이력은 초기화합니다. [전투 경계와 호신용품](docs/BATTLE_BOUNDARIES.md)에 구현과 검증을 정리했습니다.

## 실행

준비: Rust 1.85 이상, Node.js 20.19+ 또는 22.12+, npm, wasm-pack. 이 작업에서는 Rust 1.99.0 / Node 22.20.0 / wasm-pack 0.14.0을 사용했습니다.

저장소 루트에서 실행합니다.

```sh
rustup target add wasm32-unknown-unknown
# wasm-pack이 없다면 한 번 설치
cargo install wasm-pack --locked

npm ci --prefix web
npm run wasm --prefix web
npm run dev --prefix web
```

브라우저에서 **http://127.0.0.1:5173**을 엽니다. `새 게임 시작 → 캐릭터 선택 → 런 시작`을 누르세요. 종료하려면 서버 터미널에서 Ctrl+C를 누릅니다.

- 지도 아래쪽의 세 시작점 중 하나를 선택합니다. 초록색 노드만 이동할 수 있습니다.
- 백 기물 클릭 → 표시된 합법적인 칸 클릭으로 이동합니다.
- 전투 중 적 기물 클릭 → 현재 보드에서 가능한 행마와 기물 정보를 조회합니다. 파란 원은 빈 도착 칸, 빨간 테두리는 포획 대상, 점선은 계산 과정에서 확인된 경유 경로입니다. 다시 클릭하거나 빈 칸·Esc를 누르면 닫힙니다. 조회는 행동이나 저장 기록을 변경하지 않습니다.
- 대상이 필요한 카드 클릭 → 표시된 기물을 클릭합니다. Esc로 선택을 취소합니다.
- 가속 캐릭터는 두 행동 사이에도 카드를 사용할 수 있습니다.
- 행동을 모두 쓰거나 `턴 종료`를 누르면 적이 자동으로 행동합니다.
- 승리 후 카드·유물 보상 3개 중 하나를 고르면 지도로 돌아옵니다. 현재 위치에서 연결된 다음 노드를 선택하세요.
- 보물에서는 유물을 받고, 엘리트 승리 시 선택 보상과 추가 유물을 얻습니다. 휴식에서는 보드의 아군 기물 또는 초기 빈칸을 선택해 강화·부활을 미리 보고 확정합니다. 노드당 한 번 행동하며 그냥 출발할 수도 있습니다.
- 전투 중에도 `전체 지도 보기`로 남은 경로를 확인할 수 있습니다.
- 진행은 이 브라우저에 자동 저장됩니다. 새로고침 후 `저장된 원정 이어하기`를 누르면 재개합니다. 새 원정 시작은 기존 저장을 대체합니다.

Rust 코드를 바꾼 뒤에는 `npm run wasm --prefix web`을 다시 실행하고 브라우저를 새로고침하세요. React/CSS는 Vite가 즉시 반영합니다. WASM 생성물은 Git에 포함하지 않습니다.

웹 화면의 버전은 `web/package.json`에서 읽습니다. 이후 작은 변경사항을 반영할 때는 `cd web && npm version patch --no-git-tag-version`으로 패치 버전을 올리면 잠금 파일과 화면 표시가 함께 갱신됩니다. 게임 저장 데이터 버전은 별도로 관리합니다.

적 기물 조회는 `Game::inspect_enemy`와 WASM의 `WebGame.inspect_enemy`가 현재 보드를 읽어 반환합니다. 이동 실행은 계속 `Battle::legal_actions`의 턴 검증을 거칩니다. 점선은 Chesssembly가 실제로 기록한 착지점 중 반복 점프나 방향 전환이 확인된 경로에만 표시하며, 순간이동처럼 경유 경로가 없는 행동에는 그리지 않습니다.

조회 검증은 `cargo test -p chesslayer --test variants enemy_preview`로 실행합니다. 이 테스트는 상대 턴의 적 조회가 보드·턴·행동 횟수를 바꾸지 않고 이동·포획·특수 경로를 구분하는지 확인합니다. `npm run test:e2e --prefix web -- --grep '적 로얄 조회'`는 로얄 정보, 선택 전환, Esc 및 저장 불변성을 확인합니다. 전체 브라우저 회귀 검증은 `npm run test:e2e --prefix web`로 실행합니다.

프로덕션 빌드도 같은 WASM 엔진을 사용합니다.

```sh
npm run wasm --prefix web
npm run build --prefix web
npm run preview --prefix web
```

정적 배포 산출물은 `web/dist/`입니다. `index.html`을 파일로 직접 여는 대신 HTTP 서버를 사용하세요. 첫 WASM 빌드는 wasm-bindgen CLI 다운로드 때문에 네트워크가 필요할 수 있습니다. 다운로드가 제한된 환경에서는 Cargo.lock과 같은 버전의 CLI를 설치한 뒤 실행합니다: `cargo install wasm-bindgen-cli --version 0.2.126 --locked`.

## 구현 범위

- 표준 백 진형, 클릭 이동, 이동 애니메이션, 양측 로얄·가호 표시
- 일반 전투 4종, 엘리트 4종(마하라자·호드·건어물·런던), 한 번 직각으로 꺾는 구행 보스, 시드로 재현 가능한 휴리스틱 적 AI. [현재 편성·저장 호환성](docs/content/encounters.md)
- 7단계·18~20개 노드의 전체 맵, 세 시작점, 반복 분기와 합류, 현재 위치·방문·선택 경로 표시
- Rust에서 검증하는 경로 선택과 노드 완료, 보물·엘리트 유물 보상
- 빈 시작 덱, 마나, 턴당 1장 드로우, 손패 5장 이상일 때 일반 카드 버림, 손패 10장 제한, 재셔플, 소멸·보존·선천성·휘발성
- 청바지, 가호, 알레킨의 총과 가호의 휘발성 변형
- 시작 유물이 다른 캐릭터 3종, 기존 유물 3종과 KIA·QID·Hedgehog·선공권·아이스크림·하드디스크 6종
- 기물별 영구 시작 칸, 사망·영구 성장·보상 카드 계승, 전투 한정 상태 초기화
- 보드 선택 기반 휴식, 강화 테크트리·부활 미리보기·최종 확인, 로컬 저장·이어하기
- 기본·강화·신규 기물 30종 × 양 진영 SVG, 카드 9종과 유물 9종의 전용 벡터 이미지, 데스크톱·모바일 미리보기
- 한국어 기본 오류 메시지와 게임 플레이용 전투 기록, 모바일 레이아웃

디버그 화면이나 상태 조작 API는 만들지 않았습니다. 2층, 상점·랜덤 이벤트, 계정, 서버 저장, 멀티플레이, 학습형 AI는 이번 범위에 없습니다. 제공된 SVG를 적용했으며 일부 원본의 재배포 권한은 아직 확인되지 않았습니다. [에셋 출처](docs/ASSETS.md)를 확인하세요.

## 코드와 문서

| 경로 | 역할 |
| --- | --- |
| `crates/chessembly` | 기존 DSL 파서·행마 계산과 공통 보드·액션 구현 |
| `crates/chesslayer` | 통합 게임 엔진: 전투, 카드, 유물, AI, 분기 맵·런·보상·휴식 |
| `crates/chesslayer-web` | JSON 명령/스냅샷을 노출하는 얇은 WASM 어댑터 |
| `web` | React + TypeScript + Vite UI와 Playwright E2E |

- [1차 폴리싱 변경 사항과 검증](docs/POLISHING.md)
- [기물 이미지 매핑·출처·라이선스 확인 상태](docs/ASSETS.md)
- [최종 콘텐츠 시각 통합·수동 확인 방법](docs/VISUAL_INTEGRATION.md)
- [분기 맵 생성·진행과 콘텐츠의 임시 규칙](docs/ACT_MAP.md)
- [프로토타입 임시 규칙·미구현 항목·알려진 한계](docs/PROTOTYPE.md)
- [신규 유물 6종·배치 좌표·검증과 저장 v6](docs/RELICS.md)
- [빈 덱·손패 유지·혼합 보상](docs/DECK_REWARDS.md)
- [상위 복합 기물 8종·전체 강화 트리·해금 조건](docs/COMPOSITE_PIECES.md)
- [테스트 설명·실행 순서·결과](docs/TESTING.md)
- [통합 구조와 상태 흐름](docs/ARCHITECTURE.md)
- [콘텐츠 구조와 카드·기물·캐릭터 추가 가이드](docs/content/README.md)
- [기존 Chesssembly 문법](docs/CHESSEMBLY.md)

핵심 검증 명령은 다음과 같습니다. 자세한 준비와 통과 조건은 테스트 문서에 있습니다.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --target wasm32-unknown-unknown
npm run build --prefix web
npm run test:e2e --prefix web
```
