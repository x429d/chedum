import { For } from "solid-js";

import { Icon } from "../ui/Icon";
import { settingsStore } from "../../stores/settingsStore";

const DEPTHS = [8, 10, 12, 14, 16, 18, 20, 22];

export function AnalysisCard() {
  return (
          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">Локальный анализ</div>
                <div class="card-subtitle">Stockfish запускается как дочерний процесс</div>
              </div>
              <Icon name="cpu" size={16} />
            </div>
            <div class="card-pad">
              <div class="form-row">
                <label class="form-label" for="analysis-depth">
                  Глубина анализа
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
                <div class="form-hint">
                  Больше глубина — точнее классификация, но дольше разбор. 14 — хороший баланс для
                  рапида и блица.
                </div>
              </div>
              <div class="form-row">
                <span class="form-label">Потоки анализа</span>
                <div class="segmented">
                  <button
                    type="button"
                    classList={{ active: settingsStore.settings().analysisThreads === 0 }}
                    onClick={() => void settingsStore.update({ analysisThreads: 0 })}
                  >
                    Авто
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
                <div class="form-hint">
                  Сколько партий разбирать одновременно. Авто = производительные ядра − 1
                  (сейчас доступно {(settingsStore.info()?.maxAnalysisWorkers ?? 1)}). Каждый
                  процесс Stockfish получает Threads=1 и общий лимит памяти.
                </div>
              </div>
              <div class="form-row">
                <span class="form-label">MultiPV</span>
                <div class="segmented">
                  <For each={[1, 2, 3]}>
                    {(value) => (
                      <button
                        type="button"
                        classList={{ active: settingsStore.settings().multipv === value }}
                        onClick={() => void settingsStore.update({ multipv: value })}
                      >
                        {value}
                      </button>
                    )}
                  </For>
                </div>
                <div class="form-hint">
                  Две линии нужны для поиска критических позиций и Brilliant/Great. Три — для
                  более богатого дерева вариантов.
                </div>
              </div>
              <div class="toggle-row">
                <div>
                  <div style={{ "font-size": "12px", "font-weight": "600" }}>
                    Продолжать разбор при запуске
                  </div>
                  <div class="form-hint">
                    Если очередь не закончена, начать её автоматически после старта.
                  </div>
                </div>
                <button
                  type="button"
                  class="switch"
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
                    Автоанализ после импорта
                  </div>
                  <div class="form-hint">Ставить новые партии в очередь автоматически.</div>
                </div>
                <button
                  type="button"
                  class="switch"
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
