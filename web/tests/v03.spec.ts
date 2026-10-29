import { test,expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Snapshot,Command } from "../src/types";

test("보스 격파 → 영구 해금 → 새로고침 → 새 부대·층 시작",async({page})=>{
  const errors:string[]=[];page.on("pageerror",e=>errors.push(e.message));
  await page.emulateMedia({reducedMotion:"reduce"});await page.goto("/");
  await page.getByRole("button",{name:"새 게임 시작"}).click();
  await expect(page.locator(".squad:disabled")).toHaveCount(4);
  const runs=["guardian-1","vanguard-2","heretic-3","artillery-4"];
  for(let i=0;i<runs.length;i++){
    const save=JSON.parse(readFileSync(`../crates/chesslayer/tests/fixtures/v03-${runs[i]}.json`,"utf8"));
    while(save.commands.at(-1)?.type==="enemy_step")save.commands.pop();
    const finish:Command=save.commands.pop();
    await page.evaluate(value=>localStorage.setItem("chesslayer.run.v1",JSON.stringify(value)),save);
    await page.reload();await page.getByRole("button",{name:"저장된 원정 이어하기"}).click();
    const before=await page.evaluate(async()=>{const {default:init,WebGame}=await import("/src/wasm/chesslayer_web.js");await init();const g=WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);try{return JSON.parse(g.snapshot()) as Snapshot;}finally{g.free();}});
    expect(before.phase).toBe("combat");
    if(finish.type!=="move")throw new Error("fixture must end with a normal royal capture");
    await page.locator(".square").nth(finish.from).click();await page.locator(".square").nth(finish.to).click();
    if(finish.promotion) await page.locator(`.promotion-picker button[data-kind="${finish.promotion}"]`).click();
    await expect(page.locator("main")).toHaveAttribute("data-phase","victory");
    await expect(page.locator(".unlock-notice")).toBeVisible();
    const progress=await page.evaluate(()=>JSON.parse(localStorage.getItem("chesslayer.progress.v1")!).cleared);
    expect(progress).toBe((1<<(i+1))-1);
    await page.reload();await page.getByRole("button",{name:"새 게임 시작"}).click();
    await expect(page.locator(".squad:disabled")).toHaveCount(3-i);
    const next=["선봉대","이단 부대","포병대대","특수부대"][i];
    await page.locator(".squad").filter({has:page.getByRole("heading",{name:next,exact:true})}).click();
    const floor=Math.min(i+2,4);await page.getByRole("group",{name:"시작 층 선택"}).getByRole("button",{name:`${floor}층`,exact:true}).click();
    await page.getByRole("button",{name:"런 시작"}).click();
    await expect(page.locator(".act-map-screen")).toHaveAttribute("data-floor",String(floor));
    await page.locator(".map-node:enabled").first().click();await expect(page.locator("main")).toHaveAttribute("data-phase","combat");
    await expect(page.locator(".missing-piece")).toHaveCount(0);
    if(next==="포병대대"){await expect(page.locator(".mana-orb")).toHaveText("400");await expect(page.locator(".turn-panel")).toContainText("탄약 / 400");}
    const snapshot=await page.evaluate(async()=>{const {default:init,WebGame}=await import("/src/wasm/chesslayer_web.js");await init();const g=WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);try{return JSON.parse(g.snapshot()) as Snapshot;}finally{g.free();}});
    expect(snapshot.moves.length).toBeGreaterThan(0);
    if(next==="이단 부대")expect(snapshot.pieces.filter(p=>p.owner===0).every(p=>p.status.blessed && p.status.ingredient)).toBe(true);
    await page.screenshot({path:`test-results/v03-${runs[i]}.png`,fullPage:true});
  }
  expect(errors).toEqual([]);
});

test("신규 부대 카드·능력·상태 및 모바일 주요 조작",async({page})=>{
  await page.emulateMedia({reducedMotion:"reduce"});await page.goto("/");await page.getByRole("button",{name:"새 게임 시작"}).waitFor();
  for(const squad of ["vanguard","heretic","artillery","special"]){
    const saved=await page.evaluate(async(squad)=>{
      const {default:init,WebGame}=await import("/src/wasm/chesslayer_web.js");await init();
      const g=WebGame.start(squad,2026,1,15);
      try{
        g.dispatch(JSON.stringify({type:"select_node",id:0}));
        // Draw naturally to reach a targeted card; never mutate engine state.
        for(let i=0;i<6;i++){
          const s:Snapshot=JSON.parse(g.snapshot());
          if(s.hand.some(c=>["stimpack","excited_steps","special_maneuver","form_line"].includes(c.definition.kind)))break;
          g.dispatch(JSON.stringify({type:"end_turn"}));
          while(JSON.parse(g.snapshot()).turn===1)g.dispatch(JSON.stringify({type:"enemy_step"}));
        }
        return g.save();
      }finally{g.free();}
    },squad);
    await page.evaluate(saved=>localStorage.setItem("chesslayer.run.v1",saved),saved);await page.reload();await page.getByRole("button",{name:"저장된 원정 이어하기"}).click();
    await page.setViewportSize({width:1440,height:900});
    const card=page.locator(".hand .playing-card").filter({hasText:squad==="vanguard"?"스팀팩":squad==="heretic"?"들뜬 발걸음":squad==="special"?"기동":"전선잡기"}).first();
    await card.click();
    if(squad==="artillery"){
      await page.locator('[data-square="a2"]').click();await page.locator('[data-square="a4"]').click();await page.getByRole("button",{name:"배치 완료"}).click();
      await expect(page.locator('[data-piece-id="9"]')).toHaveAttribute("data-cell","32");
    }else{
      await page.locator(squad==="special"?'[data-square="a1"]':'[data-square="a2"]').click();
      await expect(page.locator(".status-badge").first()).toBeVisible();
    }
    await page.setViewportSize({width:390,height:844});
    await expect(page.locator(".end-turn")).toBeInViewport();
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    await page.screenshot({path:`test-results/v03-${squad}-mobile.png`,fullPage:true});
  }
});
