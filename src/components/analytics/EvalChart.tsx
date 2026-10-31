import { createMemo, createSignal, For, onCleanup, onMount, Show } from "solid-js";

import { t, tCategory, tPhase } from "../../i18n";
import type { MoveEvaluation, Phase } from "../../types/models";
import { CATEGORY_META } from "../../types/models";
import { formatClock, formatEval, formatNumber } from "../../utils/format";
import { layoutPhaseLabels } from "./chartLabels";

const H = 214;
const ML = 38;
const MR = 14;
const MT = 12;
const EH = 138;
const HIST_TOP = 166;
const HIST_H = 38;

/** Цвета фаз: три разных тона, чтобы подложка читалась на тёмном фоне. */
const PHASE_COLORS: Record<Phase, string> = {
  opening: "#2f9e8b",
  middlegame: "#d9a441",
  endgame: "#6d8fe0",
};

export function EvalChart(props: {
  moves: MoveEvaluation[];
  totalPlies: number;
  currentPly: number;
  onSelect: (ply: number) => void;
}) {
  const [hover, setHover] = createSignal<{ ply: number; left: number } | null>(null);
  const [width, setWidth] = createSignal(1000);
  let wrapRef: HTMLDivElement | undefined;

  onMount(() => {
    if (!wrapRef) return;
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) setWidth(Math.max(360, Math.round(entry.contentRect.width)));
    });
    observer.observe(wrapRef);
    onCleanup(() => observer.disconnect());
  });

  const plotW = () => width() - ML - MR;
  const total = () => Math.max(1, props.totalPlies);
  const x = (ply: number): number => ML + (ply / total()) * plotW();
  const y = (win: number): number => MT + (1 - Math.max(0, Math.min(100, win)) / 100) * EH;
  const points = createMemo(() => {
    const list: { ply: number; x: number; y: number; win: number }[] = [
      { ply: 0, x: x(0), y: y(50), win: 50 },
    ];
    props.moves.forEach((move, index) => {
      list.push({
        ply: index + 1,
        x: x(index + 1),
        y: y(move.winProbWhite),
        win: move.winProbWhite,
      });
    });
    return list;
  });
  const linePath = createMemo(() =>
    points()
      .map((point, index) => `${index === 0 ? "M" : "L"} ${point.x.toFixed(2)} ${point.y.toFixed(2)}`)
      .join(" "),
  );
  /** Фаза хода на этом ply: подложка графика режется по фазам. */
  const phaseOf = (ply: number): Phase =>
    props.moves[ply <= 0 ? 0 : ply - 1]?.phase ?? "opening";
  const phaseRuns = createMemo(() => {
    const list = points();
    const bottom = MT + EH;
    const runs: { phase: Phase; area: string }[] = [];
    const first = list[0];
    if (!first) return runs;
    let phase = phaseOf(first.ply);
    let start = first;
    let path = `M ${first.x.toFixed(2)} ${first.y.toFixed(2)}`;
    const flush = (endX: number) => {
      runs.push({
        phase,
        area: `${path} L ${endX.toFixed(2)} ${bottom} L ${start.x.toFixed(2)} ${bottom} Z`,
      });
    };
    for (let index = 1; index < list.length; index += 1) {
      const point = list[index];
      if (!point) continue;
      path += ` L ${point.x.toFixed(2)} ${point.y.toFixed(2)}`;
      const next = phaseOf(point.ply);
      if (next === phase) continue;
      // Смена фазы ровно по её вертикальной линии: подложки стыкуются без зазора.
      flush(point.x);
      phase = next;
      start = point;
      path = `M ${point.x.toFixed(2)} ${point.y.toFixed(2)}`;
    }
    const last = list[list.length - 1];
    flush(last ? last.x : start.x);
    return runs;
  });
  const phaseLabels = createMemo(() => {
    const starts: { phase: Phase; ply: number }[] = [];
    const seen = new Set<Phase>();
    for (let ply = 1; ply <= total(); ply += 1) {
      const phase = phaseOf(ply);
      if (seen.has(phase)) continue;
      seen.add(phase);
      starts.push({ phase, ply });
    }
    return layoutPhaseLabels(starts, { left: ML, right: width() - MR }, x);
  });
  const markers = createMemo(() =>
    props.moves
      .map((move, index) => ({ move, ply: index + 1 }))
      .filter(
        ({ move }) =>
          ["brilliant", "great", "miss", "mistake", "blunder"].includes(move.classification) ||
          move.isCritical,
      ),
  );
  const phaseMarks = createMemo(() => {
    const marks: { phase: Phase; ply: number; x: number }[] = [];
    const seen = new Set<Phase>();
    props.moves.forEach((move, index) => {
      if (!seen.has(move.phase)) {
        seen.add(move.phase);
        marks.push({ phase: move.phase, ply: index + 1, x: x(index + 1) });
      }
    });
    return marks.filter((mark) => mark.phase !== "opening");
  });
  const histMax = createMemo(() => {
    const values = props.moves
      .map((move) => move.timeSpentSec)
      .filter((value): value is number => value !== null && Number.isFinite(value));
    return Math.max(1, ...values);
  });

  // The svg keeps its aspect ratio (xMidYMid meet) inside a fixed-height box,
  // so the drawing is letterboxed horizontally. Every client->viewBox conversion
  // must subtract that offset, otherwise the cursor and the highlighted ply drift apart.
  const geometry = (target: SVGSVGElement): { rect: DOMRect; scale: number; padX: number } => {
    const rect = target.getBoundingClientRect();
    const scale = Math.min(rect.width / width(), rect.height / H) || 1;
    return { rect, scale, padX: (rect.width - width() * scale) / 2 };
  };

  const plyFromEvent = (event: PointerEvent | MouseEvent): number => {
    const target = event.currentTarget;
    if (!(target instanceof SVGSVGElement)) return 0;
    const { rect, scale, padX } = geometry(target);
    const viewX = (event.clientX - rect.left - padX) / scale;
    const raw = ((viewX - ML) / plotW()) * total();
    return Math.max(0, Math.min(total(), Math.round(raw)));
  };

  const handleMove = (event: PointerEvent): void => {
    const target = event.currentTarget;
    if (!(target instanceof SVGSVGElement)) return;
    const ply = plyFromEvent(event);
    const { rect, scale, padX } = geometry(target);
    // Snap the guide to the data point so the highlight always matches the tooltip.
    const left = ((padX + x(ply) * scale) / rect.width) * 100;
    setHover({ ply, left });
  };

  const hovered = createMemo(() => {
    const state = hover();
    if (!state) return null;
    const move = state.ply === 0 ? null : (props.moves[state.ply - 1] ?? null);
    const win = move ? move.winProbWhite : 50;
    const moveNumber = Math.max(1, Math.ceil(state.ply / 2));
    const label = state.ply === 0 ? "" : state.ply % 2 === 1 ? `${moveNumber}.` : `${moveNumber}…`;
    return { ...state, move, win, label };
  });

  return (
    <div class="chart-wrap" ref={wrapRef}>
      <svg
        class="chart-svg"
        viewBox={`0 0 ${width()} ${H}`}
        onPointerMove={handleMove}
        onPointerLeave={() => setHover(null)}
        onClick={(event) => props.onSelect(plyFromEvent(event))}
      >
        <defs>
          <For each={["opening", "middlegame", "endgame"] as Phase[]}>
            {(phase) => (
              <linearGradient id={`eval-fill-${phase}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color={PHASE_COLORS[phase]} stop-opacity="0.34" />
                <stop offset="100%" stop-color={PHASE_COLORS[phase]} stop-opacity="0.05" />
              </linearGradient>
            )}
          </For>
        </defs>

        <For each={[0, 25, 50, 75, 100]}>
          {(value) => (
            <>
              <line x1={ML} x2={width() - MR} y1={y(value)} y2={y(value)} stroke="#293123" stroke-width="0.7" />
              <text x={ML - 7} y={y(value) + 3} text-anchor="end" fill="#868d78" font-size="9">
                {value}
              </text>
            </>
          )}
        </For>

        <For each={phaseMarks()}>
          {(mark) => (
            <line
              x1={mark.x}
              x2={mark.x}
              y1={MT}
              y2={H - 4}
              stroke="#3a4432"
              stroke-width="0.8"
              stroke-dasharray="3 4"
            />
          )}
        </For>

        <For each={phaseRuns()}>
          {(run) => <path d={run.area} fill={`url(#eval-fill-${run.phase})`} />}
        </For>
        <path d={linePath()} fill="none" stroke="#e0bf7d" stroke-width="1.8" stroke-linejoin="round" />
        <For each={phaseLabels()}>
          {(label) => (
            <text
              x={label.x}
              y={MT - 3}
              text-anchor={label.anchor}
              fill={PHASE_COLORS[label.phase]}
              font-size="9"
              opacity="0.9"
            >
              {tPhase(label.phase)}
            </text>
          )}
        </For>
        <line x1={ML} x2={width() - MR} y1={y(50)} y2={y(50)} stroke="#4a5344" stroke-width="0.7" stroke-dasharray="4 4" />

        <For each={markers()}>
          {(marker) => (
            <circle
              cx={x(marker.ply)}
              cy={y(marker.move.winProbWhite)}
              r="2.3"
              fill={CATEGORY_META[marker.move.classification].color}
              stroke="#0c110b"
              stroke-width="0.8"
              opacity="0.9"
            />
          )}
        </For>

        <line
          x1={x(props.currentPly)}
          x2={x(props.currentPly)}
          y1={MT}
          y2={H - 4}
          stroke="#e0bf7d"
          stroke-width="1.1"
          opacity="0.8"
        />

        <text x={ML - 7} y={HIST_TOP - 4} text-anchor="end" fill="#868d78" font-size="9">
          t
        </text>
        <line x1={ML} x2={width() - MR} y1={HIST_TOP + HIST_H} y2={HIST_TOP + HIST_H} stroke="#293123" stroke-width="0.7" />
        <For each={props.moves}>
          {(move, index) => {
            const height = () =>
              move.timeSpentSec === null ? 0 : Math.max(1, (move.timeSpentSec / histMax()) * HIST_H);
            const barWidth = plotW() / total() * 0.34;
            const color = () =>
              move.timeFlag === "time_trouble" || move.timeFlag === "impulsive_error"
                ? "#e05c4f"
                : move.timeFlag === "overthink"
                  ? "#e08a3c"
                  : index() + 1 === props.currentPly
                    ? "#e0bf7d"
                    : "#4a5344";
            return (
              <rect
                x={x(index() + 1) - barWidth / 2}
                y={HIST_TOP + HIST_H - height()}
                width={barWidth}
                height={height()}
                rx={barWidth / 2}
                fill={color()}
                opacity="0.82"
              />
            );
          }}
        </For>

        <Show when={hovered()}>
          {(state) => (
            <>
              <line
                x1={x(state().ply)}
                x2={x(state().ply)}
                y1={MT}
                y2={H - 4}
                class="chart-hover-line"
              />
              <circle
                cx={x(state().ply)}
                cy={y(state().win)}
                r="4"
                fill="#e0bf7d"
                stroke="#0c110b"
                stroke-width="1.4"
              />
            </>
          )}
        </Show>
      </svg>

      <Show when={hovered()}>
        {(state) => (
          <div
            class="chart-tooltip"
            style={{
              left: `${Math.max(12, Math.min(88, state().left))}%`,
              top: "6px",
              transform: "translateX(-50%)",
            }}
          >
            <Show
              when={state().move}
              fallback={<span class="muted">{t("studio.startPosition")}</span>}
            >
              {(move) => (
                <div style={{ display: "flex", "flex-direction": "column", gap: "2px" }}>
                  <div style={{ display: "flex", gap: "6px", "align-items": "center" }}>
                    <span class="mono">{state().label}</span>
                    <span style={{ "font-weight": "700" }}>{move().playedSan}</span>
                    <span style={{ color: CATEGORY_META[move().classification].color }}>
                      {tCategory(move().classification)}
                    </span>
                  </div>
                  <div class="mono" style={{ color: "var(--muted)" }}>
                    {formatEval(move().evalCp, move().evalMate)}, Win{" "}
                    {formatNumber(move().winProbWhite, 1)}%, {formatClock(move().timeSpentSec)}
                  </div>
                </div>
              )}
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
}
