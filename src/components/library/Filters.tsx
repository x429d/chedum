import { For, Show } from "solid-js";

import { t, tTimeClass } from "../../i18n";
import { libraryStore } from "../../stores/libraryStore";
import type { TimeClass } from "../../types/models";
import { Icon } from "../ui/Icon";

const TIME_CLASSES: TimeClass[] = ["bullet", "blitz", "rapid", "classical"];

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
    const endOfDay = key === "dateTo" ? 86_399 : 0;
    const timestamp = Math.floor(new Date(`${value}T00:00:00`).getTime() / 1000) + endOfDay;
    libraryStore.updateFilter({ [key]: Number.isFinite(timestamp) ? timestamp : null });
  };

  return (
    <div class="filter-bar">
      <input
        class="input filter-search"
        type="search"
        aria-label={t("library.searchPlaceholder")}
        placeholder={t("library.searchPlaceholder")}
        value={filter().search ?? ""}
        onInput={(event) => libraryStore.setSearch(event.currentTarget.value)}
      />

      <div class="segmented">
        <button
          type="button"
          aria-pressed={(filter().color ?? null) === null}
          classList={{ active: (filter().color ?? null) === null }}
          onClick={() => libraryStore.updateFilter({ color: null })}
        >
          {t("library.all")}
        </button>
        <button
          type="button"
          aria-pressed={filter().color === "white"}
          classList={{ active: filter().color === "white" }}
          onClick={() => libraryStore.updateFilter({ color: "white" })}
        >
          {t("library.white")}
        </button>
        <button
          type="button"
          aria-pressed={filter().color === "black"}
          classList={{ active: filter().color === "black" }}
          onClick={() => libraryStore.updateFilter({ color: "black" })}
        >
          {t("library.black")}
        </button>
      </div>

      <select
        class="select"
        style={{ width: "132px" }}
        aria-label={t("library.result")}
        value={filter().result ?? ""}
        onChange={(event) =>
          libraryStore.updateFilter({ result: event.currentTarget.value || null })
        }
      >
        <option value="">{t("library.result")}</option>
        <option value="win">{t("library.wins")}</option>
        <option value="loss">{t("library.losses")}</option>
        <option value="draw">{t("library.draws")}</option>
      </select>

      <select
        class="select"
        style={{ width: "120px" }}
        aria-label={t("library.timeClass")}
        value={filter().timeClass ?? ""}
        onChange={(event) =>
          libraryStore.updateFilter({
            timeClass: (event.currentTarget.value || null) as TimeClass | null,
          })
        }
      >
        <option value="">{t("library.timeClass")}</option>
        <For each={TIME_CLASSES}>
          {(val) => <option value={val}>{tTimeClass(val)}</option>}
        </For>
      </select>

      <input
        class="input"
        style={{ width: "84px" }}
        aria-label={t("hub.eco")}
        placeholder="ECO"
        value={filter().eco ?? ""}
        onChange={(event) =>
          libraryStore.updateFilter({
            eco: event.currentTarget.value.trim() ? event.currentTarget.value.trim().toUpperCase() : null,
          })
        }
      />

      <div class="date-field">
        <input
          class="input"
          style={{ width: "142px" }}
          type="date"
          title={t("library.fromDate")}
          aria-label={t("library.fromDate")}
          value={dateValue(filter().dateFrom)}
          onChange={(event) => setDate("dateFrom", event.currentTarget.value)}
        />
        <Show when={filter().dateFrom}>
          <button
            type="button"
            class="date-clear"
            title={t("library.clearStartDate")}
            aria-label={t("library.clearStartDate")}
            onClick={() => libraryStore.updateFilter({ dateFrom: null })}
          >
            <Icon name="close" size={11} />
          </button>
        </Show>
      </div>
      <div class="date-field">
        <input
          class="input"
          style={{ width: "142px" }}
          type="date"
          title={t("library.toDate")}
          aria-label={t("library.toDate")}
          value={dateValue(filter().dateTo)}
          onChange={(event) => setDate("dateTo", event.currentTarget.value)}
        />
        <Show when={filter().dateTo}>
          <button
            type="button"
            class="date-clear"
            title={t("library.clearEndDate")}
            aria-label={t("library.clearEndDate")}
            onClick={() => libraryStore.updateFilter({ dateTo: null })}
          >
            <Icon name="close" size={11} />
          </button>
        </Show>
      </div>

      <label class="chip" style={{ cursor: "pointer" }}>
        <input
          type="checkbox"
          checked={filter().analyzedOnly ?? false}
          onChange={(event) =>
            libraryStore.updateFilter({ analyzedOnly: event.currentTarget.checked })
          }
        />
        {t("library.withAnalysis")}
      </label>
      <label class="chip" style={{ cursor: "pointer" }}>
        <input
          type="checkbox"
          checked={filter().favoritesOnly ?? false}
          onChange={(event) =>
            libraryStore.updateFilter({ favoritesOnly: event.currentTarget.checked })
          }
        />
        {t("library.favorites")}
      </label>
      <label class="chip" style={{ cursor: "pointer" }}>
        <input
          type="checkbox"
          checked={filter().hasNotes ?? false}
          onChange={(event) => libraryStore.updateFilter({ hasNotes: event.currentTarget.checked })}
        />
        {t("library.withNotes")}
      </label>

      <button type="button" class="btn btn-ghost btn-sm" onClick={() => libraryStore.resetFilter()}>
        <Icon name="refresh" size={13} />
        {t("library.resetFilter")}
      </button>
    </div>
  );
}
