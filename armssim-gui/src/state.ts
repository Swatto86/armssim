// Mirrors armssim-gui/src-tauri/src/progress.rs (serde camelCase).

export type OnlySetting = "all" | "st" | "blend" | "aoe";

export interface RunSettings {
  iterations: number;
  finalIterations: number;
  aoeFraction: number;
  only: OnlySetting;
  seed: number;
  jobs: number | null;
  noRefine: boolean;
}

export interface SlotChangeDto {
  label: string;
  from: string;
  to: string;
}

export interface RefineChangeDto {
  label: string;
  what: string;
  from: string;
  to: string;
}

export interface ObjectiveResult {
  name: string;
  stDps: number;
  aoeDps: number;
  stDelta: number;
  stDeltaPct: number;
  aoeDelta: number;
  aoeDeltaPct: number;
  searchSims: number;
  refineSims: number;
  changes: SlotChangeDto[];
  refinements: RefineChangeDto[];
  missing: string[];
}

export interface RunResult {
  characterName: string;
  characterRace: string;
  candidatePool: number;
  skipped: number;
  baseSt: number;
  baseAoe: number;
  objectives: ObjectiveResult[];
}

export interface ProgressEvent {
  objective: string;
  sims: number;
}

// cli.rs::Args' defaults.
export const DEFAULT_SETTINGS: RunSettings = {
  iterations: 1500,
  finalIterations: 20000,
  aoeFraction: 0.3,
  only: "all",
  seed: 1,
  jobs: null,
  noRefine: false,
};

const CHARACTER_KEY = "armssim.characterJson";
const SETTINGS_KEY = "armssim.settings";
const THEME_KEY = "armssim.theme";

export function loadCharacterJson(): string {
  return localStorage.getItem(CHARACTER_KEY) ?? "";
}

export function saveCharacterJson(value: string): void {
  localStorage.setItem(CHARACTER_KEY, value);
}

export function loadSettings(): RunSettings {
  const raw = localStorage.getItem(SETTINGS_KEY);
  if (!raw) return { ...DEFAULT_SETTINGS };
  try {
    return { ...DEFAULT_SETTINGS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export function saveSettings(settings: RunSettings): void {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
}

export type ThemeChoice = "system" | "light" | "dark";

export function loadTheme(): ThemeChoice {
  const raw = localStorage.getItem(THEME_KEY);
  return raw === "light" || raw === "dark" ? raw : "system";
}

export function saveTheme(theme: ThemeChoice): void {
  localStorage.setItem(THEME_KEY, theme);
}
