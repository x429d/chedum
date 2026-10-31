import { t } from "../../i18n";
import { libraryStore } from "../../stores/libraryStore";
import { Icon } from "../ui/Icon";

export function ImportExport() {
  return (
    <>
      <button
        type="button"
        class="btn btn-primary"
        disabled={libraryStore.importing()}
        onClick={() => void libraryStore.importFiles()}
      >
        <Icon name="import" size={14} />
        {libraryStore.importing() ? t("library.importing") : t("library.importPgn")}
      </button>
      <button type="button" class="btn" onClick={() => void libraryStore.exportSelection()}>
        <Icon name="export" size={14} />
        {t("library.export")}
      </button>
      <button
        type="button"
        class="btn btn-ghost icon-btn"
        title={t("library.refresh")}
        aria-label={t("library.refresh")}
        onClick={() => void libraryStore.load()}
      >
        <Icon name="refresh" size={15} />
      </button>
    </>
  );
}
