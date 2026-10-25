// The theme the window is showing. Reading it inside a component, a `$derived` or an `$effect`
// tracks it, so everything drawn from it redraws when the mode changes.

import { themes, type Theme } from "./theme";

export type ThemeMode = "light" | "dark";

let currentMode: ThemeMode = $state("light");

export function themeMode(): ThemeMode {
  return currentMode;
}

export function setThemeMode(mode: ThemeMode): void {
  currentMode = mode;
}

export function toggleThemeMode(): void {
  if (currentMode === "light") {
    currentMode = "dark";
    return;
  }
  currentMode = "light";
}

export function currentTheme(): Theme {
  return themes[currentMode];
}
