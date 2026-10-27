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
    <div class="board-toolbar" title="Листайте партию стрелками ← →">
      <button type="button" class="btn btn-ghost btn-sm icon-btn" title="Начало" onClick={props.onFirst}>
        <Icon name="chevrons-left" />
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        title="Предыдущий ход"
        onClick={props.onPrevious}
      >
        <Icon name="chevron-left" />
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        title="Следующий ход"
        onClick={props.onNext}
      >
        <Icon name="chevron-right" />
      </button>
      <button type="button" class="btn btn-ghost btn-sm icon-btn" title="Конец" onClick={props.onLast}>
        <Icon name="chevrons-right" />
      </button>
      <span class="toolbar-label mono">
        {props.ply} / {props.total}
      </span>
      <span class="spacer" />
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        title="Перевернуть доску"
        onClick={props.onFlip}
      >
        <Icon name="flip" />
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm"
        title="Тема доски"
        onClick={props.onTheme}
      >
        <Icon name="board" />
        <span style={{ "text-transform": "capitalize" }}>{props.theme}</span>
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        classList={{ active: props.showArrows }}
        title="Стрелки линий"
        onClick={props.onArrows}
      >
        <Icon name="arrows" />
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm"
        classList={{ active: props.liveEnabled }}
        title="Live-анализ Stockfish"
        disabled={props.liveDisabled}
        onClick={props.onLive}
      >
        <Icon name="activity" />
        <span>Live</span>
      </button>
      <button
        type="button"
        class="btn btn-ghost btn-sm icon-btn"
        title={props.fullscreen ? "Выйти из полноэкранного режима" : "Полноэкранный режим"}
        onClick={props.onFullscreen}
      >
        <Icon name={props.fullscreen ? "shrink" : "expand"} />
      </button>
    </div>
  );
}
