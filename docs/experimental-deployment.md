# Chesslayer 실험 배포

`version/v1.0.0`의 전체 게임과 전투 시뮬레이터를 `chesslayer-test` Cloudflare Pages 프로젝트에 배포한다. `develop`의 GitHub Pages 워크플로는 별도로 유지된다. 게임 규칙과 콘텐츠에는 배포용 변경이 없다.

## 구조와 주소

- `.github/workflows/experimental.yml`은 `version/v*`, `hotfix/**` 브랜치 Push와 수동 실행에서 기존 Rust/WASM, Vite 빌드를 수행하고 `web/dist`를 Pages에 Direct Upload한다.
- Pages의 **production branch를 `version/v1.0.0`으로 설정**한다. 성공적인 첫 배포 후 전체 게임은 `https://chesslayer-test.pages.dev/`, 전투 시뮬레이터는 `https://chesslayer-test.pages.dev/battle-simulator.html`에서 실행된다. 프로젝트 이름이 이미 사용 중이면 Cloudflare가 실제로 발급한 `pages.dev` 호스트명을 사용한다.
- 다른 `version/v*` 또는 `hotfix/**` 브랜치의 Push는 같은 프로젝트의 preview 배포를 만든다. 실제 preview URL은 Pages의 Deployments 화면에서 확인한다.
- 두 화면의 빌드 정보는 GitHub Actions가 설정하는 `VITE_BUILD_BRANCH`, `VITE_BUILD_SHA`와 `web/package.json`의 버전으로 표시된다. 실험 사이트의 브라우저 저장소는 기존 GitHub Pages 사이트와 출처가 달라 서로 분리된다.

## 최초 설정 순서

1. 기존 Cloudflare 계정에서 **Direct Upload** Pages 프로젝트 `chesslayer-test`를 생성하고 production branch를 `version/v1.0.0`으로 지정한다. Wrangler CLI를 사용하면 `npx wrangler login` 후 `npx wrangler pages project create chesslayer-test --production-branch=version/v1.0.0`이다. Git 연동 프로젝트를 함께 만들지 않는다. GitHub Actions가 빌드와 업로드를 담당한다.
2. Cloudflare Zero Trust의 무료 플랜을 선택한다. Zero Trust > Integrations > Identity providers에서 **One-time PIN**을 추가한다. 테스트에 허용할 이메일 주소를 Access의 Allow 정책에 등록한다. 테스터는 이메일로 받은 코드로 로그인하며 ChatGPT 계정은 필요하지 않다.
3. **첫 게임 배포 전에** Pages 프로젝트 Settings > General에서 `Enable access policy`를 켜고, 생성된 Access 애플리케이션의 Public hostname에서 와일드카드 `*`를 제거해 `chesslayer-test.pages.dev` 원 도메인을 보호한다. 다시 Pages Settings > General에서 `Enable access policy`를 켜 preview 주소 `*.chesslayer-test.pages.dev`도 보호한다. 두 Access 애플리케이션 모두 허용 이메일 정책을 확인한다. 기본 Pages 주소와 preview 주소는 각각 별도로 보호해야 한다.
4. Cloudflare API Tokens에서 이 계정에 대해 **Account > Cloudflare Pages > Edit** 권한을 가진 토큰을 생성한다. 저장소 GitHub Settings > Secrets and variables > Actions에 repository secret `CLOUDFLARE_API_TOKEN`(토큰 값), `CLOUDFLARE_ACCOUNT_ID`(Cloudflare 계정 ID)를 등록한다. 토큰 값은 저장소 파일에 넣지 않는다.
5. 이 변경을 `version/v1.0.0`에 Push한다. GitHub Actions의 **Deploy Chesslayer experiment** 실행이 성공했는지 확인한다. 로그인하지 않은 브라우저에서 전체 게임, 시뮬레이터, preview 주소가 Access 인증을 요구하는지 확인하고, 허용된 이메일로 로그인해 게임과 전투를 실행한다. 인증 전 게임 파일이나 `.wasm` 파일에 직접 접근할 수 없어야 한다.
6. 실제 URL을 확인한 뒤 [게임 테스트 센터](https://chatgpt.com/space/page_8f7592d6930c8191bff5e8970add253f)의 게임 실행 섹션에 전체 게임과 시뮬레이터 링크를 등록한다. Space 페이지의 접근 권한은 Pages Access 권한과 별도로 관리한다.

Cloudflare의 Direct Upload, Access 설정 절차는 [Pages CI 안내](https://developers.cloudflare.com/pages/how-to/use-direct-upload-with-continuous-integration/), [Pages Access 주의사항](https://developers.cloudflare.com/pages/platform/known-issues/#enable-access-on-your-pagesdev-domain), [One-time PIN 안내](https://developers.cloudflare.com/cloudflare-one/integrations/identity-providers/one-time-pin/)를 따른다.

## 다음 버전 또는 hotfix로 전환

새 브랜치를 Push하면 먼저 Access로 보호되는 preview URL에서 검증할 수 있다. 이 preview를 기본 `chesslayer-test.pages.dev` 주소로 올릴 때는 Direct Upload 프로젝트의 production branch를 새 브랜치 이름으로 변경해야 한다. Cloudflare는 Direct Upload의 production branch 변경을 대시보드에서 지원하지 않으므로 [Update Project API](https://developers.cloudflare.com/pages/configuration/branch-build-controls/#production-branch-control)에 `{"production_branch":"version/vX.X.X"}` 또는 `{"production_branch":"hotfix/이름"}`을 보내고, 해당 브랜치의 배포 실행을 다시 실행하거나 새 Push를 만든다. 이전 브랜치의 preview는 Cloudflare의 배포 보존 기간 내에 별도 URL에서 확인할 수 있다. 테스트 센터의 기본 실행 링크는 도메인이 같으므로 유지된다.

## Sites 및 Space 검토

ChatGPT Space의 Page는 링크와 문서 중심이다. 기존 다중 파일 Vite/WASM 결과물을 Page 자체에서 실행하는 기능은 확인되지 않았다. ChatGPT Sites는 호환되는 정적 프로젝트의 결과물을 호스팅할 수 있지만, 이 저장소의 GitHub Actions Push를 Sites에 자동 배포하는 지원 경로는 확인되지 않았다. 이 실험 환경은 외부 Pages에서 실행하고 Space에서는 실행 링크와 테스트 기록을 제공한다. [Sites 프로젝트 안내](https://learn.chatgpt.com/docs/sites/creating-a-site), [Space Page 안내](https://learn.chatgpt.com/docs/space/pages)
