import { For, Show } from "solid-js";

import { t } from "../../i18n";
import { libraryStore } from "../../stores/libraryStore";
import { Icon } from "../ui/Icon";

const PAGE_SIZES = [25, 50, 100];

export function Pagination() {
  const offset = () => libraryStore.filter().offset ?? 0;
  const start = () => (libraryStore.total() === 0 ? 0 : offset() + 1);
  const end = () => offset() + libraryStore.games().length;
  const page = () => libraryStore.pageIndex();
  const pages = () => libraryStore.pageCount();

  return (
    <Show when={libraryStore.total() > 0}>
      <div class="pagination-bar">
        <span class="muted">
          {start()}–{end()} {t("library.of")} {libraryStore.total()}
        </span>
        <span class="spacer" />
        <label class="pagination-size">
          {t("library.perPage")}
          <select
            class="select"
            value={String(libraryStore.pageSize())}
            onChange={(event) => libraryStore.setPageSize(Number(event.currentTarget.value))}
          >
            <For each={PAGE_SIZES}>
              {(size) => <option value={String(size)}>{size}</option>}
            </For>
          </select>
        </label>
        <button
          type="button"
          class="btn btn-ghost btn-sm icon-btn"
          title={t("library.firstPage")}
          aria-label={t("library.firstPage")}
          disabled={page() === 0 || libraryStore.loading()}
          onClick={() => libraryStore.setPage(0)}
        >
          <Icon name="chevrons-left" size={13} />
        </button>
        <button
          type="button"
          class="btn btn-ghost btn-sm icon-btn"
          title={t("library.prevPage")}
          aria-label={t("library.prevPage")}
          disabled={page() === 0 || libraryStore.loading()}
          onClick={() => libraryStore.setPage(page() - 1)}
        >
          <Icon name="chevron-left" size={13} />
        </button>
        <span class="mono pagination-label">
          {page() + 1} / {pages()}
        </span>
        <button
          type="button"
          class="btn btn-ghost btn-sm icon-btn"
          title={t("library.nextPage")}
          aria-label={t("library.nextPage")}
          disabled={page() >= pages() - 1 || libraryStore.loading()}
          onClick={() => libraryStore.setPage(page() + 1)}
        >
          <Icon name="chevron-right" size={13} />
        </button>
        <button
          type="button"
          class="btn btn-ghost btn-sm icon-btn"
          title={t("library.lastPage")}
          aria-label={t("library.lastPage")}
          disabled={page() >= pages() - 1 || libraryStore.loading()}
          onClick={() => libraryStore.setPage(pages() - 1)}
        >
          <Icon name="chevrons-right" size={13} />
        </button>
      </div>
    </Show>
  );
}
