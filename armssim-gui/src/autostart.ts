import { isEnabled, enable, disable } from "@tauri-apps/plugin-autostart";

export async function initAutostartToggle(): Promise<void> {
  const toggle = document.querySelector<HTMLInputElement>("#autostart-toggle")!;
  toggle.checked = await isEnabled();

  toggle.addEventListener("change", async () => {
    if (toggle.checked) {
      await enable();
    } else {
      await disable();
    }
    // Reflect the actual state in case the OS call failed silently.
    toggle.checked = await isEnabled();
  });
}
