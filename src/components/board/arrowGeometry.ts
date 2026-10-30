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

/**
 * Отступы подобраны так, чтобы хвост выходил из-под фигуры, а острие мягко
 * останавливалось перед центром цели. На коротких ходах отступы ужимаются
 * пропорционально: иначе стрелка на одну клетку превращается в огрызок.
 */
const TAIL_GAP = 0.3;
const TIP_GAP = 0.15;
/** Максимальная длина наконечника. */
const HEAD_LENGTH = 0.32;
/** Полуширина наконечника относительно его длины. */
const HEAD_RATIO = 0.55;
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
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const length = Math.hypot(dx, dy);
  if (length < 0.05) return null;

  const knight = isKnight(from, to);
  const corner = knight
    ? Math.abs(dx) > Math.abs(dy)
      ? { x: to.x, y: from.y }
      : { x: from.x, y: to.y }
    : null;

  const origin = corner ?? to;
  const first = unit(origin.x - from.x, origin.y - from.y);
  const last = unit(to.x - (corner?.x ?? from.x), to.y - (corner?.y ?? from.y));

  const tail = Math.min(TAIL_GAP, length * 0.22);
  // Наконечник не должен съесть всё древко, поэтому его длина ограничена остатком.
  const headLength = Math.min(HEAD_LENGTH, Math.max(0.18, (length - tail) * 0.5));
  const tip = Math.min(TIP_GAP, length * 0.1);
  const headWidth = headLength * HEAD_RATIO;

  const start = { x: from.x + first.x * tail, y: from.y + first.y * tail };
  const tipPoint = { x: to.x - last.x * tip, y: to.y - last.y * tip };
  const base = { x: to.x - last.x * (tip + headLength), y: to.y - last.y * (tip + headLength) };
  const sideX = -last.y;
  const sideY = last.x;
  const head = [
    `${round(tipPoint.x)},${round(tipPoint.y)}`,
    `${round(base.x + sideX * headWidth)},${round(base.y + sideY * headWidth)}`,
    `${round(base.x - sideX * headWidth)},${round(base.y - sideY * headWidth)}`,
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
