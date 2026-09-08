import { initTheme } from "./theme";
import { initAutostartToggle } from "./autostart";
import { initSettingsForm } from "./views/settingsForm";
import { initPasteForm } from "./views/pasteForm";
import { renderResults, renderError } from "./views/results";
import { initAboutDialog } from "./views/about";

window.addEventListener("DOMContentLoaded", async () => {
  initTheme(document.querySelector<HTMLSelectElement>("#theme-select")!);
  initAboutDialog();
  void initAutostartToggle();

  const resultsEl = document.querySelector<HTMLElement>("#results")!;
  const settingsForm = await initSettingsForm();

  initPasteForm(
    settingsForm,
    (result) => renderResults(resultsEl, result),
    (message) => renderError(resultsEl, message),
  );
});
