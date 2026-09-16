//! Coordinate ascent over the search space, plus baseline evaluation. Holds all
//! slots fixed, finds the best item for one group, repeats until nothing
//! improves.

use crate::domain::gear::{GearSet, Scenario};
use crate::domain::item::ItemCatalog;
use crate::domain::objective::Objective;
use crate::domain::plan::Plan;
use crate::domain::refine::{self, RefineCatalog};

use super::simulator::{Job, Simulator};

/// Evaluate an objective over many gear sets, combining the scenarios it needs
/// into a single score. All required sims are issued as one batch so the backend
/// can parallelize across them.
pub fn score_sets(
    sim: &dyn Simulator,
    sets: &[GearSet],
    objective: Objective,
    iterations: u32,
    seed: i64,
) -> anyhow::Result<Vec<f64>> {
    let mut jobs: Vec<Job> = Vec::with_capacity(sets.len() * 2);
    for set in sets {
        if objective.need_st() {
            jobs.push((set, Scenario::SingleTarget));
        }
        if objective.need_aoe() {
            jobs.push((set, Scenario::Aoe));
        }
    }

    let dps = sim.run(&jobs, iterations, seed)?;

    let mut scores = Vec::with_capacity(sets.len());
    let mut k = 0;
    for _ in sets {
        let st = if objective.need_st() {
            let v = dps[k];
            k += 1;
            v
        } else {
            0.0
        };
        let aoe = if objective.need_aoe() {
            let v = dps[k];
            k += 1;
            v
        } else {
            0.0
        };
        scores.push(objective.score(st, aoe));
    }
    Ok(scores)
}

/// Outcome of a coordinate-ascent run.
pub struct Ascent {
    pub best: GearSet,
    /// Number of gear-set evaluations performed (for progress reporting).
    pub evaluations: usize,
}

/// Run coordinate ascent maximizing `objective`.
pub fn ascend(
    sim: &dyn Simulator,
    plan: &Plan,
    base: &GearSet,
    objective: Objective,
    iterations: u32,
    seed: i64,
    on_progress: impl Fn(usize),
) -> anyhow::Result<Ascent> {
    let mut current = base.clone();
    let mut evaluations = 0;

    // Nothing to decide (e.g. --keep-items): the base is the answer, and
    // scoring it here would only spend a sim.
    if plan.groups.is_empty() {
        return Ok(Ascent {
            best: current,
            evaluations,
        });
    }

    let mut best_score = score_sets(
        sim,
        std::slice::from_ref(&current),
        objective,
        iterations,
        seed,
    )?[0];
    evaluations += 1;
    on_progress(evaluations);

    loop {
        let mut improved = false;
        for group in &plan.groups {
            let candidates = group.expand(&current);
            if candidates.is_empty() {
                continue;
            }
            let scores = score_sets(sim, &candidates, objective, iterations, seed)?;
            evaluations += candidates.len();
            on_progress(evaluations);

            let mut best_idx: Option<usize> = None;
            let mut local_best = best_score;
            for (i, &s) in scores.iter().enumerate() {
                if s > local_best {
                    local_best = s;
                    best_idx = Some(i);
                }
            }
            if let Some(i) = best_idx {
                current = candidates[i].clone();
                best_score = local_best;
                improved = true;
            }
        }
        if !improved {
            break;
        }
    }

    Ok(Ascent {
        best: current,
        evaluations,
    })
}

/// Simulate one gear set in both scenarios (used for the precise re-sim of the
/// winning set and the baseline).
pub fn eval(
    sim: &dyn Simulator,
    set: &GearSet,
    iterations: u32,
    seed: i64,
) -> anyhow::Result<(f64, f64)> {
    let jobs = [(set, Scenario::SingleTarget), (set, Scenario::Aoe)];
    let dps = sim.run(&jobs, iterations, seed)?;
    Ok((dps[0], dps[1]))
}

/// What the gem/enchant pass needs to build its shortlists. Absent means "items
/// only".
pub struct Refinement<'a> {
    pub items: &'a dyn ItemCatalog,
    pub gems: &'a dyn RefineCatalog,
    pub professions: &'a [String],
}

/// The result of a full search for one objective.
pub struct Outcome {
    /// The set after the item search, before any gems or enchants moved. The
    /// gem/enchant report is diffed against this.
    pub items_only: GearSet,
    /// The final recommendation.
    pub best: GearSet,
    pub search_sims: usize,
    pub refine_sims: usize,
}

/// Search for one objective: items first, then — if `refine` is given — gems and
/// enchants for the items that won, then a meta-gem repair if the greedy socket
/// pass left the meta dark.
///
/// `on_progress` is called with a short phase label ("searching", "gems &
/// enchants") and the running sim count for that phase.
/// Matches the shape of `ascend` it wraps; splitting the sim budget out into a
/// struct would churn every call site for one call.
#[allow(clippy::too_many_arguments)]
pub fn search(
    sim: &dyn Simulator,
    plan: &Plan,
    base: &GearSet,
    objective: Objective,
    iterations: u32,
    seed: i64,
    refine_with: Option<Refinement<'_>>,
    on_progress: impl Fn(&str, usize),
) -> anyhow::Result<Outcome> {
    let ascent = ascend(sim, plan, base, objective, iterations, seed, |n| {
        on_progress("searching", n)
    })?;

    let items_only = ascent.best.clone();
    let mut best = ascent.best;
    let mut refine_sims = 0;

    if let Some(r) = refine_with {
        let groups = refine::plan(&best, r.items, r.gems, r.professions);
        if !groups.is_empty() {
            let refine_plan = Plan {
                groups,
                skipped: Vec::new(),
            };
            let pass = ascend(sim, &refine_plan, &best, objective, iterations, seed, |n| {
                on_progress("gems & enchants", n)
            })?;
            best = pass.best;
            refine_sims = pass.evaluations;
        }

        // Coordinate ascent cannot light a meta gem one socket at a time, so
        // the cheapest repair is proposed here and judged by the engine.
        if let Some(repaired) = refine::light_meta(&best, r.items, r.gems, r.professions) {
            let scores = score_sets(
                sim,
                &[best.clone(), repaired.clone()],
                objective,
                iterations,
                seed,
            )?;
            refine_sims += 2;
            if scores[1] > scores[0] {
                best = repaired;
            }
        }
    }

    Ok(Outcome {
        items_only,
        best,
        search_sims: ascent.evaluations,
        refine_sims,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::gear::{ItemSlot, ItemSpec};
    use crate::domain::gems;
    use crate::domain::item::ItemInfo;
    use crate::domain::refine::{EnchantInfo, GemInfo};

    /// DPS is the sum of every socketed gem id, so a higher id always wins.
    struct GemSum;
    impl Simulator for GemSum {
        fn run(&self, jobs: &[Job<'_>], _: u32, _: i64) -> anyhow::Result<Vec<f64>> {
            Ok(jobs
                .iter()
                .map(|(set, _)| {
                    (0..crate::domain::gear::NUM_SLOTS)
                        .filter_map(|i| set.get(i))
                        .flat_map(|s| s.gems.iter())
                        .map(|g| f64::from(*g))
                        .sum()
                })
                .collect())
        }
    }

    struct Items;
    impl ItemCatalog for Items {
        fn lookup(&self, id: i32) -> Option<ItemInfo> {
            (id == 100).then(|| ItemInfo {
                name: "Legs".into(),
                slots: vec![ItemSlot::Legs],
                sockets: vec![gems::RED],
                item_type: 9,
            })
        }
    }

    struct Gems(Vec<GemInfo>);
    impl RefineCatalog for Gems {
        fn gem(&self, id: i32) -> Option<&GemInfo> {
            self.0.iter().find(|g| g.id == id)
        }
        fn all_gems(&self) -> &[GemInfo] {
            &self.0
        }
        fn enchant(&self, _: i32, _: i32) -> Option<&EnchantInfo> {
            None
        }
    }

    fn red(id: i32, strength: f64) -> GemInfo {
        let mut stats = vec![0.0; 42];
        stats[0] = strength;
        GemInfo {
            id,
            name: format!("gem{id}"),
            color: gems::RED,
            stats,
            unique: false,
            profession: String::new(),
        }
    }

    #[test]
    fn keeping_items_refines_the_equipped_set_without_searching() {
        let mut equipped = GearSet::new();
        let legs = ItemSpec {
            id: 100,
            enchant: 0,
            gems: vec![1],
            random_suffix: 0,
        };
        equipped.set(ItemSlot::Legs.index(), Some(legs));
        let gem_db = Gems(vec![red(1, 4.0), red(2, 8.0)]);
        let no_items = Plan {
            groups: Vec::new(),
            skipped: Vec::new(),
        };

        let outcome = search(
            &GemSum,
            &no_items,
            &equipped,
            Objective::SingleTarget,
            1,
            1,
            Some(Refinement {
                items: &Items,
                gems: &gem_db,
                professions: &[],
            }),
            |_, _| {},
        )
        .unwrap();

        assert_eq!(
            outcome.search_sims, 0,
            "no item search when nothing to swap"
        );
        assert_eq!(
            outcome.items_only.get(ItemSlot::Legs.index()),
            equipped.get(ItemSlot::Legs.index())
        );
        let best = outcome.best.get(ItemSlot::Legs.index()).unwrap();
        assert_eq!(best.id, 100, "the equipped item is kept");
        assert_eq!(best.gems, vec![2], "the better gem is suggested");
    }
}
