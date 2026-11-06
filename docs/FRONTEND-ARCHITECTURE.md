# 프론트엔드 UI 구조

이번 변경은 `version/v0.3.3`의 React 내부 구조를 정리한다. Rust 엔진, Chesssembly, 게임 콘텐츠, WASM `Command`/`Snapshot`, 저장 키와 저장 포맷은 변경하지 않는다.

## 책임과 의존 방향

```text
web/src/
  main.tsx                    React 마운트와 스타일 진입점
  app/
    App.tsx                   최상위 화면 선택, 세션과 화면 연결
    AppChrome.tsx             상단 바와 플레이 안내
    FloorOverlay.tsx          기존 층 전환 오버레이
  game/
    useGameSession.ts         WASM 초기화·시작·복원·명령·적 조회
    useRunStorage.ts          저장·해금·개발자 설정·강화 이력
    useEnemyTurn.ts           적 워커 실행과 오래된 결과 차단
    useGamePresentation.ts    확정 상태를 표시 상태로 넘기는 지점
    inputState.ts             입력 모드, reducer, 공통 입력 정책
    useGameInput.ts           사용자 의도 해석과 기존 명령 전송
  screens/
    HomeScreen.tsx            메인 메뉴
    GameScreen.tsx            원정 화면 조립, 현황·보상 오버레이 관리
    BattleScreen.tsx          전투 보드·손패·정보 패널 배치
    RestScreen.tsx            휴식 및 보상 강화 화면
    OutcomeScreen.tsx         상점·이벤트·보물·승리·패배 화면
  board/
    ChessBoard.tsx            64칸 보드와 렌더링 슬롯
    RunBoard.tsx              전투·휴식·강화 표시를 보드에 연결
    BoardPiece.tsx            기물 및 상태 배지
    TargetPreview.tsx         이동·포획·능력·위험 표시
    useBoardAnnotations.ts    우클릭 표식 및 드래그 화살표
  components/
    BattleHeading.tsx         전투·휴식 공통 제목과 턴 표시
    PieceInspector.tsx        선택한 기물의 정보와 능력 버튼
    pieceInfo.ts              기존 행마·능력 설명
    CardHand.tsx              손패 및 카드 선택 전달
    ActionHint.tsx            대상 선택 안내·확인·취소
    RelicList.tsx             유물 목록
  styles/                     책임별 스타일, 기존 cascade 순서 유지
  style.css                   순서가 명시된 CSS import 진입점
```

기존 `SquadSelection`, `ActMap`, `BattleRewards`, `LocationPanel`, `RestPanel`, `Compendium`, `PieceSandbox`, `UpgradeTree`, `CardUpgradePanel`, `PromotionPicker`, `PieceImage`, `CardFace`는 재사용한다. 기존 컴포넌트를 디렉터리 정리만을 위해 이동하거나 재작성하지 않는다.

흐름은 **화면 → 입력 컨트롤러 → 게임 세션/WASM → 확정 Snapshot → 표시 계층 → 화면**이다. 렌더링 컴포넌트는 `WebGame`을 참조하지 않는다. 테스트장은 별도의 `WebSandbox`와 기존 입력 로직을 유지하며 `ChessBoard`와 `BoardSquare`만 공유한다.

## 입력 상태

`InputState`는 `idle`, `piece`, `enemy`, `ability`, `card`, `promotion`, `rest`의 판별 유니언이다. 기물 선택 모드는 엔진의 `moves`를 이동 대상으로 사용한다. 카드 모드 안에 선택 순서, 복수 대상, 기물별 강화 경로와 현재 강화 대상을 보존하고, 능력 모드 안에 대상 목록과 hover를 보존한다. 모드 전환 시 다른 모드의 데이터가 남지 않는다.

`useGameInput`은 클릭·확인·취소를 해석한다. 대상과 이동의 합법성은 엔진의 `moves`, `targets`, `abilities`, `effect_choice`를 사용하며, 전투 판정을 재구현하지 않는다. 엔진이 명령을 거절하면 오류를 표시하고 선택을 유지한다. 명령 성공, 워커 응답, 새 원정과 복원 시 선택을 초기화한다.

`inputPolicy`는 모달·표시 대기·승격·턴·엔진 효과 선택에 따른 입력 허용을 결정한다. 적 턴에는 기존의 적 기물 정보 조회를 허용하되 플레이어 명령은 차단한다. 원정 현황을 닫으면 선택을 보존하며, 일반 Esc는 선택을 취소한다. 보상 강화에서 Esc는 보상 화면으로 돌아간다. 원정 지도·결과 화면의 명령도 `GameScreen`에서 모달과 표시 대기를 확인한다.

## 표시 계층과 향후 애니메이션

- `useGameSession.snapshot`: 엔진이 확정한 실제 상태.
- `useGamePresentation.displayState`: 현재 화면에 표시할 상태. 현재는 같은 Snapshot 참조를 넘기며, 기존 층 전환만 표시를 지연한다.
- `useGamePresentation.present(next, before, source)`: 명령 전후 상태와 명령을 받을 수 있는 애니메이션 연결 지점.
- `presentationBusy`: 표시가 끝날 때까지 사용자 입력을 제한하는 공통 연결 지점.
- `ChessBoard`: 칸 → 경로 → 기물 → 주석 → 선택적 `effects` 순서의 슬롯. 현재 DOM과 CSS 배치를 보존하며, 효과 슬롯에는 아직 구현을 넣지 않는다.

기물 표시와 엔진 상태 갱신은 별도 책임이다. 추후 표시 계층에서 이전 Snapshot을 유지한 채 보드 효과를 출력하고, 종료 시 최종 Snapshot을 표시할 수 있다. 이번 변경에는 애니메이션 큐, 타임라인, VFX registry를 추가하지 않는다.

## CSS

`style.css`의 import 순서는 기존 파일의 선언 순서다. 반응형 규칙을 일괄적으로 파일 끝에 옮기면 뒤따르던 기존 override보다 우선될 수 있으므로, 공통 반응형 규칙과 각 화면의 후속 반응형 규칙을 원래 위치에 둔다. CSS layer나 선택자 우선순위를 새로 도입하지 않는다.

구형 테스트장의 `sandbox-layout`, `sandbox-catalog`, `sandbox-board` 등 14개 클래스는 TypeScript/TSX에서 사용하지 않는 것을 확인한 뒤 관련 규칙만 제거했다. 현재 `sandbox-v2`, 공통 제목, 행동 선택 모달과 반응형 규칙은 유지한다.

## 검증

- `npm run build`: TypeScript 검사와 Vite 프로덕션 빌드 통과. Vite의 500 kB 청크 크기 경고는 남아 있다.
- `npm run test:e2e -- input-state.spec.ts ui-architecture.spec.ts`: 입력 전환 5개와 실제 WASM 저장·복원 기반 UI 회귀 6개, **11개 모두 통과**.
- 최종 `npm run test:e2e`: **54개 중 37개 통과, 17개 실패**. 변경 전 통과한 26개는 모두 통과했고, 실패한 17개 항목은 변경 전과 동일하다.
- 변경 전 전체 테스트는 **43개 중 26개 통과, 17개 실패**였다. 오래된 `new WebGame("vanguard", ...)` 해금 가정, 옛 기물/지도에 대한 고정 명령 fixture, `제1층` 문구 및 14행 지도 기대값 등이 현재 엔진과 맞지 않는다. 해당 실패를 해결하기 위해 게임 규칙이나 기존 테스트의 기대값을 바꾸지 않는다.
- 새 UI fixture는 현재 엔진의 공개 명령과 기존 개발자 강화 기능만 사용한다. 주입한 Snapshot이나 테스트 전용 엔진 API를 사용하지 않는다.
- 변경 전 소스를 별도 진입점으로 로드해 메인 메뉴·부대 선택·전투·휴식·보상·테스트장을 1440px 및 390px에서 비교했다. reduced-motion을 적용한 총 **12개 화면의 PNG가 바이트 단위로 일치**했다. 비교용 소스와 임시 테스트는 검증 후 제거했다.
- CSS AST 비교에서도 확인된 미사용 테스트장 규칙을 제외한 선택자, 선언값, 미디어 쿼리, keyframe 및 순서가 모두 일치했다.
