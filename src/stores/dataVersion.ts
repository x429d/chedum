import { createSignal } from "solid-js";

/** Bumped whenever analysis results change, so analytics views can refresh
 * even when the total number of games stays the same. */
const [dataVersion, setDataVersion] = createSignal(0);

export function bumpDataVersion(): void {
  setDataVersion((value) => value + 1);
}

export { dataVersion };
