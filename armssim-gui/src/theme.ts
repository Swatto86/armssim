import { loadTheme, saveTheme, type ThemeChoice } from "./state";

function apply(theme: ThemeChoice): void {
  const resolved =
    theme === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : theme;
  document.documentElement.dataset.theme = resolved;
}

export function initTheme(select: HTMLSelectElement): void {
  const current = loadTheme();
  select.value = current;
  apply(current);

  select.addEventListener("change", () => {
    const choice = select.value as ThemeChoice;
    saveTheme(choice);
    apply(choice);
  });

  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
    if (loadTheme() === "system") apply("system");
  });
}
