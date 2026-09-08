import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";

const REPO_URL = "https://github.com/Swatto86/armssim";

export function initAboutDialog(): void {
  const button = document.querySelector<HTMLButtonElement>("#about-btn")!;
  const dialog = document.querySelector<HTMLDialogElement>("#about-dialog")!;
  const versionEl = document.querySelector<HTMLSpanElement>("#about-version")!;
  const repoLink = document.querySelector<HTMLAnchorElement>("#about-repo-link")!;

  repoLink.addEventListener("click", (e) => {
    e.preventDefault();
    void openUrl(REPO_URL);
  });

  button.addEventListener("click", async () => {
    versionEl.textContent = await getVersion();
    dialog.showModal();
  });
}
