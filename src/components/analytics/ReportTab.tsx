import { Show } from "solid-js";

import { t } from "../../i18n";
import { gameStore } from "../../stores/gameStore";
import type { Game, GameAnalysis, MoveEvaluation } from "../../types/models";
import { formatNumber } from "../../utils/format";
import { AccuracyRing } from "./AccuracyRing";
import { CategoryTable } from "./CategoryTable";
import { PhaseBars } from "./PhaseBars";
import { TimeSummary } from "./TimeSummary";

export function ReportTab(props: {
  analysis: GameAnalysis;
  game: Game;
  moves: MoveEvaluation[];
}) {
  const isWhiteUser = () => props.game.userColor === "white";
  const userColor = () => (props.game.userColor === "black" ? "black" : "white");
  const userAccuracy = () =>
    props.game.userColor === "black" ? props.analysis.blackAccuracy : props.analysis.whiteAccuracy;
  const userAcpl = () =>
    props.game.userColor === "black" ? props.analysis.blackAcpl : props.analysis.whiteAcpl;
  const opponentElo = () =>
    props.game.userColor === "black" ? props.game.whiteElo : props.game.blackElo;
  const platformLabel = () => {
    if (props.game.source === "chesscom") return t("accounts.platformChesscom");
    if (props.game.source === "lichess") return t("accounts.platformLichess");
    return null;
  };
  const resultPerf = () => {
    const elo = opponentElo();
    if (elo === null) return null;
    const score =
      props.game.userColor === "white"
        ? props.game.result === "1-0"
          ? 1
          : props.game.result === "1/2-1/2"
            ? 0.5
            : 0
        : props.game.result === "0-1"
          ? 1
          : props.game.result === "1/2-1/2"
            ? 0.5
            : 0;
    return Math.round(elo + 800 * (score - 0.5));
  };

  return (
    <div style={{ display: "flex", "flex-direction": "column", gap: "14px" }}>
      <div>
        <div class="card-title" style={{ "margin-bottom": "6px" }}>
          {t("studio.summaryTitle")}
        </div>
        <div class="summary-text">{props.analysis.summaryText}</div>
        <div style={{ display: "flex", gap: "6px", "flex-wrap": "wrap", "margin-top": "9px" }}>
          <Show when={props.game.eco}>
            <span class="chip accent">{props.game.eco}</span>
          </Show>
          <Show when={props.game.openingName}>
            <span class="chip">{props.game.openingName}</span>
          </Show>
          <span class="chip">{t("studio.depth")} {props.analysis.depth}</span>
          <span class="chip">{t("studio.keyMoments")}: {props.analysis.keyMomentsCount}</span>
        </div>
      </div>

      <div class="ring-row">
        <AccuracyRing
          value={props.analysis.whiteAccuracy}
          color={isWhiteUser() ? "#e0bf7d" : "#9a9f8d"}
          label={t("library.white")}
        />
        <AccuracyRing
          value={props.analysis.blackAccuracy}
          color={!isWhiteUser() ? "#e0bf7d" : "#9a9f8d"}
          label={t("library.black")}
        />
      </div>

      <div class="metric-tiles">
        <div class="metric-tile">
          <div class="metric-label">{t("studio.yourAccuracy")}</div>
          <div class="metric-value">{userAccuracy() === null ? "—" : `${formatNumber(userAccuracy(), 1)}%`}</div>
        </div>
        <div class="metric-tile">
          <div class="metric-label">{t("studio.perfRating")}</div>
          <div class="metric-value">{resultPerf() ?? "—"}</div>
          <Show when={platformLabel()}>
            {(platform) => (
              <div class="stat-hint">
                {t("studio.perfOnPlatform", { platform: platform() })}
              </div>
            )}
          </Show>
        </div>
        <div class="metric-tile">
          <div class="metric-label">ACPL</div>
          <div class="metric-value">{formatNumber(userAcpl(), 1)}</div>
        </div>
        <div class="metric-tile">
          <div class="metric-label">{t("studio.opponent")}</div>
          <div class="metric-value" style={{ "font-size": "13px" }}>
            {isWhiteUser() ? props.game.blackUsername : props.game.whiteUsername}
          </div>
          <Show when={isWhiteUser() ? props.game.blackElo : props.game.whiteElo}>
            {(elo) => <div class="stat-hint">{elo()} Elo</div>}
          </Show>
        </div>
      </div>

      <div>
        <div style={{ display: "flex", "justify-content": "space-between", "align-items": "baseline" }}>
          <span class="card-title">{t("studio.category")}</span>
        </div>
        <div style={{ "margin-top": "5px" }}>
          <CategoryTable
            counts={props.analysis.counts}
            userColor={props.game.userColor}
            active={gameStore.categoryFilter()}
            onSelect={(category) => gameStore.toggleCategory(category)}
          />
        </div>
      </div>

      <div>
        <div class="card-title" style={{ "margin-bottom": "8px" }}>
          {t("studio.phases")}
        </div>
        <PhaseBars analysis={props.analysis} userColor={props.game.userColor} />
      </div>

      <div>
        <div class="card-title" style={{ "margin-bottom": "8px" }}>
          {t("studio.moveTimes")}
        </div>
        <TimeSummary moves={props.moves} userColor={userColor()} />
      </div>
    </div>
  );
}
