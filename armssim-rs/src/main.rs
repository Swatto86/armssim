mod cli;

use std::io::Write;

use anyhow::Context;
use clap::Parser;

use armssim::app::character::Character;
use armssim::app::optimize;
use armssim::app::simulator::Simulator;
use armssim::domain::gear::GearSet;
use armssim::domain::item::ItemCatalog;
use armssim::domain::objective::Objective;
use armssim::domain::plan::{self, Plan};
use armssim::domain::refine;
use armssim::domain::report;
use armssim::infra::engine;
use armssim::infra::itemdb::DbCatalog;
use armssim::infra::wowsims::WowSimsBackend;
use cli::{Args, Only};

fn main() {
    if let Err(e) = run() {
        eprintln!("armssim: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    if !(0.0..=1.0).contains(&args.aoe_fraction) {
        anyhow::bail!("--aoe-fraction must be between 0 and 1");
    }

    let data = std::fs::read(&args.character)
        .with_context(|| format!("read {}", args.character.display()))?;
    let character = Character::parse(&data)?;
    println!("Character: {} — {} Arms", character.name, character.race);

    let engine = engine::resolve(args.engine.clone())?;
    let catalog = DbCatalog::load(&engine.db_json)?;
    let jobs = args.jobs.unwrap_or_else(default_jobs);
    let backend = WowSimsBackend::new(&character, &engine, jobs, catalog.gem_colors())?;

    let equipped = character.equipped_set();
    let plan = Plan::build(&equipped, &character.bag_items, &catalog);
    println!(
        "Candidate pool: {} bag/bank items ({} skipped as non-gear or not a two-hander)",
        character.bag_items.len(),
        plan.skipped.len()
    );
    warn_if_not_two_handed(&equipped, &catalog);

    // Precise baseline (current gear) at final iterations.
    let (base_st, base_aoe) = optimize::eval(&backend, &equipped, args.final_iterations, args.seed)
        .context("baseline sim")?;
    println!("Current gear:  ST {base_st:.1}   AoE {base_aoe:.1} DPS");

    let objectives = select_objectives(args.only, args.aoe_fraction);
    let ran_blend = objectives
        .iter()
        .any(|o| matches!(o, Objective::Blend { .. }));

    let professions: Vec<String> = character
        .professions
        .iter()
        .map(|p| p.name.clone())
        .collect();

    for objective in &objectives {
        optimize_and_report(
            &backend,
            &plan,
            &catalog,
            &equipped,
            *objective,
            base_st,
            base_aoe,
            &args,
            &professions,
        )?;
    }

    if ran_blend {
        println!("\nThe BLEND set is the 'no need to pick' choice — it maximises effective DPS");
        println!("over a fight that is part single-target, part AoE (tune with --aoe-fraction).");
    }
    println!("\nNote: coordinate ascent swaps one slot at a time, so multi-piece tier-set");
    println!("bonuses (Warbringer/Onslaught 4-piece) may not be fully captured.");
    Ok(())
}

fn select_objectives(only: Only, aoe_fraction: f64) -> Vec<Objective> {
    let st = Objective::SingleTarget;
    let blend = Objective::Blend { aoe_fraction };
    let aoe = Objective::Aoe;
    match only {
        Only::All => vec![st, blend, aoe],
        Only::St => vec![st],
        Only::Blend => vec![blend],
        Only::Aoe => vec![aoe],
    }
}

#[allow(clippy::too_many_arguments)]
fn optimize_and_report(
    backend: &dyn Simulator,
    plan: &Plan,
    catalog: &DbCatalog,
    equipped: &GearSet,
    objective: Objective,
    base_st: f64,
    base_aoe: f64,
    args: &Args,
    professions: &[String],
) -> anyhow::Result<()> {
    let name = objective.name();
    let progress = |phase: &str, n: usize| {
        eprint!("\r  [{name}] {phase}... {n} sims");
        let _ = std::io::stderr().flush();
    };

    let refine_with = (!args.no_refine).then_some(optimize::Refinement {
        items: catalog,
        gems: catalog,
        professions,
    });
    let outcome = optimize::search(
        backend,
        plan,
        equipped,
        objective,
        args.iterations,
        args.seed,
        refine_with,
        progress,
    )
    .with_context(|| format!("search ({name})"))?;
    eprintln!();
    let best = outcome.best;
    let refine_sims = outcome.refine_sims;

    // Re-sim the winner precisely in BOTH scenarios to show the real trade-off.
    let (st, aoe) = optimize::eval(backend, &best, args.final_iterations, args.seed)
        .with_context(|| format!("final re-sim ({name})"))?;

    let extra = if refine_sims > 0 {
        format!(" + {refine_sims} for gems/enchants")
    } else {
        String::new()
    };
    println!(
        "
=== BEST FOR {name} ===  ({} search sims{extra})",
        outcome.search_sims
    );
    println!(
        "  ST  {st:.1}  ({:+.1}, {:+.2}%)    AoE {aoe:.1}  ({:+.1}, {:+.2}%)",
        st - base_st,
        100.0 * (st - base_st) / base_st,
        aoe - base_aoe,
        100.0 * (aoe - base_aoe) / base_aoe
    );

    let changes = report::diff(equipped, &best, catalog);
    if changes.is_empty() {
        println!("  Current gear is already optimal here.");
    } else {
        println!("  Equip:");
        for ch in changes {
            println!("    {:<9} {}  ->  {}", ch.label, ch.from, ch.to);
        }
    }

    // Gems and enchants are diffed against the post-item-search set: a slot
    // whose item changed is already reported above, and its sockets are not a
    // like-for-like comparison with what you had.
    let refinements = report::refine_diff(&outcome.items_only, &best, catalog, catalog);
    if !refinements.is_empty() {
        println!("  Gems & enchants:");
        for ch in refinements {
            println!(
                "    {:<9} {:<9} {}  ->  {}",
                ch.label, ch.what, ch.from, ch.to
            );
        }
    }

    // Anything still unsocketed or unenchanted is free DPS being ignored —
    // worth saying out loud, especially with --no-refine.
    let gaps = refine::gaps(&best, catalog, catalog);
    if !gaps.is_empty() {
        println!("  Still missing:");
        for gap in gaps {
            println!("    {:<9} {}", gap.label, gap.what);
        }
    }
    Ok(())
}

/// Arms is a two-handed spec: a one-hander (with or without an off-hand) means
/// the export was taken in the wrong gear. The search still runs — it will move
/// onto a 2H if the bags hold one — but the baseline number is not an Arms
/// number, so say so rather than quietly reporting it.
fn warn_if_not_two_handed(equipped: &GearSet, catalog: &dyn ItemCatalog) {
    match equipped.get(14) {
        Some(mh) if plan::is_two_hander(catalog, mh.id) => {}
        Some(_) => eprintln!(
            "warning: equipped main hand is not a two-hander. armssim only equips 2H              weapons, so 'Current gear' is not a valid Arms setup."
        ),
        None => eprintln!("warning: no main hand equipped in the export."),
    }
}

fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8)
}
