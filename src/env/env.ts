/* src/env/env.ts — service overlay. Edit defaults here; regenerate generated.ts from flags-2-env. */

import * as generated from "./generated.ts";

const defaults: Record<string, string> = {};

export default {
  get env(): Record<string, string> {
    return load();
  },
};

export function load(
  shell: Record<string, string | undefined> = typeof process !== "undefined" ? process.env : {},
): Record<string, string> {
  return Object.assign({}, defaults, generated.loadEnvMapFromOs(shell));
}

export { generated };
