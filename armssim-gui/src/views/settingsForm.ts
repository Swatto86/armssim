import { invoke } from "@tauri-apps/api/core";
import { loadSettings, saveSettings, type OnlySetting, type RunSettings } from "../state";

export interface SettingsForm {
  read(): RunSettings | null;
}

export async function initSettingsForm(): Promise<SettingsForm> {
  const onlySelect = document.querySelector<HTMLSelectElement>("#setting-only")!;
  const aoeFractionInput = document.querySelector<HTMLInputElement>("#setting-aoe-fraction")!;
  const iterationsInput = document.querySelector<HTMLInputElement>("#setting-iterations")!;
  const finalIterationsInput = document.querySelector<HTMLInputElement>("#setting-final-iterations")!;
  const seedInput = document.querySelector<HTMLInputElement>("#setting-seed")!;
  const jobsInput = document.querySelector<HTMLInputElement>("#setting-jobs")!;
  const errorEl = document.querySelector<HTMLParagraphElement>("#settings-error")!;

  const settings = loadSettings();
  onlySelect.value = settings.only;
  aoeFractionInput.value = String(settings.aoeFraction);
  iterationsInput.value = String(settings.iterations);
  finalIterationsInput.value = String(settings.finalIterations);
  seedInput.value = String(settings.seed);
  jobsInput.value = settings.jobs === null ? "" : String(settings.jobs);

  invoke<number>("get_default_jobs").then((defaultJobs) => {
    jobsInput.placeholder = `auto (${defaultJobs})`;
  });

  function persist(): void {
    saveSettings(readRaw());
  }
  for (const input of [onlySelect, aoeFractionInput, iterationsInput, finalIterationsInput, seedInput, jobsInput]) {
    input.addEventListener("change", persist);
  }

  function readRaw(): RunSettings {
    return {
      only: onlySelect.value as OnlySetting,
      aoeFraction: Number(aoeFractionInput.value),
      iterations: Number(iterationsInput.value),
      finalIterations: Number(finalIterationsInput.value),
      seed: Number(seedInput.value),
      jobs: jobsInput.value === "" ? null : Number(jobsInput.value),
    };
  }

  return {
    read(): RunSettings | null {
      const raw = readRaw();
      if (!(raw.aoeFraction >= 0 && raw.aoeFraction <= 1)) {
        errorEl.textContent = "AoE fraction must be between 0 and 1.";
        errorEl.classList.remove("hidden");
        return null;
      }
      errorEl.classList.add("hidden");
      return raw;
    },
  };
}
