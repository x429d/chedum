import { For } from "solid-js";

export type IconName =
  | "library"
  | "analysis"
  | "hub"
  | "settings"
  | "import"
  | "export"
  | "refresh"
  | "play"
  | "pause"
  | "chevron-left"
  | "chevron-right"
  | "chevrons-left"
  | "chevrons-right"
  | "flip"
  | "expand"
  | "shrink"
  | "arrows"
  | "star"
  | "trash"
  | "plus"
  | "check"
  | "close"
  | "clock"
  | "target"
  | "sparkles"
  | "search"
  | "filter"
  | "download"
  | "upload"
  | "sync"
  | "cpu"
  | "user"
  | "board"
  | "info"
  | "alert"
  | "eye"
  | "note"
  | "retry"
  | "book"
  | "activity";

const PATHS: Record<IconName, string[]> = {
  library: ["M4 4h5v16H4z", "M10.5 4h4v16h-4z", "M16 5.5l4 15"],
  analysis: ["M4 20V4", "M4 20h16", "M8 17v-5", "M12 17V8", "M16 17v-3", "M20 17V6"],
  hub: ["M21 12a9 9 0 1 1-9-9", "M12 3v9h9", "M12 12l5 5"],
  settings: ["M4 7h16", "M4 12h16", "M4 17h16", "M8 4.5v5", "M16 9.5v5", "M10 14.5v5"],
  import: ["M12 3v11", "M8 10l4 4 4-4", "M4 17v3h16v-3"],
  export: ["M12 15V4", "M8 8l4-4 4 4", "M4 17v3h16v-3"],
  refresh: ["M20 11a8 8 0 1 0-2.3 6.1", "M20 4v7h-7"],
  play: ["M8 5l11 7-11 7z"],
  pause: ["M8 5v14", "M16 5v14"],
  "chevron-left": ["M14 6l-6 6 6 6"],
  "chevron-right": ["M10 6l6 6-6 6"],
  "chevrons-left": ["M18 6l-6 6 6 6", "M11 6l-6 6 6 6"],
  "chevrons-right": ["M6 6l6 6-6 6", "M13 6l6 6-6 6"],
  flip: ["M4 8h13l-3-3", "M20 16H7l3 3", "M4 8v4", "M20 16v-4"],
  expand: ["M8 3H3v5", "M16 3h5v5", "M8 21H3v-5", "M16 21h5v-5"],
  shrink: ["M3 8h5V3", "M21 8h-5V3", "M3 16h5v5", "M21 16h-5v5"],
  arrows: ["M4 20L20 4", "M14 4h6v6"],
  star: ["M12 3l2.8 5.7 6.2.9-4.5 4.4 1.1 6.2L12 17.3 6.4 20.2l1.1-6.2L3 9.6l6.2-.9z"],
  trash: ["M4 7h16", "M9 7V4h6v3", "M6 7l1 13h10l1-13", "M10 11v5", "M14 11v5"],
  plus: ["M12 5v14", "M5 12h14"],
  check: ["M5 13l4 4L19 7"],
  close: ["M6 6l12 12", "M18 6L6 18"],
  clock: ["M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z", "M12 7v5l3 2"],
  target: ["M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z", "M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10z", "M12 13a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"],
  sparkles: ["M12 3l1.7 5.3L19 10l-5.3 1.7L12 17l-1.7-5.3L5 10l5.3-1.7z", "M19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8z"],
  search: ["M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z", "M21 21l-4.3-4.3"],
  filter: ["M3 5h18l-7 8v6l-4 2v-8z"],
  download: ["M12 3v12", "M7 10l5 5 5-5", "M4 21h16"],
  upload: ["M12 16V4", "M7 9l5-5 5 5", "M4 21h16"],
  sync: ["M20 7h-5V2", "M20 7a8 8 0 0 0-14.5-2", "M4 17h5v5", "M4 17a8 8 0 0 0 14.5 2"],
  cpu: ["M6 6h12v12H6z", "M9 9h6v6H9z", "M9 2v4", "M15 2v4", "M9 18v4", "M15 18v4", "M2 9h4", "M2 15h4", "M18 9h4", "M18 15h4"],
  user: ["M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8z", "M4 21a8 8 0 0 1 16 0"],
  board: ["M3 3h18v18H3z", "M3 9h18", "M3 15h18", "M9 3v18", "M15 3v18"],
  info: ["M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z", "M12 11v6", "M12 7h.01"],
  alert: ["M12 3L2 21h20z", "M12 9v5", "M12 17h.01"],
  eye: ["M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z", "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"],
  note: ["M5 3h10l4 4v14H5z", "M9 8h5", "M9 12h6", "M9 16h4"],
  retry: ["M4 12a8 8 0 1 0 2.3-5.6", "M4 4v6h6", "M12 8v5l3 2"],
  book: ["M4 4h7v16H4z", "M11 4h9v16h-9z", "M13 8h5", "M13 12h5"],
  activity: ["M3 12h4l3-8 4 16 3-8h4"],
};

export function Icon(props: { name: IconName; size?: number; class?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width={props.size ?? 18}
      height={props.size ?? 18}
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
      class={props.class}
      aria-hidden="true"
    >
      <For each={PATHS[props.name]}>{(path) => <path d={path} />}</For>
    </svg>
  );
}
