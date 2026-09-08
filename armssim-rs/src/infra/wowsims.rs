//! The [`Simulator`] implementation: drives the bundled `wowsimcli` engine, one
//! subprocess per gear set, across a bounded worker pool sized to the machine.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;

use anyhow::Context;
use serde_json::Value;

use crate::app::character::Character;
use crate::app::simulator::{Job, Simulator};
use crate::domain::gear::{GearSet, Scenario};
use crate::error::ArmssimError;

use super::engine::EnginePaths;
use super::request::RequestBuilder;

/// Unique-suffix source for per-sim temp files.
static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct WowSimsBackend {
    builder: RequestBuilder,
    cli: PathBuf,
    concurrency: usize,
}

impl WowSimsBackend {
    pub fn new(
        character: &Character,
        engine: &EnginePaths,
        concurrency: usize,
    ) -> anyhow::Result<WowSimsBackend> {
        Ok(WowSimsBackend {
            builder: RequestBuilder::new(character)?,
            cli: engine.cli.clone(),
            concurrency: concurrency.max(1),
        })
    }

    /// Run one sim by writing the request to a temp file and invoking the CLI.
    fn run_one(
        &self,
        set: &GearSet,
        scenario: Scenario,
        iterations: u32,
        seed: i64,
    ) -> anyhow::Result<f64> {
        let req = self.builder.build(set, scenario, iterations, seed);
        let bytes = serde_json::to_vec(&req).context("encode sim request")?;

        let suffix = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("armssim-{}-{}.json", std::process::id(), suffix));
        std::fs::write(&path, &bytes)
            .with_context(|| format!("write sim request {}", path.display()))?;

        let mut command = Command::new(&self.cli);
        command.arg("sim").arg("--infile").arg(&path);
        // Without this, a GUI-subsystem parent (no console of its own) makes
        // Windows pop a new console window for every console-subsystem child
        // it spawns -- one per sim, hundreds per search.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        let result = command.output();
        let _ = std::fs::remove_file(&path);

        let output = result.with_context(|| format!("run {}", self.cli.display()))?;
        if !output.status.success() {
            return Err(ArmssimError::Sim(format!(
                "wowsimcli exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ))
            .into());
        }

        let res: Value = serde_json::from_slice(&output.stdout).context("parse sim result")?;
        if let Some(msg) = res
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
        {
            if !msg.is_empty() {
                return Err(ArmssimError::Sim(msg.to_string()).into());
            }
        }

        res.pointer("/raidMetrics/parties/0/players/0/dps/avg")
            .and_then(Value::as_f64)
            .ok_or_else(|| ArmssimError::Sim("sim result missing player dps".into()).into())
    }
}

impl Simulator for WowSimsBackend {
    fn run(&self, jobs: &[Job<'_>], iterations: u32, seed: i64) -> anyhow::Result<Vec<f64>> {
        let n = jobs.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        let next = AtomicUsize::new(0);
        let (tx, rx) = mpsc::channel::<(usize, anyhow::Result<f64>)>();
        let workers = self.concurrency.min(n);

        std::thread::scope(|scope| {
            for _ in 0..workers {
                let tx = tx.clone();
                let next = &next;
                scope.spawn(move || loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= n {
                        break;
                    }
                    let (set, scenario) = jobs[i];
                    let dps = self.run_one(set, scenario, iterations, seed);
                    if tx.send((i, dps)).is_err() {
                        break;
                    }
                });
            }
            drop(tx); // workers hold the only remaining senders
        });

        let mut out = vec![0.0_f64; n];
        let mut first_err: Option<anyhow::Error> = None;
        for (i, dps) in rx.iter() {
            match dps {
                Ok(d) => out[i] = d,
                Err(e) if first_err.is_none() => first_err = Some(e),
                Err(_) => {}
            }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(out),
        }
    }
}
