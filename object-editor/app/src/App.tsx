import { useState } from "react";
import { EditorScreen } from "./components/EditorScreen";
import { SetupScreen } from "./components/SetupScreen";
import type { GameSummary, MapSummary } from "./types";

type Ready = {
  game: GameSummary;
  map: MapSummary | null;
};

export function App() {
  const [ready, setReady] = useState<Ready | null>(null);

  if (!ready) {
    return <SetupScreen onReady={(game, map) => setReady({ game, map })} />;
  }

  return <EditorScreen game={ready.game} initialMap={ready.map} />;
}
