import { Match, Switch } from "solid-js";
import type { Role } from "chessops/types";

import type { Color } from "../../types/models";

function Body(props: { role: Role }) {
  return (
    <Switch>
      <Match when={props.role === "pawn"}>
        <circle cx="22.5" cy="12.2" r="5.2" />
        <path d="M18.4 18.6c1.9 4.1 1.7 8.1-1.1 13.1-1.4 2.5-2.6 3.7-3.7 5.8h17.8c-1.1-2.1-2.3-3.3-3.7-5.8-2.8-5-3-9-1.1-13.1z" />
        <rect x="11.2" y="36.2" width="22.6" height="3.8" rx="1.9" />
      </Match>
      <Match when={props.role === "rook"}>
        <path d="M11 8.5h5v4.8h4V8.5h5v4.8h4V8.5h5v9.6l-4 4v9.4h4v4.5H11v-4.5h4v-9.4l-4-4z" />
        <rect x="10" y="36.2" width="25" height="4.2" rx="2" />
      </Match>
      <Match when={props.role === "knight"}>
        <path d="M14 37.5c0-6 4-9.2 8.2-13.1 3-2.8 4.2-6.8 2.1-10.7l-6.4 3.2-3.1-8.2 10.7-4.2c8.2 3.2 13.1 11 13.1 20v9l-5.1 4v4H14z" />
        <path d="M22 14.2l-5.4 3.1M17.2 8.9c2.4 1.1 4.8 1.2 7.1.2" class="piece-detail" stroke-width="1.2" />
        <rect x="11.4" y="36.2" width="25" height="4.2" rx="2" />
      </Match>
      <Match when={props.role === "bishop"}>
        <circle cx="22.5" cy="6.9" r="2.2" />
        <path d="M22.5 9c-4.1 4.1-7.1 8.2-7.1 12.2 0 4 3 7.1 7.1 8.2 4.1-1.1 7.1-4.2 7.1-8.2 0-4-3-8.1-7.1-12.2z" />
        <path d="M22.5 15.2v9.1M18 19.7h9" class="piece-detail" stroke-width="1.4" />
        <rect x="16.2" y="30.1" width="12.6" height="3.2" rx="1.5" />
        <rect x="11.4" y="36.2" width="22.2" height="4.2" rx="2" />
      </Match>
      <Match when={props.role === "queen"}>
        <path d="M8.7 15.1l4.2 8.6 3.6-11.7 3.1 11.7 2.9-11.7 3.1 11.7 3.6-11.7 4.2 8.6-2.1 11.3H10.8z" />
        <circle cx="8.7" cy="13" r="1.8" />
        <circle cx="15.8" cy="9.4" r="1.8" />
        <circle cx="22.5" cy="7.9" r="1.8" />
        <circle cx="29.2" cy="9.4" r="1.8" />
        <circle cx="36.3" cy="13" r="1.8" />
        <rect x="13.5" y="29.8" width="18" height="3.2" rx="1.5" />
        <rect x="9.6" y="36.2" width="25.8" height="4.2" rx="2" />
      </Match>
      <Match when={props.role === "king"}>
        <path d="M22.5 3.6v8.2M18.4 7.7h8.2" class="piece-detail" stroke-width="2.6" />
        <path d="M13 18c0-4.2 4.1-6.2 9.5-6.2S32 13.8 32 18l2.2 11.2H10.8z" />
        <rect x="13.5" y="29.8" width="18" height="3.2" rx="1.5" />
        <rect x="9.6" y="36.2" width="25.8" height="4.2" rx="2" />
      </Match>
    </Switch>
  );
}

export function PieceGlyph(props: { role: Role; color: Color }) {
  return (
    <g class={props.color === "white" ? "piece-white" : "piece-black"}>
      <Body role={props.role} />
      <circle
        cx="28.6"
        cy="12.4"
        r="1.45"
        class={props.color === "white" ? "piece-eye-white" : "piece-eye-black"}
        style={{ display: props.role === "knight" ? "block" : "none" }}
      />
    </g>
  );
}
