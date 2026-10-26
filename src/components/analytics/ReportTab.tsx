import { Show } from "solid-js";

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
  const userPerf = () =>
    props.game.userColor === "black" ? props.analysis.blackPerfElo : props.analysis.whitePerfElo;

  return (
    <div style={{ display: "flex", "flex-direction": "column", gap: "14px" }}>
      <div>
        <div class="card-title" style={{ "margin-bottom": "6px" }}>
          Резюме партии
        </div>
        <div class="summary-text">{props.analysis.summaryText}</div>
        <div style={{ display: "flex", gap: "6px", "flex-wrap": "wrap", "margin-top": "9px" }}>
          <Show when={props.game.eco}>
            <span class="chip accent">{props.game.eco}</span>
          </Show>
          <Show when={props.game.openingName}>
            <span class="chip">{props.game.openingName}</span>
          </Show>
          <span class="chip">depth {props.analysis.depth}</span>
          <span class="chip">ключевых: {props.analysis.keyMomentsCount}</span>
        </div>
      </div>

      <div class="ring-row">
        <AccuracyRing
          value={props.analysis.whiteAccuracy}
          color={isWhiteUser() ? "#2dd4bf" : "#cbd5e1"}
          label="Белые"
        />
        <AccuracyRing
          value={props.analysis.blackAccuracy}
          color={!isWhiteUser() ? "#2dd4bf" : "#64748b"}
          label="Чёрные"
        />
      </div>

      <div class="metric-tiles">
        <div class="metric-tile">
          <div class="metric-label">Ваша точность</div>
          <div class="metric-value">{userAccuracy() === null ? "—" : `${formatNumber(userAccuracy(), 1)}%`}</div>
          <div class="stat-hint">средневзвешенная</div>
        </div>
        <div class="metric-tile">
          <div class="metric-label">Performance Elo</div>
          <div class="metric-value">{userPerf() ?? "—"}</div>
          <div class="stat-hint">оценка силы игры</div>
        </div>
        <div class="metric-tile">
          <div class="metric-label">ACPL</div>
          <div class="metric-value">{formatNumber(userAcpl(), 1)}</div>
          <div class="stat-hint">потеря сантипешек</div>
        </div>
        <div class="metric-tile">
          <div class="metric-label">Соперник</div>
          <div class="metric-value" style={{ "font-size": "13px" }}>
            {isWhiteUser() ? props.game.blackUsername : props.game.whiteUsername}
          </div>
          <div class="stat-hint">{isWhiteUser() ? props.game.blackElo ?? "—" : props.game.whiteElo ?? "—"} Elo</div>
        </div>
      </div>

      <div>
        <div style={{ display: "flex", "justify-content": "space-between", "align-items": "baseline" }}>
          <span class="card-title">Категории ходов</span>
          <span class="dim" style={{ "font-size": "10px" }}>
            клик — фильтр
          </span>
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
          Точность по фазам
        </div>
        <PhaseBars analysis={props.analysis} userColor={props.game.userColor} />
      </div>

      <div>
        <div class="card-title" style={{ "margin-bottom": "8px" }}>
          Тайм-менеджмент
        </div>
        <TimeSummary moves={props.moves} userColor={userColor()} />
      </div>
    </div>
  );
}
