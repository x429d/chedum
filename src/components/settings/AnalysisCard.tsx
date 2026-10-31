import { For } from "solid-js";

import { t } from "../../i18n";
import { settingsStore } from "../../stores/settingsStore";
import { Icon } from "../ui/Icon";

const DEPTHS = [8, 10, 12, 14, 16, 18, 20, 22, 24];

export function AnalysisCard() {
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">{t("settings.analysisTitle")}</div>
        </div>
        <Icon name="cpu" size={16} />
      </div>
      <div class="card-pad">
        <div class="form-row">
          <label class="form-label" for="analysis-depth">
            {t("settings.depthLabel")}
          </label>
          <select
            id="analysis-depth"
            class="select"
            value={String(settingsStore.settings().analysisDepth)}
            onChange={(event) =>
              void settingsStore.update({
                analysisDepth: Number(event.currentTarget.value),
              })
            }
          >
            <For each={DEPTHS}>
              {(depth) => <option value={String(depth)}>{depth}</option>}
            </For>
          </select>
        </div>
        <div class="form-row">
          <span class="form-label">{t("settings.threadsLabel")}</span>
          <div class="segmented">
            <button
              type="button"
              aria-pressed={settingsStore.settings().analysisThreads === 0}
              classList={{ active: settingsStore.settings().analysisThreads === 0 }}
              onClick={() => void settingsStore.update({ analysisThreads: 0 })}
            >
              {t("settings.autoThreads")}
            </button>
            <For
              each={Array.from(
                { length: settingsStore.info()?.maxAnalysisWorkers ?? 1 },
                (_, index) => index + 1,
              )}
            >
              {(value) => (
                <button
                  type="button"
                  aria-pressed={settingsStore.settings().analysisThreads === value}
                  classList={{
                    active: settingsStore.settings().analysisThreads === value,
                  }}
                  onClick={() => void settingsStore.update({ analysisThreads: value })}
                >
                  {value}
                </button>
              )}
            </For>
          </div>
        </div>
        <div class="form-row">
          <span class="form-label">MultiPV</span>
          <div class="segmented">
            <For each={[1, 2, 3]}>
              {(value) => (
                <button
                  type="button"
                  aria-pressed={settingsStore.settings().multipv === value}
                  classList={{ active: settingsStore.settings().multipv === value }}
                  onClick={() => void settingsStore.update({ multipv: value })}
                >
                  {value}
                </button>
              )}
            </For>
          </div>
        </div>
        <div class="toggle-row">
          <div>
            <div style={{ "font-size": "12px", "font-weight": "600" }}>
              {t("settings.autoResume")}
            </div>
          </div>
          <button
            type="button"
            class="switch"
            role="switch"
            aria-checked={settingsStore.settings().autoResumeAnalysis}
            aria-label={t("settings.autoResume")}
            classList={{ on: settingsStore.settings().autoResumeAnalysis }}
            onClick={() =>
              void settingsStore.update({
                autoResumeAnalysis: !settingsStore.settings().autoResumeAnalysis,
              })
            }
          />
        </div>
        <div class="toggle-row">
          <div>
            <div style={{ "font-size": "12px", "font-weight": "600" }}>
              {t("settings.autoAnalyze")}
            </div>
          </div>
          <button
            type="button"
            class="switch"
            role="switch"
            aria-checked={settingsStore.settings().autoAnalyze}
            aria-label={t("settings.autoAnalyze")}
            classList={{ on: settingsStore.settings().autoAnalyze }}
            onClick={() =>
              void settingsStore.update({
                autoAnalyze: !settingsStore.settings().autoAnalyze,
              })
            }
          />
        </div>
      </div>
    </div>
  );
}
