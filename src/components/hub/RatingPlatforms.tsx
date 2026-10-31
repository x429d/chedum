import { For, Show } from "solid-js";

import { pluralGames, t } from "../../i18n";
import type { GameSource, PlatformRating } from "../../types/models";
import { formatNumber } from "../../utils/format";
import { Icon } from "../ui/Icon";

function platformLabel(source: GameSource): string {
  if (source === "chesscom") return t("accounts.platformChesscom");
  if (source === "lichess") return t("accounts.platformLichess");
  return t("hub.platformManual");
}

function accountLabel(rating: PlatformRating): string {
  return rating.accountUsername ? `@${rating.accountUsername}` : t("hub.ratingNoAccount");
}

function perfValue(rating: PlatformRating): string {
  if (rating.perfElo === null) return "—";
  return `${Math.round(rating.perfElo)} ± ${Math.round(rating.perfEloSpread ?? 0)}`;
}

export function RatingPlatforms(props: { ratings: PlatformRating[] }) {
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">{t("hub.ratingTitle")}</div>
          <div class="card-subtitle">{t("hub.ratingHint")}</div>
        </div>
        <Icon name="target" size={16} />
      </div>
      <div class="card-pad">
        <Show
          when={props.ratings.length > 0}
          fallback={
            <div class="empty-state" style={{ padding: "20px" }}>
              {t("hub.noData")}
            </div>
          }
        >
          <div class="rating-list">
            <For each={props.ratings}>
              {(rating) => (
                <div class="rating-row">
                  <div class="rating-account">
                    <span class={`platform-badge platform-${rating.platform}`}>
                      {platformLabel(rating.platform)}
                    </span>
                    <span class="rating-username" title={accountLabel(rating)}>
                      {accountLabel(rating)}
                    </span>
                  </div>
                  <div class="rating-metrics">
                    <div class="rating-metric">
                      <span>{t("hub.avgUserElo")}</span>
                      <strong>{formatNumber(rating.avgUserElo, 0)}</strong>
                    </div>
                    <div class="rating-metric">
                      <span>{t("hub.avgOpponentElo")}</span>
                      <strong>{formatNumber(rating.avgOpponentElo, 0)}</strong>
                    </div>
                    <div
                      class="rating-metric"
                      title={rating.perfElo === null ? t("hub.perfEloNeedMore") : undefined}
                    >
                      <span>{t("hub.perfElo")}</span>
                      <strong>{perfValue(rating)}</strong>
                    </div>
                  </div>
                  <div class="rating-games">
                    <span>{pluralGames(rating.games)}</span>
                    <span class="dim">
                      {t("hub.ratingSample", {
                        analyzed: rating.analyzed,
                        games: rating.games,
                      })}
                    </span>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
}
