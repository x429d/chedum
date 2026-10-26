import { For } from "solid-js";

import { libraryStore } from "../../stores/libraryStore";
import type { TimeClass } from "../../types/models";
import { Icon } from "../ui/Icon";

const TIME_CLASSES: { value: TimeClass; label: string }[] = [
  { value: "bullet", label: "Пуля" },
  { value: "blitz", label: "Блиц" },
  { value: "rapid", label: "Рапид" },
  { value: "classical", label: "Классика" },
];

function dateValue(timestamp: number | null | undefined): string {
  if (!timestamp) return "";
  return new Date(timestamp * 1000).toISOString().slice(0, 10);
}

export function Filters() {
  const filter = () => libraryStore.filter();
  const setDate = (key: "dateFrom" | "dateTo", value: string): void => {
    if (!value) {
      libraryStore.updateFilter({ [key]: null });
      return;
    }
    const timestamp = Math.floor(new Date(`${value}T00:00:00`).getTime() / 1000);
    libraryStore.updateFilter({ [key]: Number.isFinite(timestamp) ? timestamp : null });
  };

  return (
    <div class="filter-bar">
      <input
        class="input filter-search"
        type="search"
        placeholder="Поиск: ник, дебют, ECO…"
        value={filter().search ?? ""}
        onInput={(event) => libraryStore.setSearch(event.currentTarget.value)}
      />

      <div class="segmented">
        <button
          type="button"
          classList={{ active: (filter().color ?? null) === null }}
          onClick={() => libraryStore.updateFilter({ color: null })}
        >
          Все
        </button>
        <button
          type="button"
          classList={{ active: filter().color === "white" }}
          onClick={() => libraryStore.updateFilter({ color: "white" })}
        >
          Белые
        </button>
        <button
          type="button"
          classList={{ active: filter().color === "black" }}
          onClick={() => libraryStore.updateFilter({ color: "black" })}
        >
          Чёрные
        </button>
      </div>

      <select
        class="select"
        style={{ width: "132px" }}
        value={filter().result ?? ""}
        onChange={(event) =>
          libraryStore.updateFilter({ result: event.currentTarget.value || null })
        }
      >
        <option value="">Результат</option>
        <option value="win">Победы</option>
        <option value="loss">Поражения</option>
        <option value="draw">Ничьи</option>
      </select>

      <select
        class="select"
        style={{ width: "120px" }}
        value={filter().timeClass ?? ""}
        onChange={(event) =>
          libraryStore.updateFilter({
            timeClass: (event.currentTarget.value || null) as TimeClass | null,
          })
        }
      >
        <option value="">Контроль</option>
        <For each={TIME_CLASSES}>
          {(item) => <option value={item.value}>{item.label}</option>}
        </For>
      </select>

      <input
        class="input"
        style={{ width: "84px" }}
        placeholder="ECO"
        value={filter().eco ?? ""}
        onChange={(event) =>
          libraryStore.updateFilter({
            eco: event.currentTarget.value.trim() ? event.currentTarget.value.trim().toUpperCase() : null,
          })
        }
      />

      <input
        class="input"
        style={{ width: "132px" }}
        type="date"
        title="С даты"
        value={dateValue(filter().dateFrom)}
        onChange={(event) => setDate("dateFrom", event.currentTarget.value)}
      />
      <input
        class="input"
        style={{ width: "132px" }}
        type="date"
        title="По дату"
        value={dateValue(filter().dateTo)}
        onChange={(event) => setDate("dateTo", event.currentTarget.value)}
      />

      <label class="chip" style={{ cursor: "pointer" }}>
        <input
          type="checkbox"
          checked={filter().analyzedOnly ?? false}
          onChange={(event) =>
            libraryStore.updateFilter({ analyzedOnly: event.currentTarget.checked })
          }
        />
        С анализом
      </label>
      <label class="chip" style={{ cursor: "pointer" }}>
        <input
          type="checkbox"
          checked={filter().favoritesOnly ?? false}
          onChange={(event) =>
            libraryStore.updateFilter({ favoritesOnly: event.currentTarget.checked })
          }
        />
        Избранные
      </label>
      <label class="chip" style={{ cursor: "pointer" }}>
        <input
          type="checkbox"
          checked={filter().hasNotes ?? false}
          onChange={(event) => libraryStore.updateFilter({ hasNotes: event.currentTarget.checked })}
        />
        С заметками
      </label>

      <button type="button" class="btn btn-ghost btn-sm" onClick={() => libraryStore.resetFilter()}>
        <Icon name="refresh" size={13} />
        Сбросить
      </button>
    </div>
  );
}
