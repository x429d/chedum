import { createEffect, createSignal, For, on, Show } from "solid-js";

import { t, tCategory } from "../../i18n";
import { gameStore } from "../../stores/gameStore";
import type { MoveCategory, MoveEvaluation } from "../../types/models";
import { CATEGORY_META } from "../../types/models";
import { formatClock } from "../../utils/format";
import { CategoryGlyph } from "../ui/CategoryGlyph";
import { CoachCard } from "./CoachCard";

function MoveCell(props: {
  move: MoveEvaluation | null;
  currentPly: number;
  filter: MoveCategory | null;
  onSelect: (ply: number) => void;
}) {
  return (
    <Show
      when={props.move}
      fallback={<span class="move-cell dimmed">—</span>}
    >
      {(move) => (
        <button
          type="button"
          class="move-cell"
          classList={{
            current: move().ply === props.currentPly,
            dimmed: props.filter !== null && move().classification !== props.filter,
          }}
          onClick={() => props.onSelect(move().ply)}
        >
          <span
            class="category-symbol"
            style={{ background: CATEGORY_META[move().classification].color, width: "16px", height: "16px" }}
          >
            <CategoryGlyph category={move().classification} size={11} />
          </span>
          <span class="move-san">{move().playedSan}</span>
          <Show when={move().timeSpentSec !== null}>
            <span class="move-accuracy">{formatClock(move().timeSpentSec)}</span>
          </Show>
        </button>
      )}
    </Show>
  );
}

export function MovesTab() {
  const [draft, setDraft] = createSignal("");
  const [notes, setNotes] = createSignal("");

  createEffect(
    on(
      () => [gameStore.ply(), gameStore.currentMove()?.userComment] as const,
      (state) => setDraft(state[1] ?? ""),
    ),
  );
  createEffect(
    on(
      () => gameStore.detail()?.game.notes ?? "",
      (value) => setNotes(value),
    ),
  );

  return (
    <div style={{ display: "flex", "flex-direction": "column", gap: "12px" }}>
      <CoachCard move={gameStore.currentMove()} onPlayLine={() => gameStore.playBestLine()} />

      <div>
        <div style={{ display: "flex", "align-items": "center", "justify-content": "space-between" }}>
          <span class="card-title">{t("studio.moves")}</span>
          <Show when={gameStore.categoryFilter()}>
            {(category) => (
              <button
                type="button"
                class="chip accent"
                onClick={() => gameStore.setCategoryFilter(null)}
              >
                {tCategory(category())} ✕
              </button>
            )}
          </Show>
        </div>
        <div class="moves-grid" style={{ "margin-top": "8px" }}>
          <For each={gameStore.movePairs()}>
            {(pair) => (
              <>
                <span class="move-number">{pair.number}.</span>
                <MoveCell
                  move={pair.white}
                  currentPly={gameStore.ply()}
                  filter={gameStore.categoryFilter()}
                  onSelect={(ply) => gameStore.goTo(ply)}
                />
                <MoveCell
                  move={pair.black}
                  currentPly={gameStore.ply()}
                  filter={gameStore.categoryFilter()}
                  onSelect={(ply) => gameStore.goTo(ply)}
                />
              </>
            )}
          </For>
        </div>
      </div>

      <Show when={gameStore.currentMove()}>
        {(move) => (
          <div class="card" style={{ padding: "10px" }}>
            <div class="form-label">{t("studio.noteForMove", { ply: move().ply })}</div>
            <textarea
              class="textarea"
              style={{ "min-height": "60px", "margin-top": "5px" }}
              value={draft()}
              placeholder={t("studio.noteMovePlaceholder")}
              onInput={(event) => setDraft(event.currentTarget.value)}
            />
            <button
              type="button"
              class="btn btn-sm"
              style={{ "margin-top": "7px" }}
              onClick={() => void gameStore.saveComment(move().ply, draft())}
            >
              {t("studio.saveNote")}
            </button>
          </div>
        )}
      </Show>

      <div class="card" style={{ padding: "10px" }}>
        <div class="form-label">{t("studio.gameNotes")}</div>
        <textarea
          class="textarea"
          style={{ "min-height": "56px", "margin-top": "5px" }}
          value={notes()}
          placeholder={t("studio.gameNotesPlaceholder")}
          onInput={(event) => setNotes(event.currentTarget.value)}
        />
        <button
          type="button"
          class="btn btn-sm"
          style={{ "margin-top": "7px" }}
          onClick={() => void gameStore.saveNotes(notes())}
        >
          {t("studio.saveGameNotes")}
        </button>
      </div>
    </div>
  );
}
