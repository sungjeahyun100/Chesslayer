import { useState } from "react";
import type { Snapshot } from "../types";
import { PieceImage } from "../PieceImage";
import { floorLabel } from "../progress";
import { version } from "../../package.json";

export type AppScreen = "home" | "choose" | "game" | "compendium" | "sandbox";

export function AppChrome({ screen, state, onHome }: { screen: AppScreen; state: Snapshot | null; onHome: () => void }) {
  const [help, setHelp] = useState(false);
  return <>
    <header className="topbar">
      <a
        className="brand"
        href="#"
        onClick={(e) => {
          e.preventDefault();
          if (screen !== "game") onHome();
        }}
      >
        <PieceImage pieceKey="knight" owner={1} /> CHESSLAYER
      </a>
      <div className="top-right">
        <span className="edition">v{version} / {screen === "sandbox" ? "SANDBOX" : screen === "game" && state ? floorLabel(state.floor, state.final_battle) : "EXPEDITION"}</span>
        {screen !== "sandbox" && <button
          className="text-button"
          onClick={() => setHelp(!help)}
          aria-expanded={help}
        >
          플레이 안내 {help ? "−" : "+"}
        </button>}
      </div>
    </header>
    {help && screen !== "sandbox" && (
      <section className="guide">
        <strong>체크메이트 대신, 로얄을 포획하세요.</strong>
        <p>
          백 기물을 클릭하면 이동 가능한 칸이 표시됩니다. 카드 사용은 이동
          횟수를 쓰지 않습니다. 이동 행동을 모두 마치면 적이 행동합니다.
          ★는 로얄, ◇는 가호입니다.
        </p>
        <p>전투 보드에서 칸을 우클릭하면 표식을 남기고, 우클릭한 채 다른 칸으로 드래그하면 화살표를 그립니다. 같은 표식이나 화살표를 반복하면 지워집니다.</p>
        <p>
          맵에서 이어진 다음 노드를 선택하세요. 승리한 진형은 다음 전투로
          이어집니다. 적의 시작 칸과 겹친 아군은 역습으로 사라집니다. 로얄을
          잃으면 원정이 끝납니다. 포병대대는 모든 창고를 잃으면 패배합니다. 진행은 이 브라우저에 자동 저장됩니다.
        </p>
        <p>
          소멸: 사용 후 이번 전투에서 제외 · 보존: 턴 종료에도 손패 유지 ·
          선천성: 첫 손패 우선 · 휘발성: 쓰지 않고 턴 종료 시 소멸. 체크
          제한·캐슬링·앙파상은 없습니다. 폰 계열은 최종 랭크로 이동할 때
          승격할 기물을 선택합니다.
        </p>
        <p>기물을 선택하면 행마와 상태를 확인할 수 있습니다. 사격·제물 능력은 기물 정보에서 능력을 선택한 뒤 표시된 대상 칸을 누르세요. 식재료를 포획하면 공격자는 제자리에 남고 대상은 시체가 됩니다. 시체는 양 진영이 포획할 수 있습니다.</p>
      </section>
    )}
  </>;
}
