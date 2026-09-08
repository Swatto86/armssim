//! Tauri commands the frontend invokes.

use tauri::Emitter;

use armssim::app::character::Character;
use armssim::app::optimize;
use armssim::domain::objective::Objective;
use armssim::domain::plan::Plan;
use armssim::domain::refine;
use armssim::domain::report;
use armssim::infra::itemdb::DbCatalog;
use armssim::infra::wowsims::WowSimsBackend;

use crate::engine_resources;
use crate::progress::{
    ObjectiveResult, OnlySetting, ProgressEvent, RefineChangeDto, RunResult, RunSettings,
    SlotChangeDto,
};

#[tauri::command]
pub fn get_default_jobs() -> usize {
    default_jobs()
}

#[tauri::command]
pub async fn run_optimizer(
    app: tauri::AppHandle,
    character_json: String,
    settings: RunSettings,
) -> Result<RunResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_optimizer_blocking(&app, &character_json, &settings)
    })
    .await
    .map_err(|e| format!("optimizer task panicked: {e}"))?
    .map_err(|e| format!("{e:#}"))
}

fn run_optimizer_blocking(
    app: &tauri::AppHandle,
    character_json: &str,
    settings: &RunSettings,
) -> anyhow::Result<RunResult> {
    let character = Character::parse(character_json.as_bytes())?;
    let engine = engine_resources::resolve(app).map_err(anyhow::Error::msg)?;
    let catalog = DbCatalog::load(&engine.db_json)?;
    let jobs = settings.jobs.unwrap_or_else(default_jobs);
    let backend = WowSimsBackend::new(&character, &engine, jobs, catalog.gem_colors())?;

    let equipped = character.equipped_set();
    let plan = Plan::build(&equipped, &character.bag_items, &catalog);
    let candidate_pool = character.bag_items.len();
    let skipped = plan.skipped.len();

    let (base_st, base_aoe) = optimize::eval(
        &backend,
        &equipped,
        settings.final_iterations,
        settings.seed,
    )?;

    let objectives = select_objectives(settings.only, settings.aoe_fraction);
    let mut results = Vec::with_capacity(objectives.len());
    let professions: Vec<String> = character
        .professions
        .iter()
        .map(|p| p.name.clone())
        .collect();

    for objective in objectives {
        let name = objective.name();
        let app_handle = app.clone();
        let objective_name = name.clone();
        let progress = move |phase: &str, n: usize| {
            let _ = app_handle.emit(
                "optimize-progress",
                ProgressEvent {
                    objective: format!("{objective_name} — {phase}"),
                    sims: n,
                },
            );
        };

        let refine_with = (!settings.no_refine).then_some(optimize::Refinement {
            items: &catalog,
            gems: &catalog,
            professions: &professions,
        });
        let outcome = optimize::search(
            &backend,
            &plan,
            &equipped,
            objective,
            settings.iterations,
            settings.seed,
            refine_with,
            progress,
        )?;
        let (st, aoe) = optimize::eval(
            &backend,
            &outcome.best,
            settings.final_iterations,
            settings.seed,
        )?;
        let changes = report::diff(&equipped, &outcome.best, &catalog)
            .into_iter()
            .map(|c| SlotChangeDto {
                label: c.label,
                from: c.from,
                to: c.to,
            })
            .collect();
        let refinements =
            report::refine_diff(&outcome.items_only, &outcome.best, &catalog, &catalog)
                .into_iter()
                .map(|c| RefineChangeDto {
                    label: c.label,
                    what: c.what,
                    from: c.from,
                    to: c.to,
                })
                .collect();
        let missing = refine::gaps(&outcome.best, &catalog, &catalog)
            .into_iter()
            .map(|g| format!("{}: {}", g.label, g.what))
            .collect();

        results.push(ObjectiveResult {
            name,
            st_dps: st,
            aoe_dps: aoe,
            st_delta: st - base_st,
            st_delta_pct: 100.0 * (st - base_st) / base_st,
            aoe_delta: aoe - base_aoe,
            aoe_delta_pct: 100.0 * (aoe - base_aoe) / base_aoe,
            search_sims: outcome.search_sims,
            refine_sims: outcome.refine_sims,
            changes,
            refinements,
            missing,
        });
    }

    Ok(RunResult {
        character_name: character.name,
        character_race: character.race,
        candidate_pool,
        skipped,
        base_st,
        base_aoe,
        objectives: results,
    })
}

/// Mirrors `main.rs::select_objectives` — kept as a small, presentation-layer
/// duplicate rather than sharing `cli::Only` (clap-derived, CLI-only).
fn select_objectives(only: OnlySetting, aoe_fraction: f64) -> Vec<Objective> {
    let st = Objective::SingleTarget;
    let blend = Objective::Blend { aoe_fraction };
    let aoe = Objective::Aoe;
    match only {
        OnlySetting::All => vec![st, blend, aoe],
        OnlySetting::St => vec![st],
        OnlySetting::Blend => vec![blend],
        OnlySetting::Aoe => vec![aoe],
    }
}

fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8)
}
