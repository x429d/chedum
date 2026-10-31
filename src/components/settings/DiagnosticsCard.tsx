import { t } from "../../i18n";
import { api } from "../../services/api";
import { settingsStore } from "../../stores/settingsStore";
import { ui } from "../../stores/uiStore";
import { toErrorMessage } from "../../utils/format";
import { Icon } from "../ui/Icon";

async function copyText(text: string): Promise<void> {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  const area = document.createElement("textarea");
  area.value = text;
  area.style.position = "fixed";
  area.style.opacity = "0";
  document.body.appendChild(area);
  area.select();
  document.execCommand("copy");
  area.remove();
}

export function DiagnosticsCard() {
  const copy = async (): Promise<void> => {
    try {
      await copyText(await api.getDiagnostics());
      ui.notify(t("settings.diagnosticsCopied"), "success");
    } catch (error) {
      ui.notify(toErrorMessage(error), "error");
    }
  };

  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">{t("settings.diagnosticsTitle")}</div>
        </div>
        <Icon name="info" size={16} />
      </div>
      <div class="card-pad" style={{ display: "flex", "flex-direction": "column", gap: "8px" }}>
        <div class="form-hint">
          {t("settings.dbPath")}: <span class="mono">{settingsStore.info()?.dbPath ?? "—"}</span>
        </div>
        <div class="form-hint">
          {t("settings.logPath")}: <span class="mono">{settingsStore.info()?.logPath ?? "—"}</span>
        </div>
        <button type="button" class="btn btn-sm" style={{ "align-self": "flex-start" }} onClick={() => void copy()}>
          <Icon name="copy" size={13} />
          {t("settings.copyDiagnostics")}
        </button>
      </div>
    </div>
  );
}
