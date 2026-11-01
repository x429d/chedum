import { For, onCleanup, onMount, type JSX } from "solid-js";

import { MASONRY_GAP, packMasonry } from "../../utils/masonry";

/**
 * Two-column masonry for the dashboard.
 *
 * A plain CSS grid aligns rows, so one short card leaves a hole under the whole
 * row when its neighbour is tall. Here each card is placed into whichever column
 * is currently shorter, and the row span is derived from its measured height.
 * The layout re-runs whenever a card or the container changes size, so content
 * updates (more accounts, longer opening names, another sort order) never leave
 * a gap in the middle of the page.
 *
 * Column count still comes from CSS (`grid-template-columns`), which keeps the
 * single-column breakpoint in the stylesheet where the rest of them live.
 */
export function Masonry(props: { children: JSX.Element }) {
  let container: HTMLDivElement | undefined;
  let observer: ResizeObserver | undefined;
  let frame = 0;
  const nodes: HTMLElement[] = [];

  const place = (): void => {
    const root = container;
    if (!root) return;
    const columns = getComputedStyle(root).gridTemplateColumns.split(" ").filter(Boolean).length;
    for (const node of nodes) {
      node.style.gridColumn = "";
      node.style.gridRowStart = "";
      node.style.gridRowEnd = "";
    }
    if (columns <= 1) return;

    const heights = nodes.map((node) => node.getBoundingClientRect().height);
    const slots = packMasonry(heights, columns, MASONRY_GAP);
    nodes.forEach((node, index) => {
      const slot = slots[index];
      if (!slot) return;
      node.style.gridColumn = String(slot.column);
      node.style.gridRowStart = String(slot.start);
      node.style.gridRowEnd = `span ${slot.span}`;
    });
  };

  const schedule = (): void => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(place);
  };

  const register = (node: HTMLElement): void => {
    if (nodes.includes(node)) return;
    nodes.push(node);
    observer?.observe(node);
    schedule();
  };

  onMount(() => {
    observer = new ResizeObserver(schedule);
    if (container) observer.observe(container);
    for (const node of nodes) observer.observe(node);
    // Place synchronously so the first paint is already the masonry layout.
    place();
  });

  onCleanup(() => {
    cancelAnimationFrame(frame);
    observer?.disconnect();
  });

  const children = (): JSX.Element[] => {
    const value = props.children;
    return (Array.isArray(value) ? value : [value]).filter(
      (item): item is JSX.Element => item !== undefined && item !== null && item !== false,
    );
  };

  return (
    <div class="masonry" ref={container}>
      <For each={children()}>
        {(child) => (
          <div class="masonry-item" ref={register}>
            {child}
          </div>
        )}
      </For>
    </div>
  );
}
