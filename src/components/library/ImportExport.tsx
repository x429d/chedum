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
        {libraryStore.importing() ? "Импорт…" : "Импорт PGN"}
      </button>
      <button type="button" class="btn" onClick={() => void libraryStore.exportSelection()}>
        <Icon name="export" size={14} />
        Экспорт
      </button>
      <button
        type="button"
        class="btn btn-ghost icon-btn"
        title="Обновить библиотеку"
        onClick={() => void libraryStore.load()}
      >
        <Icon name="refresh" size={15} />
      </button>
    </>
  );
}
