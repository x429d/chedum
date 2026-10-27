import { For } from "solid-js";
import type { Role } from "chessops/types";

import { settingsStore } from "../../stores/settingsStore";
import type { Color } from "../../types/chess";
import { pieceImage, PIECE_SETS, pieceSetMeta } from "../../utils/pieceSets";
import { Icon } from "../ui/Icon";

interface PreviewCell {
  color: Color;
  role: Role;
  light: boolean;
}

const PREVIEW: PreviewCell[] = [
  { color: "white", role: "king", light: false },
  { color: "black", role: "knight", light: true },
  { color: "white", role: "knight", light: true },
  { color: "black", role: "king", light: false },
];

export function PieceSetPicker() {
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">Набор фигур</div>
          <div class="card-subtitle">Шесть open-source SVG-наборов для доски</div>
        </div>
        <Icon name="sparkles" size={16} />
      </div>
      <div class="card-pad">
        <div class="piece-set-grid">
          <For each={PIECE_SETS}>
            {(set) => (
              <button
                type="button"
                class="piece-set-card"
                classList={{ selected: settingsStore.settings().pieceSet === set.id }}
                aria-pressed={settingsStore.settings().pieceSet === set.id}
                title={`${set.author} · ${set.license}`}
                onClick={() => void settingsStore.setPieceSet(set.id)}
              >
                <span
                  class={`piece-set-board theme-mini-${settingsStore.settings().boardTheme}`}
                >
                  <For each={PREVIEW}>
                    {(cell) => (
                      <span classList={{ light: cell.light }}>
                        <img src={pieceImage(set.id, cell.color, cell.role)} alt="" />
                      </span>
                    )}
                  </For>
                </span>
                <span class="piece-set-meta">
                  <span class="piece-set-name">{set.label}</span>
                  <span class="piece-set-license">{set.license}</span>
                </span>
              </button>
            )}
          </For>
        </div>
        <div class="form-hint" style={{ "margin-top": "9px" }}>
          {pieceSetMeta(settingsStore.settings().pieceSet).author} ·{" "}
          {pieceSetMeta(settingsStore.settings().pieceSet).license}. Все наборы распространяются
          по открытым лицензиям.
        </div>
      </div>
    </div>
  );
}
