import { createEffect, createMemo, createSignal, on, Show } from "solid-js";

import { gameStore } from "../../stores/gameStore";
import { settingsStore } from "../../stores/settingsStore";
import type { Color, MoveCategory, MoveEvaluation } from "../../types/models";
import { CATEGORY_META } from "../../types/models";
import { applyUci, checkSquare, playMove } from "../../utils/chess";
import { formatEval } from "../../utils/format";
import { Board } from "../board/Board";
import { CategoryGlyph } from "../ui/CategoryGlyph";
import { Icon } from "../ui/Icon";

type RetryStatus = "idle" | "correct" | "wrong";
const RETRY_CATEGORIES: MoveCategory[] = ["mistake", "blunder", "miss"];

export function RetryTrainer() {
  const [index, setIndex] = createSignal(0);
  const [status, setStatus] = createSignal<RetryStatus>("idle");
  const [attempts, setAttempts] = createSignal(0);
  const [solved, setSolved] = createSignal<number[]>([]);
  const [resultFen, setResultFen] = createSignal<string | null>(null);
  const userColor = (): Color =>
    gameStore.detail()?.game.userColor === "black" ? "black" : "white";
  const mistakes = createMemo<MoveEvaluation[]>(() => {
    const color = userColor();
    return gameStore
      .moves()
      .filter(
        (move) =>
          RETRY_CATEGORIES.includes(move.classification) &&
          (color === "white" ? move.ply % 2 === 1 : move.ply % 2 === 0),
      )
      .sort((left, right) => right.winProbLoss - left.winProbLoss);
  });
  const active = createMemo(() => mistakes()[index()] ?? null);

  createEffect(
    on(
      () => active()?.ply,
      () => {
        setStatus("idle");
        setResultFen(null);
      },
    ),
  );

  createEffect(
    on(
      () => gameStore.detail()?.game.id,
      () => {
        setIndex(0);
        setStatus("idle");
        setResultFen(null);
        setSolved([]);
        setAttempts(0);
      },
    ),
  );

  createEffect(
    on(
      () => mistakes().length,
      (length) => setIndex((value) => Math.min(value, Math.max(0, length - 1))),
    ),
  );

  const boardFen = (): string => {
    const move = active();
    if (!move) return "";
    return status() === "idle" ? move.fenBefore : resultFen() ?? move.fenBefore;
  };

  const showSolution = (): void => {
    const move = active();
    if (!move) return;
    const best = move.bestUci ? applyUci(move.fenBefore, move.bestUci) : null;
    setResultFen(best?.fenAfter ?? null);
    setStatus("wrong");
  };

  const handleMove = (from: string, to: string): void => {
    const move = active();
    if (!move || status() === "correct") return;
    const played = playMove(move.fenBefore, from, to);
    if (!played) return;
    setResultFen(played.fenAfter);
    setAttempts((value) => value + 1);
    if (move.bestUci && played.uci === move.bestUci) {
      setStatus("correct");
      setSolved((items) => (items.includes(move.ply) ? items : [...items, move.ply]));
    } else {
      setStatus("wrong");
    }
  };

  const next = (): void => {
    setResultFen(null);
    setStatus("idle");
    setIndex((value) => Math.min(value + 1, Math.max(0, mistakes().length - 1)));
  };

  const retry = (): void => {
    setResultFen(null);
    setStatus("idle");
  };

  return (
    <Show
      when={active()}
      fallback={
        <div class="empty-state" style={{ padding: "24px" }}>
          <div class="empty-icon">
            <Icon name="target" />
          </div>
          <div style={{ "font-weight": "600" }}>Ошибок для тренировки нет</div>
          <div style={{ "font-size": "11px" }}>Партия проведена чисто — так держать.</div>
        </div>
      }
    >
      {(move) => (
        <div>
          <div style={{ display: "flex", "align-items": "center", "justify-content": "space-between" }}>
            <div class="card-title">Работа над ошибками</div>
            <span class="chip accent">
              решено {solved().length} / {mistakes().length}
            </span>
          </div>
          <div class="sync-log" style={{ "margin-top": "4px" }}>
            Найдите лучший ход за {userColor() === "white" ? "белых" : "чёрных"}. Стрелки скрыты.
          </div>
          <div style={{ display: "flex", "align-items": "center", gap: "6px", "margin": "8px 0" }}>
            <span
              class="badge"
              style={{
                color: CATEGORY_META[move().classification].color,
                "border-color": CATEGORY_META[move().classification].color,
              }}
            >
              <CategoryGlyph category={move().classification} size={12} />{" "}
              {CATEGORY_META[move().classification].label}
            </span>
            <span class="mono" style={{ "font-size": "11px" }}>
              {move().ply}. {move().playedSan}
            </span>
            <span class="mono" style={{ "font-size": "10.5px", color: "var(--muted)", "margin-left": "auto" }}>
              {formatEval(move().bestEvalCp, move().bestEvalMate)} → {formatEval(move().evalCp, move().evalMate)}
            </span>
          </div>
          <div class="retry-board">
            <Board
              fen={boardFen()}
              orientation={userColor()}
              theme={settingsStore.settings().boardTheme}
              pieceSet={settingsStore.settings().pieceSet}
              embedded
              interactive={status() === "idle"}
              checkSquare={checkSquare(boardFen())}
              onMove={handleMove}
            />
          </div>
          <div style={{ "margin-top": "8px", "font-size": "11.5px", "line-height": "1.45" }}>
            <Show when={status() === "idle"}>
              <span class="muted">Ход ещё не сделан. Попыток: {attempts()}.</span>
            </Show>
            <Show when={status() === "correct"}>
              <span style={{ color: "#9ccf8f" }}>
                Верно! Лучший ход — {move().bestSan ?? "найден"}.
              </span>
            </Show>
            <Show when={status() === "wrong"}>
              <span style={{ color: "#e79a90" }}>
                Неверно. Лучший ход: {move().bestSan ?? "—"}.
              </span>
            </Show>
          </div>
          <div style={{ display: "flex", gap: "6px", "margin-top": "8px", "flex-wrap": "wrap" }}>
            <button
              type="button"
              class="btn btn-sm"
              onClick={showSolution}
              disabled={status() === "correct"}
            >
              <Icon name="eye" size={12} />
              Показать решение
            </button>
            <button type="button" class="btn btn-sm" onClick={retry}>
              <Icon name="retry" size={12} />
              Повторить
            </button>
            <button type="button" class="btn btn-sm" onClick={next} disabled={index() >= mistakes().length - 1}>
              Следующая
              <Icon name="chevron-right" size={12} />
            </button>
          </div>
        </div>
      )}
    </Show>
  );
}
