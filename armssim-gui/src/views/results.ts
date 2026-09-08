import type { ObjectiveResult, RunResult } from "../state";

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function fmtDelta(value: number, pct: number): string {
  const sign = value >= 0 ? "+" : "";
  return `${sign}${value.toFixed(1)} (${sign}${pct.toFixed(2)}%)`;
}

function objectiveCard(objective: ObjectiveResult): HTMLElement {
  const card = el("div", "card bg-base-100 shadow");
  const body = el("div", "card-body gap-2");
  card.appendChild(body);

  const header = el("div", "flex items-center justify-between");
  header.appendChild(el("h3", "card-title text-base", objective.name));
  header.appendChild(el("span", "text-xs opacity-60", `${objective.searchSims} search sims`));
  body.appendChild(header);

  const dps = el("div", "grid grid-cols-2 gap-2 text-sm");
  const stCell = el("div");
  stCell.appendChild(el("div", "opacity-60 text-xs", "Single target"));
  stCell.appendChild(el("div", "font-mono", `${objective.stDps.toFixed(1)} DPS`));
  stCell.appendChild(el("div", "text-xs opacity-70", fmtDelta(objective.stDelta, objective.stDeltaPct)));
  dps.appendChild(stCell);

  const aoeCell = el("div");
  aoeCell.appendChild(el("div", "opacity-60 text-xs", "AoE"));
  aoeCell.appendChild(el("div", "font-mono", `${objective.aoeDps.toFixed(1)} DPS`));
  aoeCell.appendChild(el("div", "text-xs opacity-70", fmtDelta(objective.aoeDelta, objective.aoeDeltaPct)));
  dps.appendChild(aoeCell);
  body.appendChild(dps);

  if (objective.changes.length === 0) {
    body.appendChild(el("p", "text-sm italic opacity-70", "Current gear is already optimal here."));
  } else {
    const list = el("ul", "text-sm flex flex-col gap-1");
    for (const change of objective.changes) {
      const item = el("li", "flex items-center gap-2");
      item.appendChild(el("span", "badge badge-outline badge-sm", change.label));
      item.appendChild(el("span", "font-mono text-xs", `${change.from} -> ${change.to}`));
      list.appendChild(item);
    }
    body.appendChild(list);
  }

  return card;
}

export function renderResults(container: HTMLElement, result: RunResult): void {
  container.innerHTML = "";

  const summary = el("div", "text-sm opacity-70 flex flex-col gap-1");
  summary.appendChild(
    el(
      "p",
      undefined,
      `${result.characterName} — ${result.characterRace} Arms`,
    ),
  );
  summary.appendChild(
    el(
      "p",
      undefined,
      `Candidate pool: ${result.candidatePool} bag/bank items (${result.skipped} skipped as non-gear)`,
    ),
  );
  summary.appendChild(
    el(
      "p",
      undefined,
      `Current gear: ST ${result.baseSt.toFixed(1)}   AoE ${result.baseAoe.toFixed(1)} DPS`,
    ),
  );
  container.appendChild(summary);

  for (const objective of result.objectives) {
    container.appendChild(objectiveCard(objective));
  }
}

export function renderError(container: HTMLElement, message: string): void {
  container.innerHTML = "";
  const alert = el("div", "alert alert-error text-sm");
  alert.textContent = message;
  container.appendChild(alert);
}
