import { Show } from "solid-js";

import { t } from "../../i18n";
import type { Game, GameAnalysis } from "../../types/models";
import { formatDate, timeClassLabel } from "../../utils/format";
import { Icon } from "../ui/Icon";

export function AnalysisHeader(props: {
  game: Game;
  analysis: GameAnalysis | null;
  analyzing: boolean;
  onAnalyze: () => void;
  onBack: () => void;
}) {
  return (
    <header class="analysis-header">
      <button type="button" class="back-btn" title={t("studio.backToLibrary")} aria-label={t("studio.backToLibrary")} onClick={props.onBack}>
        <Icon name="chevron-left" size={16} />
      </button>
      <div style={{ "min-width": "0", flex: "1" }}>
        <div class="players-line">
          <span class="player-name">
            {props.game.whiteUsername}
            <span class="player-elo"> {props.game.whiteElo ?? ""}</span>
          </span>
          <span class="dim" style={{ "font-size": "11px" }}>
            vs
          </span>
          <span class="player-name">
            {props.game.blackUsername}
            <span class="player-elo"> {props.game.blackElo ?? ""}</span>
          </span>
          <span class="result-pill">{props.game.result}</span>
          <Show when={props.game.userColor !== "none"}>
            <span class="chip accent">
              {props.game.userColor === "white" ? t("studio.youWhite") : t("studio.youBlack")}
            </span>
          </Show>
        </div>
        <div class="analysis-meta">
          <span>{formatDate(props.game.playedAt)},</span>
          <span>{timeClassLabel(props.game.timeClass)},</span>
          <span>{props.game.timeControl ?? "—"}</span>
          <Show when={props.game.source !== "manual"}>
            <span class={`source-chip source-${props.game.source}`}>
              {t(
                props.game.source === "chesscom"
                  ? "accounts.platformChesscom"
                  : "accounts.platformLichess",
              )}
            </span>
          </Show>
          <Show when={props.game.eco}>
            <span class="eco-code">{props.game.eco}</span>
            <span class="opening-name" style={{ "max-width": "320px" }}>
              {props.game.openingName}
            </span>
          </Show>
          <Show when={props.analysis}>
            {(value) => (
              <span class="chip" style={{ height: "22px", "font-size": "10px" }}>
                {value().engineName}, depth {value().depth}
              </span>
            )}
          </Show>
        </div>
      </div>
      <div class="header-actions">
        <button
          type="button"
          class="btn btn-primary"
          disabled={props.analyzing}
          onClick={props.onAnalyze}
        >
          <Show when={props.analyzing} fallback={<Icon name="analyze" size={14} />}>
            <span class="spinner" style={{ width: "13px", height: "13px" }} />
          </Show>
          {props.analysis ? t("studio.reanalyze") : t("studio.analyze")}
        </button>
        <button
          type="button"
          class="btn btn-ghost icon-btn"
          title={t("nav.library")}
          aria-label={t("nav.library")}
          onClick={props.onBack}
        >
          <Icon name="library" size={15} />
        </button>
      </div>
    </header>
  );
}
