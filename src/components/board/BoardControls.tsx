import { t } from "../../i18n";
import type { BoardTheme } from "../../types/models";
import { Icon } from "../ui/Icon";

export interface BoardControlsProps {
  ply: number;
  total: number;
  theme: BoardTheme;
  showArrows: boolean;
  liveEnabled: boolean;
  liveDisabled?: boolean;
  fullscreen: boolean;
  onFirst: () => void;
  onPrevious: () => void;
  onNext: () => void;
  onLast: () => void;
  onFlip: () => void;
  onTheme: () => void;
  onArrows: () => void;
  onLive: () => void;
  onFullscreen: () => void;
}

export function BoardControls(props: BoardControlsProps) {
  return (
    <div class="board-toolbar">
      <button type="button" class="btn btn-ghost btn-sm icon-btn" title={t("board.first")} aria-label={t("board.first")} onClick={props.onFirst}>
        <Icon name="chevrons-left" />
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        title={t("board.previous")}
        aria-label={t("board.previous")}
        onClick={props.onPrevious}
      >
        <Icon name="chevron-left" />
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        title={t("board.next")}
        aria-label={t("board.next")}
        onClick={props.onNext}
      >
        <Icon name="chevron-right" />
      </button>
      <button type="button" class="btn btn-ghost btn-sm icon-btn" title={t("board.last")} aria-label={t("board.last")} onClick={props.onLast}>
        <Icon name="chevrons-right" />
      </button>
      <span class="toolbar-label mono">
        {props.ply} / {props.total}
      </span>
      <span class="spacer" />
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        title={t("board.flip")}
        aria-label={t("board.flip")}
        onClick={props.onFlip}
      >
        <Icon name="flip" />
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm"
        title={t("board.theme")}
        aria-label={t("board.theme")}
        onClick={props.onTheme}
      >
        <Icon name="board" />
        <span style={{ "text-transform": "capitalize" }}>{props.theme}</span>
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        classList={{ active: props.showArrows }}
        title={t("board.arrows")}
        aria-label={t("board.arrows")}
        onClick={props.onArrows}
      >
        <Icon name="arrows" />
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm"
        classList={{ active: props.liveEnabled }}
        title={t("board.live")}
        aria-label={t("board.live")}
        disabled={props.liveDisabled}
        onClick={props.onLive}
      >
        <Icon name="activity" />
        <span>Live</span>
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        title={props.fullscreen ? t("board.exitFullscreen") : t("board.fullscreen")}
        aria-label={props.fullscreen ? t("board.exitFullscreen") : t("board.fullscreen")}
        onClick={props.onFullscreen}
      >
        <Icon name={props.fullscreen ? "shrink" : "expand"} />
      </button>
    </div>
  );
}
