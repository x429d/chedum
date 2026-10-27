import type { Component } from "solid-js";
import { Dynamic } from "solid-js/web";

import Activity from "lucide-solid/icons/activity";
import BookOpen from "lucide-solid/icons/book-open";
import ChartPie from "lucide-solid/icons/chart-pie";
import Check from "lucide-solid/icons/check";
import ChevronLeft from "lucide-solid/icons/chevron-left";
import ChevronRight from "lucide-solid/icons/chevron-right";
import ChevronsLeft from "lucide-solid/icons/chevrons-left";
import ChevronsRight from "lucide-solid/icons/chevrons-right";
import Clock from "lucide-solid/icons/clock";
import CloudDownload from "lucide-solid/icons/cloud-download";
import Cpu from "lucide-solid/icons/cpu";
import Download from "lucide-solid/icons/download";
import Eye from "lucide-solid/icons/eye";
import FileDown from "lucide-solid/icons/file-down";
import FileUp from "lucide-solid/icons/file-up";
import ArrowUpDown from "lucide-solid/icons/arrow-up-down";
import Funnel from "lucide-solid/icons/funnel";
import Grid3x3 from "lucide-solid/icons/grid-3x3";
import Info from "lucide-solid/icons/info";
import Library from "lucide-solid/icons/library";
import Maximize2 from "lucide-solid/icons/maximize-2";
import Minimize2 from "lucide-solid/icons/minimize-2";
import MoveUpRight from "lucide-solid/icons/move-up-right";
import NotebookPen from "lucide-solid/icons/notebook-pen";
import Pause from "lucide-solid/icons/pause";
import Play from "lucide-solid/icons/play";
import Plus from "lucide-solid/icons/plus";
import RotateCcw from "lucide-solid/icons/rotate-ccw";
import RotateCw from "lucide-solid/icons/rotate-cw";
import Search from "lucide-solid/icons/search";
import Settings2 from "lucide-solid/icons/settings-2";
import ChartNoAxesColumn from "lucide-solid/icons/chart-no-axes-column";
import Sparkles from "lucide-solid/icons/sparkles";
import Star from "lucide-solid/icons/star";
import Swords from "lucide-solid/icons/swords";
import Target from "lucide-solid/icons/target";
import Trash from "lucide-solid/icons/trash";
import TriangleAlert from "lucide-solid/icons/triangle-alert";
import Upload from "lucide-solid/icons/upload";
import User from "lucide-solid/icons/user";
import X from "lucide-solid/icons/x";

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
  | "analyze"
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

interface GlyphProps {
  size?: number;
  class?: string;
  strokeWidth?: number;
}

/** Lucide glyphs (ISC) mapped onto the app's semantic icon names. */
const GLYPHS: Record<IconName, Component<GlyphProps>> = {
  library: Library as Component<GlyphProps>,
  analysis: Swords as Component<GlyphProps>,
  hub: ChartPie as Component<GlyphProps>,
  settings: Settings2 as Component<GlyphProps>,
  import: FileDown as Component<GlyphProps>,
  export: FileUp as Component<GlyphProps>,
  refresh: RotateCw as Component<GlyphProps>,
  play: Play as Component<GlyphProps>,
  pause: Pause as Component<GlyphProps>,
  "chevron-left": ChevronLeft as Component<GlyphProps>,
  "chevron-right": ChevronRight as Component<GlyphProps>,
  "chevrons-left": ChevronsLeft as Component<GlyphProps>,
  "chevrons-right": ChevronsRight as Component<GlyphProps>,
  flip: ArrowUpDown as Component<GlyphProps>,
  expand: Maximize2 as Component<GlyphProps>,
  shrink: Minimize2 as Component<GlyphProps>,
  arrows: MoveUpRight as Component<GlyphProps>,
  star: Star as Component<GlyphProps>,
  trash: Trash as Component<GlyphProps>,
  plus: Plus as Component<GlyphProps>,
  check: Check as Component<GlyphProps>,
  close: X as Component<GlyphProps>,
  clock: Clock as Component<GlyphProps>,
  target: Target as Component<GlyphProps>,
  sparkles: Sparkles as Component<GlyphProps>,
  analyze: ChartNoAxesColumn as Component<GlyphProps>,
  search: Search as Component<GlyphProps>,
  filter: Funnel as Component<GlyphProps>,
  download: Download as Component<GlyphProps>,
  upload: Upload as Component<GlyphProps>,
  sync: CloudDownload as Component<GlyphProps>,
  cpu: Cpu as Component<GlyphProps>,
  user: User as Component<GlyphProps>,
  board: Grid3x3 as Component<GlyphProps>,
  info: Info as Component<GlyphProps>,
  alert: TriangleAlert as Component<GlyphProps>,
  eye: Eye as Component<GlyphProps>,
  note: NotebookPen as Component<GlyphProps>,
  retry: RotateCcw as Component<GlyphProps>,
  book: BookOpen as Component<GlyphProps>,
  activity: Activity as Component<GlyphProps>,
};

export function Icon(props: { name: IconName; size?: number; class?: string }) {
  return (
    <Dynamic
      component={GLYPHS[props.name]}
      size={props.size ?? 18}
      class={props.class}
      strokeWidth={1.75}
    />
  );
}
