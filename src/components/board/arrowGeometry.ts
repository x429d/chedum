export interface ArrowPoint {
  x: number;
  y: number;
}

export interface ArrowGeometry {
  /** SVG path for the shaft: a straight segment or an L-shape for a knight. */
  shaft: string;
  /** Arrowhead points. */
  head: string;
}

/**
 * The gaps are tuned so the tail emerges from under the piece and the tip stops
 * just before the target center. On short moves the gaps shrink proportionally;
 * otherwise a one-square arrow becomes a stub.
 */
const TAIL_GAP = 0.3;
const TIP_GAP = 0.15;
/** Maximum arrowhead length. */
const HEAD_LENGTH = 0.32;
/** Arrowhead half-width relative to its length. */
const HEAD_RATIO = 0.55;
/** Corner rounding radius for a knight's arrow. */
const BEND_RADIUS = 0.34;

const round = (value: number): number => Math.round(value * 1000) / 1000;

function unit(dx: number, dy: number): ArrowPoint & { len: number } {
  const len = Math.hypot(dx, dy) || 1;
  return { x: dx / len, y: dy / len, len };
}

/** Knight move: two squares along one axis and one along the other. */
function isKnight(from: ArrowPoint, to: ArrowPoint): boolean {
  const ax = Math.abs(to.x - from.x);
  const ay = Math.abs(to.y - from.y);
  return (
    (Math.abs(ax - 2) < 0.01 && Math.abs(ay - 1) < 0.01) ||
    (Math.abs(ax - 1) < 0.01 && Math.abs(ay - 2) < 0.01)
  );
}

/**
 * Arrow path between square centers. A knight moves in an L: first two squares
 * along the long axis, then one along the short one, as on a chessboard.
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
  // The arrowhead must not consume the whole shaft, so its length is capped by what remains.
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
