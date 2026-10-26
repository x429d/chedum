import type { Color, MoveCategory } from "../../types/models";
import { CATEGORY_META } from "../../types/models";
import { squareCoords } from "../../utils/chess";

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
  return (
    <g style={{ "pointer-events": "none" }}>
      <circle
        cx={col() + 0.78}
        cy={row() + 0.22}
        r="0.205"
        fill={meta().color}
        stroke="#0b0e14"
        stroke-width="0.028"
      />
      <text
        x={col() + 0.78}
        y={row() + 0.225}
        text-anchor="middle"
        dominant-baseline="middle"
        font-size={meta().symbol.length > 1 ? "0.145" : "0.17"}
        font-weight="800"
        fill="#0b0e14"
      >
        {meta().symbol}
      </text>
    </g>
  );
}
