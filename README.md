# Chesslayer · v0.3.0

**5개 부대·4층 플레이 프로토타입.** [구현 상태·임시 규칙·미정 항목·검증](docs/V03_PROTOTYPE.md)에 이번 버전을 정리했습니다. 기존 v0.2.x 문서는 해당 버전의 기록이며, 충돌 시 v0.3.0 안내를 따릅니다.

템페스트 5종·바운싱 4종의 휴식 강화, 전투 승격, 저장 연결은 [구현 안내](docs/VARIANTS.md)에 있습니다.
나이트라이더·유니콘·바운싱 나이트라이더·구행·텐쿠·추기경의 행마와 강화 경로는 [고급 행마 안내](docs/ADVANCED_MOVEMENT.md)에 있습니다.
행마의 신·바운싱 템페스트·태풍 8종의 전체 강화 트리와 해금 조건은 [상위 복합 기물 안내](docs/COMPOSITE_PIECES.md)에 있습니다.

Rust 엔진이 실제 게임 상태를 관리하는 브라우저 체스 로그라이크입니다. React 화면은 Rust/WASM이 반환한 기물·합법 수·카드 대상을 표시하고 플레이어의 명령을 전달합니다.

**부대·해금된 시작 층 선택 → 15단계 분기 맵 → 전투·보상·휴식·보물·식당 → 층 보스 격파 → 부대/다음 층 영구 해금 → 새 원정**을 플레이할 수 있습니다. 수호자는 기본 해금이며 선봉대·이단·포병·특수부대는 각각 1~4층 보스 격파로 해금됩니다. 전투 엔진·체섬블리·수호자의 기존 콘텐츠를 재사용합니다. 로얄 직접 포획과 호신용품 반격으로 승리하며, 포병은 창고 전멸 시 패배합니다. 기물 사망·강화·허기·정신 상태·전투불능 및 탄약은 해당 런 안에서 이어집니다.

## 실행

준비: Rust 1.85 이상, Node.js 20.19+ 또는 22.12+, npm, wasm-pack. 검증 환경은 Rust 1.99.0 / Node 22.20.0입니다. 이 환경에서는 Cargo release 빌드와 wasm-bindgen 0.2.126으로 WASM을 생성했습니다.

저장소 루트에서 실행합니다.

```sh
rustup target add wasm32-unknown-unknown
# wasm-pack이 없다면 한 번 설치
cargo install wasm-pack --locked

npm ci --prefix web
npm run wasm --prefix web
npm run dev --prefix web
```

브라우저에서 **http://127.0.0.1:5173**을 엽니다. `새 게임 시작 → 부대·층 선택 → 런 시작`을 누르세요. 종료하려면 서버 터미널에서 Ctrl+C를 누릅니다.

- 지도 아래쪽의 세 시작점 중 하나를 선택합니다. 초록색 노드만 이동할 수 있습니다.
- 백 기물 클릭 → 표시된 합법적인 칸 클릭으로 이동합니다.
- 전투 중 적 기물 클릭 → 현재 보드에서 가능한 행마와 기물 정보를 조회합니다. 파란 원은 빈 도착 칸, 빨간 테두리는 포획 대상, 점선은 계산 과정에서 확인된 경유 경로입니다. 다시 클릭하거나 빈 칸·Esc를 누르면 닫힙니다. 조회는 행동이나 저장 기록을 변경하지 않습니다.
- 대상이 필요한 카드 클릭 → 표시된 기물을 클릭합니다. Esc로 선택을 취소합니다.
- 가속 유물을 보유한 부대는 두 행동 사이에도 카드를 사용할 수 있습니다.
- 행동을 모두 쓰거나 `턴 종료`를 누르면 적이 자동으로 행동합니다.
- 일반 전투 승리 후 카드·유물 보상 3개 중 하나를 고르거나 건너뛸 수 있습니다. 현재 위치에서 연결된 다음 노드를 선택하세요.
- 엘리트 승리 후에는 카드 보상과 아군 기물 강화 기회 1회를 받습니다. 보물에서는 유물을 받습니다. 휴식에서는 보드의 아군 기물 또는 초기 빈칸을 선택해 강화·부활을 미리 보고 확정합니다. 노드당 한 번 행동하며 그냥 출발할 수도 있습니다.
- 우측 상단 `원정 현황`에서 보유 기물과 강화 내역, 다음 전투 진입 배치, 보유 유물을 확인할 수 있습니다.
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

- 수호자 기존 콘텐츠 유지, 선봉대 기물/강화/전용 카드, 이단 6계열/강화/시체/상태/카드/유물/개정 보스.
- 포병 탄약·창고·폭발·사격·전술 카드와 특수부대 최소 편성.
- 4층·15단계 진행, 보스 격파 영구 해금, 부대별 보상, 고해성사·식당, 저장/복원.
- 상태 표시, 능력 대상 선택, 부대별 강화, 탄약, 보스 해금 안내를 기존 UI에 통합.

미정 기물/능력·배치·명중률 규격 등에 최소 임시 규칙을 적용했습니다. 부족의 독재자는 미구현이며, 특수부대와 포병 상세 콘텐츠는 기획 확정 후 교체해야 합니다. [구현/임시 규칙 보고서](docs/V03_PROTOTYPE.md)를 확인하세요. 플레이 확인 시 부대 선택 화면의 `개발자 모드 · 전체 해금`을 누르면 5개 부대와 4개 층이 바로 열리고 새로고침 후에도 유지됩니다. 진행 중인 원정 저장은 바꾸지 않습니다.

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
- [이전 버전의 수호자 유물 8종·배치 좌표·검증](docs/RELICS.md)
- [빈 덱·손패 유지·혼합 보상](docs/DECK_REWARDS.md)
- [상위 복합 기물 8종·전체 강화 트리·해금 조건](docs/COMPOSITE_PIECES.md)
- [테스트 설명·실행 순서·결과](docs/TESTING.md)
- [통합 구조와 상태 흐름](docs/ARCHITECTURE.md)
- [콘텐츠 구조와 카드·기물·부대 추가 가이드](docs/content/README.md)
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
