import { createMemo, createSignal, For, onCleanup, onMount, Show } from "solid-js";

import { t, tPhase } from "../../i18n";
import {
  CATEGORY_META,
  type MoveEvaluation,
  type Phase,
  type UserColor,
} from "../../types/models";
import { layoutPhaseLabels } from "./chartLabels";
import { EvalChartTooltip } from "./EvalChartTooltip";
import {
  type ChartScope,
  inScope,
  markerPoints,
  maxMoveTime,
  phaseMarksFor,
  scopedMovesFor,
} from "./evalChartData";
import {
  buildLinePath,
  buildPhaseRuns,
  chartX,
  chartY,
  H,
  ML,
  MR,
  MT,
  PHASE_COLORS,
} from "./evalChartLayout";
import { TimeHistogram } from "./TimeHistogram";

export type { ChartScope } from "./evalChartData";
export function EvalChart(props: {
  moves: MoveEvaluation[];
  totalPlies: number;
  currentPly: number;
  userColor?: UserColor;
  scope?: ChartScope;
  onSelect: (ply: number) => void;
}) {
  const [hover, setHover] = createSignal<{ ply: number; left: number } | null>(null);
  const [width, setWidth] = createSignal(1000);
  let wrapRef: HTMLDivElement | undefined;

  const scope = () => props.scope ?? "user";
  const isPlyInScope = (ply: number): boolean => inScope(ply, scope(), props.userColor);

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
  const x = (ply: number): number => chartX(ply, total(), plotW());
  const y = (win: number): number => chartY(win);
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
  const linePath = createMemo(() => buildLinePath(points()));
  /** Move phase at this ply: the chart backdrop is split by phase. */
  const phaseOf = (ply: number): Phase =>
    props.moves[ply <= 0 ? 0 : ply - 1]?.phase ?? "opening";

  const phaseRuns = createMemo(() => buildPhaseRuns(points(), phaseOf));
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
  const markers = createMemo(() => markerPoints(props.moves, isPlyInScope));
  const phaseMarks = createMemo(() => phaseMarksFor(props.moves, x));
  const scopedMoves = createMemo(() => scopedMovesFor(props.moves, isPlyInScope));
  const histMax = createMemo(() => maxMoveTime(scopedMoves()));

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
        role="img"
        tabIndex={0}
        aria-label={t("studio.chartLabel")}
        onPointerMove={handleMove}
        onPointerLeave={() => setHover(null)}
        onClick={(event) => props.onSelect(plyFromEvent(event))}
        onKeyDown={(event) => {
          if (event.key !== "ArrowLeft" && event.key !== "ArrowRight" && event.key !== "Home" && event.key !== "End") {
            return;
          }
          event.preventDefault();
          const target =
            event.key === "Home"
              ? 0
              : event.key === "End"
                ? total()
                : Math.max(0, Math.min(total(), props.currentPly + (event.key === "ArrowRight" ? 1 : -1)));
          props.onSelect(target);
        }}
      >
        <defs>
          <For each={["opening", "middlegame", "endgame"] as Phase[]}>
            {(phase) => (
              <linearGradient id={`eval-fill-${phase}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color={PHASE_COLORS[phase]} stop-opacity="0.36" />
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

        <TimeHistogram
          moves={props.moves}
          total={total()}
          currentPly={props.currentPly}
          isPlyInScope={isPlyInScope}
          histMax={histMax()}
          x={x}
          plotW={plotW()}
          width={width()}
        />

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
        {(state) => <EvalChartTooltip state={state()} />}
      </Show>
    </div>
  );
}
