import type { Color, MoveCategory } from "../../types/models";
import { CATEGORY_META } from "../../types/models";
import { squareCoords } from "../../utils/chess";
import { CategoryGlyph } from "../ui/CategoryGlyph";

const RADIUS = 0.195;
const GLYPH = 0.235;

export function CategoryBadge(props: {
  square: string;
  category: MoveCategory;
  orientation: Color;
}) {
  const coords = () => squareCoords(props.square);
  const meta = () => CATEGORY_META[props.category];
  const col = () => {
    const value = coords();
    if (!value) return 0;
    return props.orientation === "white" ? value.file : 7 - value.file;
  };
  const row = () => {
    const value = coords();
    if (!value) return 0;
    return props.orientation === "white" ? 7 - value.rank : value.rank;
  };
  const cx = () => col() + 0.785;
  const cy = () => row() + 0.215;
  return (
    <g style={{ "pointer-events": "none" }}>
      <circle
        cx={cx()}
        cy={cy()}
        r={RADIUS}
        fill={meta().color}
        stroke="#0b0e14"
        stroke-width="0.026"
      />
      <g
        color="#0b0e14"
        transform={`translate(${cx()} ${cy()}) scale(${GLYPH / 24}) translate(-12 -12)`}
      >
        <CategoryGlyph category={props.category} size={24} />
      </g>
    </g>
  );
}
