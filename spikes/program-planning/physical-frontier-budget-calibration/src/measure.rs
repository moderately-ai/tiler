//! Host-runtime and memory measurement protocol.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tiler_compiler::physical_provider::PhysicalImplementationProvider;
use tiler_compiler::target::TargetProfile;
use tiler_ir::semantic::SemanticProgram;

use crate::program::{Compiled, compile_governed_only, compile_installed};
use crate::providers::{Answer, as_dyn, flock, shared_tally};

/// Timing and memory statistics for one workload.
#[derive(Clone, Debug)]
pub struct Sample {
    /// Workload name.
    pub name: String,
    /// Installed extra providers (governed is always present).
    pub extra_providers: usize,
    /// Provider answer kind.
    pub kind: &'static str,
    /// Observed provider invocations.
    pub invocations: u64,
    /// Observed proposals.
    pub proposals: u64,
    /// Observed declines.
    pub declines: u64,
    /// Distinct region subjects.
    pub subjects: u64,
    /// Subjects that had a baseline.
    pub baseline_subjects: u64,
    /// Retained complete-plan alternatives.
    pub alternatives: usize,
    /// Offered physical providers including governed.
    pub offered: usize,
    /// Distinct selected physical providers.
    pub selected_providers: usize,
    /// Rendered explain bytes.
    pub explain_bytes: usize,
    /// Public failure class, when the compile refused.
    pub failure: Option<String>,
    /// Warm-up compiles discarded before timing.
    pub warmup: u32,
    /// Timed repetitions.
    pub repeats: u32,
    /// Minimum compile duration.
    pub min: Duration,
    /// Median compile duration.
    pub median: Duration,
    /// 90th-percentile compile duration.
    pub p90: Duration,
    /// Maximum compile duration.
    pub max: Duration,
    /// Mean compile duration.
    pub mean: Duration,
    /// Child-process peak RSS in bytes, when collected.
    pub peak_rss_bytes: Option<u64>,
}

/// Measures one installed population on `program` / `profile`.
pub fn measure_population(
    name: &str,
    program: &SemanticProgram,
    profile: &TargetProfile,
    extra: usize,
    answer: Answer,
    warmup: u32,
    repeats: u32,
    collect_rss: bool,
    self_exe: Option<&str>,
) -> Sample {
    let kind = kind_name(answer);
    let census_tally = shared_tally();
    let census_providers = flock(name, extra, answer, &census_tally);
    let census_refs = as_dyn(&census_providers);
    let compiled = compile_once(program, profile, extra, &census_refs);
    let observed = census_tally.borrow().clone();

    let time_tally = shared_tally();
    let time_providers = flock(&format!("{name}-time"), extra, answer, &time_tally);
    let time_refs = as_dyn(&time_providers);
    for _ in 0..warmup {
        let _ = compile_once(program, profile, extra, &time_refs);
    }
    let mut durations = Vec::with_capacity(repeats as usize);
    for _ in 0..repeats {
        let start = Instant::now();
        let _ = compile_once(program, profile, extra, &time_refs);
        durations.push(start.elapsed());
    }
    let stats = summarize(&mut durations);

    let peak_rss_bytes = if collect_rss {
        self_exe.and_then(|exe| child_peak_rss(exe, name, extra, kind))
    } else {
        None
    };

    Sample {
        name: name.to_owned(),
        extra_providers: extra,
        kind,
        invocations: observed.invocations,
        proposals: observed.proposals,
        declines: observed.declines,
        subjects: observed.distinct_subjects(),
        baseline_subjects: observed.baseline_subjects,
        alternatives: compiled.alternatives,
        offered: compiled.offered,
        selected_providers: compiled.selected_providers,
        explain_bytes: compiled.explain_bytes,
        failure: compiled.failure.map(|class| format!("{class:?}")),
        warmup,
        repeats,
        min: stats.min,
        median: stats.median,
        p90: stats.p90,
        max: stats.max,
        mean: stats.mean,
        peak_rss_bytes,
    }
}

fn compile_once(
    program: &SemanticProgram,
    profile: &TargetProfile,
    extra: usize,
    refs: &[&dyn PhysicalImplementationProvider],
) -> Compiled {
    if extra == 0 {
        compile_governed_only(program, profile)
    } else {
        compile_installed(program, profile, refs.iter().copied())
    }
}

struct Stats {
    min: Duration,
    median: Duration,
    p90: Duration,
    max: Duration,
    mean: Duration,
}

fn summarize(durations: &mut [Duration]) -> Stats {
    durations.sort_unstable();
    let n = durations.len();
    let min = durations[0];
    let max = durations[n - 1];
    let median = durations[n / 2];
    let p90 = durations[n
        .saturating_mul(9)
        .saturating_sub(1)
        .saturating_div(10)
        .min(n - 1)];
    let total: Duration = durations.iter().copied().sum();
    let mean = total / u32::try_from(n).unwrap_or(1);
    Stats {
        min,
        median,
        p90,
        max,
        mean,
    }
}

fn kind_name(answer: Answer) -> &'static str {
    match answer {
        Answer::Empty => "empty",
        Answer::Decline => "decline",
        Answer::Specialize { .. } => "propose",
        Answer::Infeasible { .. } => "infeasible",
    }
}

fn child_peak_rss(exe: &str, name: &str, extra: usize, kind: &str) -> Option<u64> {
    let output = Command::new("/usr/bin/time")
        .args(["-l", exe, "child-measure", name, &extra.to_string(), kind])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .ok()?;
    parse_time_l_rss(&String::from_utf8_lossy(&output.stderr))
}

/// Parses macOS `/usr/bin/time -l` maximum resident set size.
#[must_use]
pub fn parse_time_l_rss(stderr: &str) -> Option<u64> {
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_suffix("maximum resident set size") {
            return rest.trim().parse().ok();
        }
    }
    None
}

/// One child compile used only as the `/usr/bin/time -l` subject.
pub fn child_measure(program: &SemanticProgram, profile: &TargetProfile, extra: usize, kind: &str) {
    let answer = match kind {
        "empty" => Answer::Empty,
        "decline" => Answer::Decline,
        "propose" => Answer::Specialize { threads: 32 },
        "infeasible" => Answer::Infeasible { threads: 512 },
        other => panic!("unknown child kind {other}"),
    };
    let tally = shared_tally();
    let providers = flock("child", extra, answer, &tally);
    let refs = as_dyn(&providers);
    for _ in 0..2 {
        let _ = compile_once(program, profile, extra, &refs);
    }
    let _ = compile_once(program, profile, extra, &refs);
}

/// Formats a duration as microseconds for JSON.
#[must_use]
pub fn micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}
