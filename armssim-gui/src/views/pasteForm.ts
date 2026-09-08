import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { loadCharacterJson, saveCharacterJson, type ProgressEvent, type RunResult } from "../state";
import type { SettingsForm } from "./settingsForm";

export function initPasteForm(
  settingsForm: SettingsForm,
  onResult: (result: RunResult) => void,
  onError: (message: string) => void,
): void {
  const textarea = document.querySelector<HTMLTextAreaElement>("#character-json")!;
  const runBtn = document.querySelector<HTMLButtonElement>("#run-btn")!;
  const status = document.querySelector<HTMLSpanElement>("#status")!;

  textarea.value = loadCharacterJson();
  textarea.addEventListener("change", () => saveCharacterJson(textarea.value));

  listen<ProgressEvent>("optimize-progress", (event) => {
    status.textContent = `[${event.payload.objective}] searching... ${event.payload.sims} sims`;
  });

  runBtn.addEventListener("click", async () => {
    const characterJson = textarea.value.trim();
    if (!characterJson) {
      onError("Paste a character export first.");
      return;
    }
    const settings = settingsForm.read();
    if (!settings) return;

    saveCharacterJson(textarea.value);
    runBtn.disabled = true;
    status.textContent = "starting...";
    try {
      const result = await invoke<RunResult>("run_optimizer", {
        characterJson,
        settings,
      });
      onResult(result);
    } catch (err) {
      onError(String(err));
    } finally {
      runBtn.disabled = false;
      status.textContent = "";
    }
  });
}
