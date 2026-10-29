import { createMemo, createSignal, For, Show } from "solid-js";

import type { MoveEvaluation, Phase } from "../../types/models";
import { CATEGORY_META, PHASE_LABELS } from "../../types/models";
import { formatClock, formatEval, formatNumber } from "../../utils/format";

const W = 1000;
const H = 214;
const ML = 38;
const MR = 14;
const MT = 12;
const EH = 138;
const HIST_TOP = 166;
const HIST_H = 38;
const PLOT_W = W - ML - MR;

export function EvalChart(props: {
  moves: MoveEvaluation[];
  totalPlies: number;
  currentPly: number;
  onSelect: (ply: number) => void;
}) {
  const [hover, setHover] = createSignal<{ ply: number; left: number } | null>(null);
  const total = () => Math.max(1, props.totalPlies);
  const x = (ply: number): number => ML + (ply / total()) * PLOT_W;
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
  const areaPath = createMemo(() => {
    const bottom = MT + EH;
    const first = points()[0];
    const last = points()[points().length - 1];
    if (!first || !last) return "";
    return `${linePath()} L ${last.x.toFixed(2)} ${bottom} L ${first.x.toFixed(2)} ${bottom} Z`;
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
    const scale = Math.min(rect.width / W, rect.height / H) || 1;
    return { rect, scale, padX: (rect.width - W * scale) / 2 };
  };

  const plyFromEvent = (event: PointerEvent | MouseEvent): number => {
    const target = event.currentTarget;
    if (!(target instanceof SVGSVGElement)) return 0;
    const { rect, scale, padX } = geometry(target);
    const viewX = (event.clientX - rect.left - padX) / scale;
    const raw = ((viewX - ML) / PLOT_W) * total();
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
    <div class="chart-wrap">
      <svg
        class="chart-svg"
        viewBox={`0 0 ${W} ${H}`}
        onPointerMove={handleMove}
        onPointerLeave={() => setHover(null)}
        onClick={(event) => props.onSelect(plyFromEvent(event))}
      >
        <defs>
          <linearGradient id="eval-fill" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#c6a15b" stop-opacity="0.24" />
            <stop offset="100%" stop-color="#c6a15b" stop-opacity="0.01" />
          </linearGradient>
        </defs>

        <For each={[0, 25, 50, 75, 100]}>
          {(value) => (
            <>
              <line x1={ML} x2={W - MR} y1={y(value)} y2={y(value)} stroke="#293123" stroke-width="0.7" />
              <text x={ML - 7} y={y(value) + 3} text-anchor="end" fill="#868d78" font-size="9">
                {value}
              </text>
            </>
          )}
        </For>

        <For each={phaseMarks()}>
          {(mark) => (
            <>
              <line
                x1={mark.x}
                x2={mark.x}
                y1={MT}
                y2={H - 4}
                stroke="#3a4432"
                stroke-width="0.8"
                stroke-dasharray="3 4"
              />
              <text x={mark.x + 5} y={MT + 10} fill="#a4a794" font-size="9">
                {PHASE_LABELS[mark.phase]}
              </text>
            </>
          )}
        </For>
        <text x={ML + 5} y={MT + 10} fill="#a4a794" font-size="9">
          {PHASE_LABELS.opening}
        </text>

        <path d={areaPath()} fill="url(#eval-fill)" />
        <path d={linePath()} fill="none" stroke="#e0bf7d" stroke-width="1.8" stroke-linejoin="round" />
        <line x1={ML} x2={W - MR} y1={y(50)} y2={y(50)} stroke="#4a5344" stroke-width="0.7" stroke-dasharray="4 4" />

        <For each={markers()}>
          {(marker) => (
            <circle
              cx={x(marker.ply)}
              cy={y(marker.move.winProbWhite)}
              r="3.6"
              fill={CATEGORY_META[marker.move.classification].color}
              stroke="#0c110b"
              stroke-width="1.2"
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
        <line x1={ML} x2={W - MR} y1={HIST_TOP + HIST_H} y2={HIST_TOP + HIST_H} stroke="#293123" stroke-width="0.7" />
        <For each={props.moves}>
          {(move, index) => {
            const height = () =>
              move.timeSpentSec === null ? 0 : Math.max(1, (move.timeSpentSec / histMax()) * HIST_H);
            const barWidth = PLOT_W / total() * 0.64;
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
                opacity="0.9"
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
              fallback={<span class="muted">Стартовая позиция</span>}
            >
              {(move) => (
                <div style={{ display: "flex", "flex-direction": "column", gap: "2px" }}>
                  <div style={{ display: "flex", gap: "6px", "align-items": "center" }}>
                    <span class="mono">{state().label}</span>
                    <span style={{ "font-weight": "700" }}>{move().playedSan}</span>
                    <span style={{ color: CATEGORY_META[move().classification].color }}>
                      {CATEGORY_META[move().classification].label}
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
