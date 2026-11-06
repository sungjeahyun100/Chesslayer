import { PieceImage } from "../PieceImage";
import { version } from "../../package.json";

export function HomeScreen({ ready, hasSave, resume, onNavigate }: { ready: boolean; hasSave: boolean; resume: () => void; onNavigate: (screen: "choose" | "compendium" | "sandbox") => void }) {
  return (
    <main className="home">
      <section className="home-copy">
        <p className="eyebrow">A CHESS ROGUELIKE</p>
        <h1>Chesslayer</h1>
        <p className="home-version">v{version}</p>
        <p className="intro">
          길을 선택하고, 카드로 판을 바꾸세요.
          <br />
          다섯 부대가 펼치는 네 층의 원정.
        </p>
        <button
          className="primary large"
          disabled={!ready}
          onClick={() => onNavigate("choose")}
        >
          {ready ? "새 게임 시작" : "엔진 불러오는 중…"} <span>↗</span>
        </button>
        {ready && hasSave && (
          <button className="resume-button" onClick={resume}>
            저장된 원정 이어하기 →
          </button>
        )}
        <button className="resume-button" disabled={!ready} onClick={() => onNavigate("compendium")}>도감 보기 →</button>
        <button className="resume-button" disabled={!ready} onClick={() => onNavigate("sandbox")}>기물 테스트장 →</button>
        <div className="home-facts">
          <span>갈라지는 경로</span>
          <span>선택하는 보상</span>
          <span>마지막 로얄</span>
        </div>
      </section>
      <div className="hero-art" aria-hidden="true">
        <div className="hero-grid">
          {Array.from({ length: 64 }, (_, i) => (
            <i
              key={i}
              className={(i + Math.floor(i / 8)) % 2 ? "dark" : ""}
            />
          ))}
        </div>
        <span className="hero-king">
          <PieceImage pieceKey="king" />
        </span>
        <span className="hero-knight">
          <PieceImage pieceKey="knight" owner={1} />
        </span>
        <span className="hero-caption">EVERY POSITION TELLS A STORY.</span>
      </div>
      <div className="home-foot">
        CHESSLAYER v{version} <span>길을 선택하고, 왕관을 되찾으세요.</span>
      </div>
    </main>
  );
}
