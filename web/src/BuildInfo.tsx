import { version } from "../package.json";

const branch = import.meta.env.VITE_BUILD_BRANCH || "local";
const sha = import.meta.env.VITE_BUILD_SHA || "local";

export function BuildInfo() {
  return <span className="build-info" aria-label={`빌드 정보: 브랜치 ${branch}, 버전 ${version}, 커밋 ${sha}`}>
    <span>{branch}</span><span>v{version}</span><code>{sha}</code>
  </span>;
}
