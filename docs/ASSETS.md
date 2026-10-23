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
