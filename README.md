# Chesslayer · 첫 번째 플레이 가능 프로토타입

Rust 엔진이 실제 게임 상태를 관리하는 브라우저 체스 로그라이크입니다. React 화면은 Rust/WASM이 반환한 기물·합법 수·카드 대상을 표시하고 플레이어의 명령을 전달합니다.

**캐릭터 선택 → 전체 분기 맵 → 노드 선택 → 전투·보상·휴식·보물 → 맵 복귀 → 마하라자**를 플레이할 수 있습니다. 런 시작 시 무작위로 생성한 제1막 지도는 원정이 끝날 때까지 유지됩니다. 적의 지정 로얄을 직접 포획하면 승리하고, 킹을 잃으면 원정이 끝납니다. 생존 진형·강화·지속 상태·보상 카드는 다음 전투로 이어집니다.

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
- 대상이 필요한 카드 클릭 → 표시된 기물을 클릭합니다. Esc로 선택을 취소합니다.
- 가속 캐릭터는 두 행동 사이에도 카드를 사용할 수 있습니다.
- 행동을 모두 쓰거나 `턴 종료`를 누르면 적이 자동으로 행동합니다.
- 승리 후 카드 보상을 고르면 지도로 돌아옵니다. 현재 위치에서 연결된 다음 노드를 선택하세요.
- 보물에서는 유물을 받고, 엘리트 승리 시 카드와 유물을 얻습니다. 휴식에서는 강화·부활·그냥 출발 중 하나를 선택합니다.
- 전투 중에도 `전체 지도 보기`로 남은 경로를 확인할 수 있습니다.
- 새로고침하면 원정이 초기화됩니다. 저장 기능은 없습니다.

Rust 코드를 바꾼 뒤에는 `npm run wasm --prefix web`을 다시 실행하고 브라우저를 새로고침하세요. React/CSS는 Vite가 즉시 반영합니다. WASM 생성물은 Git에 포함하지 않습니다.

프로덕션 빌드도 같은 WASM 엔진을 사용합니다.

```sh
npm run wasm --prefix web
npm run build --prefix web
npm run preview --prefix web
```

정적 배포 산출물은 `web/dist/`입니다. `index.html`을 파일로 직접 여는 대신 HTTP 서버를 사용하세요. 첫 WASM 빌드는 wasm-bindgen CLI 다운로드 때문에 네트워크가 필요할 수 있습니다. 다운로드가 제한된 환경에서는 Cargo.lock과 같은 버전의 CLI를 설치한 뒤 실행합니다: `cargo install wasm-bindgen-cli --version 0.2.126 --locked`.

## 구현 범위

- 표준 백 진형, 클릭 이동, 이동 애니메이션, 양측 로얄·가호 표시
- 일반 전투 4종, 엘리트 1종, 퀸+나이트 행마의 아마존 보스, 시드로 재현 가능한 휴리스틱 적 AI
- 7단계·18~20개 노드의 전체 맵, 세 시작점, 반복 분기와 합류, 현재 위치·방문·선택 경로 표시
- Rust에서 검증하는 경로 선택과 노드 완료, 보물·엘리트 유물 보상
- 마나, 5장 드로우, 손패 10장 제한, 재셔플, 소멸·보존·선천성·휘발성
- 청바지, 가호, 알레킨의 총과 가호의 휘발성 변형
- 시작 유물이 다른 캐릭터 3종, 가속·소용돌이·호신용품
- 기물 위치·사망·강화·지속 상태와 보상 카드 계승, 역습
- 휴식의 폰 강화와 초기 위치 부활, 승리·패배·새 원정 화면
- 한국어 기본 오류 메시지와 게임 플레이용 전투 기록, 모바일 레이아웃

디버그 화면이나 상태 조작 API는 만들지 않았습니다. 2층, 상점·랜덤 이벤트, 계정, 서버 저장, 멀티플레이, 학습형 AI, 정식 그래픽은 이번 범위에 없습니다.

## 코드와 문서

| 경로 | 역할 |
| --- | --- |
| `crates/chessembly` | 기존 DSL 파서·행마 계산과 공통 보드·액션 구현 |
| `crates/chesslayer` | 통합 게임 엔진: 전투, 카드, 유물, AI, 분기 맵·런·보상·휴식 |
| `crates/chesslayer-web` | JSON 명령/스냅샷을 노출하는 얇은 WASM 어댑터 |
| `web` | React + TypeScript + Vite UI와 Playwright E2E |

- [분기 맵 생성·진행과 콘텐츠의 임시 규칙](docs/ACT_MAP.md)
- [프로토타입 임시 규칙·미구현 항목·알려진 한계](docs/PROTOTYPE.md)
- [테스트 설명·실행 순서·결과](docs/TESTING.md)
- [통합 구조와 상태 흐름](docs/ARCHITECTURE.md)
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
