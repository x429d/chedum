export interface ArrowPoint {
  x: number;
  y: number;
}

export interface ArrowGeometry {
  /** SVG-путь древка: прямой отрезок или «буква Г» для коня. */
  shaft: string;
  /** Точки наконечника. */
  head: string;
}

/** Отступ начала стрелки от центра клетки, в клетках. */
const START_GAP = 0.24;
/** Отступ острия от центра целевой клетки. */
const TIP_GAP = 0.06;
/** Длина наконечника. */
const HEAD_BASE = 0.34;
/** Полуширина наконечника. */
const HEAD_WIDTH = 0.18;
/** Радиус скругления излома стрелки коня. */
const BEND_RADIUS = 0.34;

const round = (value: number): number => Math.round(value * 1000) / 1000;

function unit(dx: number, dy: number): ArrowPoint & { len: number } {
  const len = Math.hypot(dx, dy) || 1;
  return { x: dx / len, y: dy / len, len };
}

/** Ход коня: две клетки по одной оси и одна по другой. */
function isKnight(from: ArrowPoint, to: ArrowPoint): boolean {
  const ax = Math.abs(to.x - from.x);
  const ay = Math.abs(to.y - from.y);
  return (
    (Math.abs(ax - 2) < 0.01 && Math.abs(ay - 1) < 0.01) ||
    (Math.abs(ax - 1) < 0.01 && Math.abs(ay - 2) < 0.01)
  );
}

/**
 * Путь стрелки между центрами клеток. Конь идёт «буквой Г»: сначала две
 * клетки по длинной оси, затем одна по короткой — как на шахматной доске.
 */
export function arrowGeometry(from: ArrowPoint, to: ArrowPoint): ArrowGeometry | null {
  if (Math.hypot(to.x - from.x, to.y - from.y) < 0.05) return null;

  const knight = isKnight(from, to);
  const corner = knight
    ? Math.abs(to.x - from.x) > Math.abs(to.y - from.y)
      ? { x: to.x, y: from.y }
      : { x: from.x, y: to.y }
    : null;

  const origin = corner ?? to;
  const first = unit(origin.x - from.x, origin.y - from.y);
  const last = unit(to.x - (corner?.x ?? from.x), to.y - (corner?.y ?? from.y));

  const start = { x: from.x + first.x * START_GAP, y: from.y + first.y * START_GAP };
  const tip = { x: to.x - last.x * TIP_GAP, y: to.y - last.y * TIP_GAP };
  const base = { x: to.x - last.x * HEAD_BASE, y: to.y - last.y * HEAD_BASE };
  const sideX = -last.y;
  const sideY = last.x;
  const head = [
    `${round(tip.x)},${round(tip.y)}`,
    `${round(base.x + sideX * HEAD_WIDTH)},${round(base.y + sideY * HEAD_WIDTH)}`,
    `${round(base.x - sideX * HEAD_WIDTH)},${round(base.y - sideY * HEAD_WIDTH)}`,
  ].join(" ");

  if (!corner) {
    return {
      shaft: `M ${round(start.x)} ${round(start.y)} L ${round(base.x)} ${round(base.y)}`,
      head,
    };
  }

  const radius = Math.min(BEND_RADIUS, first.len * 0.45, last.len * 0.45);
  const beforeCorner = {
    x: corner.x - first.x * radius,
    y: corner.y - first.y * radius,
  };
  const afterCorner = {
    x: corner.x + last.x * radius,
    y: corner.y + last.y * radius,
  };

  return {
    shaft:
      `M ${round(start.x)} ${round(start.y)} ` +
      `L ${round(beforeCorner.x)} ${round(beforeCorner.y)} ` +
      `Q ${round(corner.x)} ${round(corner.y)} ${round(afterCorner.x)} ${round(afterCorner.y)} ` +
      `L ${round(base.x)} ${round(base.y)}`,
    head,
  };
}
