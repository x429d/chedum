import { For } from "solid-js";

import type { CategoryCounts, MoveCategory, UserColor } from "../../types/models";
import { CATEGORY_META, CATEGORY_ORDER } from "../../types/models";
import { CategoryGlyph } from "../ui/CategoryGlyph";

export function CategoryTable(props: {
  counts: CategoryCounts;
  userColor: UserColor;
  active: MoveCategory | null;
  onSelect: (category: MoveCategory) => void;
}) {
  return (
    <table class="category-table">
      <thead>
        <tr>
          <th style={{ "text-align": "left" }}>Категория</th>
          <th style={{ "text-align": "right" }}>
            {props.userColor === "none"
              ? "Белые"
              : props.userColor === "white"
                ? "Вы"
                : "Соперник"}
          </th>
          <th style={{ "text-align": "right" }}>
            {props.userColor === "none"
              ? "Чёрные"
              : props.userColor === "white"
                ? "Соперник"
                : "Вы"}
          </th>
        </tr>
      </thead>
      <tbody>
        <For each={CATEGORY_ORDER}>
          {(category) => {
            const meta = CATEGORY_META[category];
            const white = () => props.counts.white[category] ?? 0;
            const black = () => props.counts.black[category] ?? 0;
            return (
              <tr
                class="clickable"
                classList={{ selected: props.active === category }}
                onClick={() => props.onSelect(category)}
                title={`Показать ходы: ${meta.label}`}
              >
                <td>
                  <span class="category-label">
                    <span class="category-symbol" style={{ background: meta.color }}>
                      <CategoryGlyph category={category} size={12} />
                    </span>
                    {meta.label}
                  </span>
                </td>
                <td
                  class="mono"
                  style={{
                    "text-align": "right",
                    color: props.userColor === "white" ? "var(--text)" : "var(--muted)",
                  }}
                >
                  {white()}
                </td>
                <td
                  class="mono"
                  style={{
                    "text-align": "right",
                    color: props.userColor === "black" ? "var(--text)" : "var(--muted)",
                  }}
                >
                  {black()}
                </td>
              </tr>
            );
          }}
        </For>
      </tbody>
    </table>
  );
}
