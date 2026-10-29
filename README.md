# Chesslayer · v0.3.0

Rust/WASM 전투 엔진과 React UI로 플레이하는 체스 로그라이크 프로토타입입니다. **수호자 → 선봉대 → 이단 → 포병 → 특수부대**를 해금하며, 각 부대는 시작 기물·덱·유물·강화와 서로 다른 전투 규칙을 사용합니다.

각 정규 층은 보스 포함 **15단계**입니다. 보스를 격파하면 다음 층과 해당 부대를 해금하고 런을 마칩니다. 새 런에서 부대와 열린 층을 선택합니다. 4층 이후에는 부족의 독재자와의 별도 결전이 열립니다. 해금 기록과 진행 중인 원정은 브라우저에 각각 저장됩니다.

[구현 범위·임시 규칙·검증 기록](docs/V030_PROTOTYPE.md)에서 v0.3.0에 적용한 결정과 원문 미정 항목을 확인할 수 있습니다. 저장 v14를 사용하며 이전 규칙의 v13 원정은 재생하지 않습니다.

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

브라우저에서 **http://127.0.0.1:5173**을 엽니다. `새 원정 → 부대와 층 선택 → 런 시작`을 누르세요. 종료하려면 서버 터미널에서 Ctrl+C를 누릅니다.

- 지도 아래쪽의 세 시작점 중 하나를 선택합니다. 초록색 노드만 이동할 수 있습니다.
- 백 기물 클릭 → 표시된 합법적인 칸 클릭으로 이동합니다.
- 전투 중 적 기물 클릭 → 현재 보드에서 가능한 행마와 기물 정보를 조회합니다. 파란 원은 빈 도착 칸, 빨간 테두리는 포획 대상, 점선은 계산 과정에서 확인된 경유 경로입니다. 다시 클릭하거나 빈 칸·Esc를 누르면 닫힙니다. 조회는 행동이나 저장 기록을 변경하지 않습니다.
- 대상이 필요한 카드 클릭 → 표시된 기물을 클릭합니다. Esc로 선택을 취소합니다.
- 선봉대는 기본 두 행동 사이에도 카드를 사용할 수 있습니다. 기물별 민첩은 추가 행동을 부여합니다.
- 사격/제물은 기물을 선택한 뒤 특수 능력 버튼으로 사용합니다. 시체 포획과 후속 대상 선택은 보드에서 처리합니다.
- 행동을 모두 쓰거나 `턴 종료`를 누르면 적이 자동으로 행동합니다.
- 일반 전투 승리 후 카드·유물 보상 3개 중 하나를 고르거나 건너뛸 수 있습니다. 현재 위치에서 연결된 다음 노드를 선택하세요.
- 엘리트·보스 승리 후에는 카드 보상과 아군 기물 강화 기회 1회를 받습니다. 보물에서는 유물을 받습니다. 휴식에서는 보드의 아군 기물 또는 초기 빈칸을 선택해 강화·부활을 미리 보고 확정합니다. 노드당 한 번 행동하며 그냥 출발할 수도 있습니다.
- 우측 상단 `원정 현황`에서 보유 카드, 다음 전투 진입 배치, 보유 유물을 확인할 수 있습니다. 배치된 기물을 선택하면 강화 내역이 표시됩니다.
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

5개 플레이 부대, 4층·15단계 분기 지도, 별도 최종 결전, 전용 보스와 일반/정예 전투, 카드·유물 보상, 휴식 강화·부활·고해성사, 식당과 상점이 연결되어 있습니다. 기물 82종, 카드 34종, 유물 21종을 등록했으며 신규 행마도 독립된 기물 정의 파일을 사용합니다.

공통 명중률과 사격, 민첩·약화, 시체·식재료·황충의 축복·허기·광기·죄책감, 포병 탄약과 창고 폭발을 실제 전투에 적용합니다. 해금과 명령 재생 저장을 분리해 새 런과 새로고침에도 부대 선택 기록이 유지됩니다. 자세한 미정 규칙과 검증은 [v0.3.0 구현 기록](docs/V030_PROTOTYPE.md)을 참고하세요.

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
- [신규 유물 5종·배치 좌표·검증과 저장 v13](docs/RELICS.md)
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
