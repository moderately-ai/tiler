//! Host-runtime and memory measurement protocol.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tiler_compiler::physical_provider::{
    InstalledPhysicalProviders, PhysicalImplementationProvider,
};
use tiler_compiler::session::NumericalContract;
use tiler_compiler::target::TargetProfile;
use tiler_ir::semantic::SemanticProgram;

use crate::program::{Compiled, compile_governed_only, compile_installed, compile_request};
use crate::providers::{Answer, as_dyn, flock, shared_tally};

/// Complete custody for one child `/usr/bin/time -l` invocation.
#[derive(Clone, Debug)]
pub struct RssCustody {
    /// Exact command and arguments, executable first.
    pub command: Vec<String>,
    /// Whether the child process reported success.
    pub child_exit_success: bool,
    /// Numeric child exit code, absent only when terminated by a signal.
    pub child_exit_code: Option<i32>,
    /// Complete stderr emitted by macOS `time -l` and the child.
    pub time_stderr: String,
    /// Peak RSS parsed from [`Self::time_stderr`].
    pub parsed_peak_rss_bytes: Option<u64>,
}

/// Timing and memory statistics for one workload.
#[derive(Clone, Debug)]
pub struct Sample {
    /// Workload name.
    pub name: String,
    /// Program family measured by this workload.
    pub program_kind: &'static str,
    /// Whether this series compiles through the public multi-target request path.
    pub request_wide: bool,
    /// Target profiles in the one public request.
    pub targets: usize,
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
    /// Every timed duration in execution order.
    pub timed_durations: Vec<Duration>,
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
    /// Complete child timing custody, when RSS was collected.
    pub rss: Option<RssCustody>,
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
    let stats = summarize(&durations);

    let rss = if collect_rss {
        self_exe.map(|exe| child_peak_rss(exe, name, extra, kind))
    } else {
        None
    };
    let peak_rss_bytes = rss
        .as_ref()
        .and_then(|custody| custody.parsed_peak_rss_bytes);

    Sample {
        name: name.to_owned(),
        program_kind: "five-op",
        request_wide: false,
        targets: 1,
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
        timed_durations: durations,
        min: stats.min,
        median: stats.median,
        p90: stats.p90,
        max: stats.max,
        mean: stats.mean,
        peak_rss_bytes,
        rss,
    }
}

/// Measures a complete multi-target public request with the same warm-up,
/// repetition, and child-RSS controls as [`measure_population`].
#[allow(
    clippy::too_many_arguments,
    reason = "the retained harness states every measurement control at its call site"
)]
pub fn measure_request_population(
    name: &str,
    program_kind: &'static str,
    program: &SemanticProgram,
    contracts: &[NumericalContract],
    profiles: &[TargetProfile],
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
    let compiled = compile_request_once(program, contracts, profiles, extra, &census_refs);
    let observed = census_tally.borrow().clone();

    let time_tally = shared_tally();
    let time_providers = flock(&format!("{name}-time"), extra, answer, &time_tally);
    let time_refs = as_dyn(&time_providers);
    for _ in 0..warmup {
        let _ = compile_request_once(program, contracts, profiles, extra, &time_refs);
    }
    let mut durations = Vec::with_capacity(repeats as usize);
    for _ in 0..repeats {
        let start = Instant::now();
        let _ = compile_request_once(program, contracts, profiles, extra, &time_refs);
        durations.push(start.elapsed());
    }
    let stats = summarize(&durations);
    let rss = if collect_rss {
        self_exe
            .map(|exe| child_request_peak_rss(exe, name, profiles.len(), extra, kind, program_kind))
    } else {
        None
    };
    let peak_rss_bytes = rss
        .as_ref()
        .and_then(|custody| custody.parsed_peak_rss_bytes);

    Sample {
        name: name.to_owned(),
        program_kind,
        request_wide: true,
        targets: profiles.len(),
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
        timed_durations: durations,
        min: stats.min,
        median: stats.median,
        p90: stats.p90,
        max: stats.max,
        mean: stats.mean,
        peak_rss_bytes,
        rss,
    }
}

fn compile_request_once(
    program: &SemanticProgram,
    contracts: &[NumericalContract],
    profiles: &[TargetProfile],
    extra: usize,
    refs: &[&dyn PhysicalImplementationProvider],
) -> Compiled {
    let environment = if extra == 0 {
        InstalledPhysicalProviders::governed()
    } else {
        InstalledPhysicalProviders::installed(refs.iter().copied())
            .expect("the request measurement providers install")
    };
    compile_request(
        program,
        contracts.iter().copied(),
        profiles.iter().cloned(),
        &environment,
    )
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Stats {
    min: Duration,
    median: Duration,
    p90: Duration,
    max: Duration,
    mean: Duration,
}

fn summarize(durations: &[Duration]) -> Stats {
    let mut sorted = durations.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    let min = sorted[0];
    let max = sorted[n - 1];
    let median = sorted[n / 2];
    let p90 = sorted[n
        .saturating_mul(9)
        .saturating_sub(1)
        .saturating_div(10)
        .min(n - 1)];
    let total: Duration = sorted.iter().copied().sum();
    let mean = total / u32::try_from(n).unwrap_or(1);
    Stats {
        min,
        median,
        p90,
        max,
        mean,
    }
}

/// Recomputes one in-memory row before rendering its retained form.
pub fn verify_sample_custody(sample: &Sample, expect_rss: bool) -> Result<(), String> {
    if sample.timed_durations.len() != usize::try_from(sample.repeats).unwrap_or(usize::MAX) {
        return Err(format!(
            "{} retained {} timing values for repeats={}",
            sample.name,
            sample.timed_durations.len(),
            sample.repeats
        ));
    }
    if sample.timed_durations.is_empty() {
        return Err(format!("{} retained no timing values", sample.name));
    }
    let recomputed = summarize(&sample.timed_durations);
    let published = Stats {
        min: sample.min,
        median: sample.median,
        p90: sample.p90,
        max: sample.max,
        mean: sample.mean,
    };
    if recomputed != published {
        return Err(format!(
            "{} timing summary does not match retained values: expected={recomputed:?} observed={published:?}",
            sample.name
        ));
    }
    match (&sample.rss, expect_rss) {
        (Some(rss), true) => {
            if !rss.child_exit_success || rss.child_exit_code != Some(0) {
                return Err(format!(
                    "{} RSS child failed: success={} code={:?}",
                    sample.name, rss.child_exit_success, rss.child_exit_code
                ));
            }
            let reparsed = parse_time_l_rss_exact(&rss.time_stderr)
                .map_err(|error| format!("{} {error}", sample.name))?;
            if rss.parsed_peak_rss_bytes != Some(reparsed)
                || sample.peak_rss_bytes != Some(reparsed)
            {
                return Err(format!(
                    "{} RSS summary does not match retained stderr: parsed={reparsed} custody={:?} summary={:?}",
                    sample.name, rss.parsed_peak_rss_bytes, sample.peak_rss_bytes
                ));
            }
        }
        (None, false) => {}
        (Some(_), false) => {
            return Err(format!("{} unexpectedly retained RSS custody", sample.name));
        }
        (None, true) => return Err(format!("{} has no retained RSS custody", sample.name)),
    }
    Ok(())
}

fn kind_name(answer: Answer) -> &'static str {
    match answer {
        Answer::Empty => "empty",
        Answer::Decline => "decline",
        Answer::Specialize { .. } => "propose",
        Answer::Infeasible { .. } => "infeasible",
    }
}

fn child_peak_rss(exe: &str, name: &str, extra: usize, kind: &str) -> RssCustody {
    capture_child_rss(vec![
        "/usr/bin/time".to_owned(),
        "-l".to_owned(),
        exe.to_owned(),
        "child-measure".to_owned(),
        name.to_owned(),
        extra.to_string(),
        kind.to_owned(),
    ])
}

fn child_request_peak_rss(
    exe: &str,
    name: &str,
    targets: usize,
    extra: usize,
    kind: &str,
    program_kind: &str,
) -> RssCustody {
    capture_child_rss(vec![
        "/usr/bin/time".to_owned(),
        "-l".to_owned(),
        exe.to_owned(),
        "child-request-measure".to_owned(),
        name.to_owned(),
        targets.to_string(),
        extra.to_string(),
        kind.to_owned(),
        program_kind.to_owned(),
    ])
}

fn capture_child_rss(command: Vec<String>) -> RssCustody {
    let output = Command::new(&command[0])
        .args(&command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("the RSS child command launches");
    let time_stderr = String::from_utf8(output.stderr).expect("time -l stderr is UTF-8");
    let parsed_peak_rss_bytes = parse_time_l_rss_exact(&time_stderr).ok();
    RssCustody {
        command,
        child_exit_success: output.status.success(),
        child_exit_code: output.status.code(),
        time_stderr,
        parsed_peak_rss_bytes,
    }
}

/// Parses exactly one macOS `/usr/bin/time -l` maximum-RSS line.
pub fn parse_time_l_rss_exact(stderr: &str) -> Result<u64, String> {
    let mut values = Vec::new();
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_suffix("maximum resident set size") {
            let value = rest
                .trim()
                .parse()
                .map_err(|_| "retained time stderr has an invalid maximum RSS".to_owned())?;
            values.push(value);
        }
    }
    match values.as_slice() {
        [value] => Ok(*value),
        [] => Err("retained time stderr has no maximum RSS".to_owned()),
        _ => Err(format!(
            "retained time stderr has {} maximum RSS lines",
            values.len()
        )),
    }
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

/// One warmed multi-target compile used as the `/usr/bin/time -l` subject.
pub fn child_request_measure(
    program: &SemanticProgram,
    contracts: &[NumericalContract],
    profiles: &[TargetProfile],
    extra: usize,
    kind: &str,
) {
    let answer = match kind {
        "empty" => Answer::Empty,
        "decline" => Answer::Decline,
        "propose" => Answer::Specialize { threads: 32 },
        "infeasible" => Answer::Infeasible { threads: 512 },
        other => panic!("unknown child kind {other}"),
    };
    let tally = shared_tally();
    let providers = flock("child-request", extra, answer, &tally);
    let refs = as_dyn(&providers);
    for _ in 0..2 {
        let _ = compile_request_once(program, contracts, profiles, extra, &refs);
    }
    let _ = compile_request_once(program, contracts, profiles, extra, &refs);
}

/// Formats a duration as microseconds for JSON.
#[must_use]
pub fn micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}

/// Formats a duration as exact nanoseconds for custody JSON.
#[must_use]
pub fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_preserves_execution_order() {
        let durations = [
            Duration::from_nanos(5),
            Duration::from_nanos(1),
            Duration::from_nanos(3),
        ];
        let before = durations;
        let stats = summarize(&durations);
        assert_eq!(durations, before, "summary must not reorder custody values");
        assert_eq!(stats.min, Duration::from_nanos(1));
        assert_eq!(stats.median, Duration::from_nanos(3));
        assert_eq!(stats.p90, Duration::from_nanos(5));
        assert_eq!(stats.max, Duration::from_nanos(5));
        assert_eq!(stats.mean, Duration::from_nanos(3));
    }

    #[test]
    fn duplicate_maximum_rss_lines_are_ambiguous() {
        assert_eq!(
            parse_time_l_rss_exact(
                "42  maximum resident set size\n43  maximum resident set size\n"
            ),
            Err("retained time stderr has 2 maximum RSS lines".to_owned())
        );
    }
}
