import init, { WebGame } from "./wasm/chesslayer_web";

self.onmessage = async (event: MessageEvent<string>) => {
  try {
    await init();
    const game = WebGame.restore(event.data);
    const choice: [number, number] | null = JSON.parse(game.recommend_enemy());
    game.free();
    self.postMessage({ choice });
  } catch (error) {
    self.postMessage({ error: String(error) });
  }
};
