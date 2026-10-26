import { gameStore } from "../../stores/gameStore";
import { KeyMoments } from "./KeyMoments";
import { RetryTrainer } from "./RetryTrainer";

export function KeyMomentsTab() {
  return (
    <div style={{ display: "flex", "flex-direction": "column", gap: "16px" }}>
      <KeyMoments moves={gameStore.moves()} onSelect={(ply) => gameStore.goTo(ply)} />
      <div class="divider" />
      <RetryTrainer />
    </div>
  );
}
