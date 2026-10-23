# 기물 SVG 매핑과 출처 확인 — 2026-10-23

## 제공 자료 조사

저장소의 실제 경로는 `chess_play/`다. `180x/svg/`, `180x/png/`, `1000x/png/`, `decoded-svg/manifest.json`, 보드 PNG가 있다. 조사 시 SVG 528개·PNG 536개를 확인했다. 저장소 안에서 `chess_play.zip` 또는 다른 zip은 발견되지 않았다. 이미 풀린 자료를 사용했고 원본 폴더나 압축파일을 삭제·수정하지 않았다.

기존 Git 추적 파일에는 기물 SVG가 없었다. 기존 UI는 Rust에서 받은 유니코드 glyph/badge를 그렸다. 제공 폴더의 `decoded-svg`는 manifest상 `project.json`에서 추출된 242개 이미지이며 추가 파일도 있다. manifest는 스프라이트·코스튬·원본 파일 해시 이름을 제공하지만 저작자·라이선스 필드는 없다.

전체 후보를 로컬 SVG 썸네일로 렌더링해 실제 모양을 비교했다. 기본 6종은 Lichess의 Cburnett 계열 외형인 추출 SVG를 우선했다. 예를 들어 `013__chess__r...svg`는 룩이 아니라 캐릭터 그림이므로 사용하지 않았다. 추가 기물은 의미와 형태를 함께 확인했다.

## 실제 포함한 16개 이미지

위치: `web/src/assets/pieces/`. 백/흑 각각 8종만 포함한다. 사용하지 않는 색상·고해상도 PNG·보드·다른 변형 기물은 복사하지 않았다.

| 종류 ID / key | 백 원본 | 흑 원본 | 적용 판단 |
| --- | --- | --- | --- |
| 1 / pawn | decoded 041 / pw | decoded 181 / pb | 일반 폰 형태 |
| 2 / knight | decoded 173 / knight | decoded 050 / knight black | 일반 나이트, camel knight 등과 구별 |
| 3 / bishop | decoded 026 / bw | decoded 118 / bb | 일반 비숍, 튕김 비숍·날개 비숍과 구별 |
| 4 / rook | decoded 042 / rw | decoded 188 / rb | 일반 룩 |
| 5 / queen | decoded 187 / qw | decoded 185 / qb | 일반 퀸 |
| 6 / king | decoded 172 / king | decoded 169 / kb | 일반 킹 |
| 7 / amazon | 180x/svg/WHITE/Amazon_W_neo.svg | 180x/svg/BLACK/Amazon_B_neo.svg | 퀸+나이트 복합 기물 전용 Neo 이미지 |
| 8 / tempest_pawn | 선택한 pawn-white.svg 기반 | 선택한 pawn-black.svg 기반 | 폰 실루엣에 직접 작성한 두 SVG 바람 곡선을 추가 |

`tempest-queen`은 템페스트 폰이 아니고 `Sergeant`도 다른 기물이므로 서로 대체하지 않았다. 템페스트 폰은 일반 폰의 확장이라는 현재 규칙을 이미지에도 유지한다.

기본 6종은 추출 과정에서 잘린 SVG에 45×45 투명 프레임과 공통 기준선 40을 복원했다. 내부 path는 변경하지 않았다. 아마존은 원본 180×180 SVG를 그대로 사용한다. 화면에서는 `object-fit: contain`과 공통 크기로 표시한다. 프리뷰도 SVG를 사용하므로 확대 시 선명하다.

각 파일의 정확한 원본 경로, SHA-256, 추출 manifest 항목, 수정 내용, 확인 상태는 같은 폴더의 `provenance.json`에 있다. 이 기록은 출처 추적용이며 라이선스 부여 문서가 아니다.

## 공통 조회와 누락 처리

Rust 기물 ID → 정의의 고유 key → `PieceImage.tsx::pieceImage(key, owner)` → `<key>-white.svg` 또는 `<key>-black.svg`로 연결한다. 모든 화면에서 `PieceImage`를 사용한다. 전투, 쉼터, 부활·강화 미리보기, 테크트리, 기물 정보, 카드 기물 그림, 홈과 맵의 기물 아이콘도 같은 경로다.

Vite의 명시된 리소스 폴더 import glob만 번들에 포함한다. 상태는 로얄·가호 강조로 표현하고 기물 종류 이미지를 임의로 바꾸지 않는다. 누락 key/진영 또는 로드 실패는 `이미지 누락: key/owner` 표식을 보여준다. 유니코드 대체는 없다. 콘텐츠 등록 테스트는 모든 정의의 백·흑 SVG 존재 여부도 검사한다.

## 라이선스 확인 상태

- 제공 폴더에는 사용 권한을 설명하는 LICENSE/README가 없었다. 추출 manifest에도 저작권·허가 정보가 없다.
- [Lichess 공식 COPYING.md](https://github.com/lichess-org/lila/blob/master/COPYING.md)는 upstream 세트별 저작자와 라이선스를 기록한다. 그러나 이 저장소의 추출본이 그 원본에서 어떤 과정을 거쳐 왔는지까지 확인하지 못했으므로, 외형 유사성만으로 해당 허가를 이 파일들에 적용하지 않았다.
- Neo의 파일명과 스타일은 Chess.com 계열을 가리키지만 원 저작자·제공 경로·재배포 허가 증빙이 없다. [Chess.com 공식 약관](https://www.chess.com/legal/user-agreement)은 개별 제공 파일의 사용 허가 증빙을 대신하지 않는다.
- 따라서 현재 16개 파일 모두 원본 권리 관계는 **미확인**으로 기록한다. 템페스트의 추가 벡터 선도 원본 폰의 권리 문제를 없애지 않는다. 공개 배포 시에는 제공자의 허가 자료를 확인하거나 출처가 확실한 이미지로 교체해야 한다.

이번 작업은 요청한 로컬 에셋 적용을 완료했으며, 코드 저장소의 라이선스가 제공 이미지의 권리까지 포함한다고 가정하지 않는다. 외부 이미지 다운로드는 하지 않았다.

## 템페스트·바운싱 확장 (현재 총 32개)

이번에는 기존 후보 `decoded-svg/025`, `126`, `193`, `194`를 실제로 렌더링해 비교했다. 025/126은 튕김 비숍 전용 형태, 193/194는 템페스트 퀸 전용 형태여서 해당 종류에 사용했다. 기존 템페스트 폰 2개는 유지하고 8종의 백·흑 16개를 추가했다.

| key (백·흑 공통 파일명) | 선택한 원본 / 수정 |
| --- | --- |
| tempest_pawn | 기존 폰 기반 바람 곡선 이미지 유지 |
| tempest_rook | 기존 rook-white/black에 같은 녹색 바람 곡선 추가 |
| tempest_bishop | 기존 bishop-white/black에 같은 녹색 바람 곡선 추가 |
| tempest_knight | 기존 knight-white/black에 같은 녹색 바람 곡선 추가 |
| tempest_queen | decoded 194 백 / 193 흑, 뾰족한 전용 왕관. 45×45 프레임·기준선 40 복원, 내부 path 유지 |
| bouncing_pawn | 기존 pawn-white/black에 황갈색 벽·반사 화살표를 양쪽에 추가 |
| bouncing_rook | 기존 rook-white/black에 같은 반사 화살표 추가 |
| bouncing_bishop | decoded 126 백 / 025 흑, 굽은 받침의 전용 형태. 45×45 프레임·기준선 40 복원, 내부 path 유지 |
| bouncing_queen | 기존 queen-white/black에 같은 반사 화살표 추가 |

확장 파일은 모두 `web/src/assets/pieces/<key>-white.svg`, `<key>-black.svg`로 매핑한다. 모든 화면에서 기존 `PieceImage`를 사용하며 승격 선택에도 같은 컴포넌트를 쓴다. 원본 `chess_play/`는 수정하지 않았다. 유니코드 대체나 외부 이미지 요청은 없다.

`provenance.json`에 추가 파일 각각의 정확한 원본 경로·원본 SHA-256·변형 내용을 추가했다. 추출 SVG와 파생 이미지의 원본 저작권·재배포 라이선스는 기존과 같이 **미확인**이며 새로 그린 장식이 그 상태를 변경하지 않는다. 앞 절의 16개는 1차 폴리싱 당시 수량이고 현재는 32개다.

## 구행 방향 이미지 (현재 총 34개)

구행의 기물 ID는 17, key는 `hook`이다. 사용자가 지정한 원본 `chess_play/decoded-svg/149__kind_of_chess_piece_white4__DCx_U2xhaGga2BHX6T4I8st1a1XszSVJv8X829uwDynl4cANjovliBTCGfaGA1z0NEWKz-bZokKbr4AR__8f2859e7dfdefd3d0331bb133a86e038.svg`를 사용한다. 원본은 쇼기 구행의 글자와 위를 향하는 오각형을 담고 있으며, SVG 내부에 PNG가 포함되어 있다. 처음 적용했던 임시 룩 이미지는 이 원본으로 교체했다.

`hook-white.svg`는 아군용으로 위를 향하고, `hook-black.svg`는 같은 그림 전체를 중심 기준으로 **180도 회전**하여 적군용으로 아래를 향한다. 글자를 거울처럼 반사하지 않으며 색은 바꾸지 않는다. 파일명의 white/black은 기존 진영 조회 키이고, 실제 구분은 쇼기처럼 방향으로 한다. 두 파일 모두 45×45 프레임 안에 같은 크기(높이 36.5)로 중앙 정렬했다. 내부 그림과 원본 파일은 수정하지 않았다. 전투와 지도 보스 아이콘 모두 공통 key 조회를 사용한다.

`provenance.json`에 원본 경로·SHA-256·manifest 항목과 회전·크기 조정을 기록했다. 원본 라이선스의 확인 상태는 기존과 같다.

## 고급 행마 5종 이미지 (현재 총 44개)

나이트라이더·유니콘·바운싱 나이트라이더는 기존 백·흑 나이트 벡터에 서로 다른 이동 표식을, 텐쿠는 비숍 벡터에 대각 꺾임 표식을, 추기경은 바운싱 비숍 벡터에 반복 반사 표식을 더했다. 구행의 기존 백·흑 이미지는 그대로 사용한다. `provenance.json`에 각 파생 파일의 원본 파일명과 SHA-256을 기록했다. 10개 새 SVG 모두 XML 파싱과 실제 브라우저 이미지 로드를 확인했다.

## 최종 콘텐츠 아트 — 2026-10-24

최종 UI 단계에서 새 기물 13종의 백·흑 26개 파일을 **직접 작성한 벡터**로 교체했다. 위의 파생 이미지 기록은 당시 중간 단계의 이력이며 현재 파일에는 적용되지 않는다. 나이트라이더는 말과 연속 궤적, 유니콘은 뿔, 바운싱 나이트라이더는 반사 모서리, 텐쿠는 천구 고리, 추기경은 양쪽 반사 궤적, 행마의 신은 교차 궤적과 관, 바운싱 템페스트 3종은 바람·반사 표시, 태풍 4종은 굵은 이중 소용돌이를 사용한다. 백은 밝은 몸체와 어두운 윤곽, 흑은 어두운 몸체와 밝은 윤곽으로 작은 보드 칸의 대비를 유지한다. 파일별 SHA-256과 `original` 표시를 `web/src/assets/pieces/provenance.json`에 기록했다. 구행은 기존 방향별 파일을 유지한다.

`web/src/assets/cards/`에는 새 카드 5종(`pawn_storm`, `reverse_jeans`, `limiter_release`, `temporary_upgrade`, `sacred_domain`)의 독립 장면 SVG를 배치했다. 전진 대열, 뒤집힌 청바지와 균열, 봉인 해제와 세 말의 잔상, 마법진 속 진화, 비숍 중심의 보드 가호를 각각 표현한다. 기존 카드 4종도 같은 크기의 자체 제작 아트를 추가해 글리프 대체를 없앴다. `web/src/assets/relics/`에는 신규 6종(`kings_indian`, `queens_indian`, `hedgehog`, `initiative`, `ice_cream`, `hard_disk`)과 기존 시작 유물 3종의 자체 제작 SVG가 있다. 세 진형 유물은 같은 책·색·프레임을 공유하고 표지 문양으로 구분한다. 모든 카드·유물 SVG는 이 작업에서 직접 작성했으며 외부 이미지를 포함하지 않는다.

UI는 카드 종류 또는 유물 ID를 그대로 파일명에 대응시킨다. 손패·혼합 보상·유물 목록·보물·캐릭터 선택에서 같은 파일을 사용한다. 기물 이미지의 기존 기본형과 구행 등은 앞 절의 재배포 권한 미확인 상태가 그대로 남는다. 공개 배포 전 해당 구형 자산의 권리 증빙 또는 자체 제작 대체가 필요하다.
