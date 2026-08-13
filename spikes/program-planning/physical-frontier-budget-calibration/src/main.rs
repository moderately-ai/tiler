//! Census and host-runtime calibration of physical-frontier provider and raw-outcome budgets.

mod census;
mod measure;
mod profile;
mod program;
mod providers;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use tiler_compiler::target::TargetProfile;
use tiler_ir::semantic::SemanticProgram;

use census::{Perturb, repo_root, run_checks};
use measure::{Sample, child_measure, measure_population, micros};
use profile::{declared_workgroup_profile, governed};
use program::five_op_program;
use providers::Answer;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None | Some("record") => record(args.next(), false),
        Some("--quick") => record(None, true),
        Some("census") => census_only(Perturb::None),
        Some("perturb") => {
            if let Some(perturb) = args.next().as_deref().and_then(Perturb::parse) {
                census_only(perturb)
            } else {
                eprintln!(
                    "usage: physical-frontier-budget-calibration perturb <{}>",
                    Perturb::ALL
                        .iter()
                        .map(|perturb| perturb.name())
                        .collect::<Vec<_>>()
                        .join("|")
                );
                ExitCode::from(2)
            }
        }
        Some("child-measure") => {
            let name = args.next().unwrap_or_else(|| "child".to_owned());
            let extra = args
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            let kind = args.next().unwrap_or_else(|| "empty".to_owned());
            let program = five_op_program(4, 3);
            let profile = if kind == "infeasible" {
                declared_workgroup_profile("test.calibrate-child.v1", 64)
            } else {
                governed()
            };
            let _ = name;
            child_measure(&program, &profile, extra, &kind);
            ExitCode::SUCCESS
        }
        Some("help" | "--help") => {
            println!(
                "physical-frontier-budget-calibration [record [path]|census|perturb <name>|--quick]"
            );
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("unknown command {other}");
            ExitCode::from(2)
        }
    }
}

fn census_only(perturb: Perturb) -> ExitCode {
    let checks = run_checks(&repo_root(), perturb);
    let mut failed = 0_usize;
    for check in &checks {
        let mark = if check.passed { "PASS" } else { "FAIL" };
        if !check.passed {
            failed += 1;
        }
        println!(
            "{mark} {} expected={} observed={}",
            check.name, check.expected, check.observed
        );
    }
    println!(
        "perturb={} checks={} failed={}",
        perturb.name(),
        checks.len(),
        failed
    );
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn record(path: Option<String>, quick: bool) -> ExitCode {
    let repo = repo_root();
    let host = host_record(&repo);
    println!("host {}", host.replace('\n', " | "));

    let checks = run_checks(&repo, Perturb::None);
    let failed: Vec<_> = checks.iter().filter(|check| !check.passed).collect();
    if !failed.is_empty() {
        for check in &failed {
            eprintln!(
                "FAIL {} expected={} observed={}",
                check.name, check.expected, check.observed
            );
        }
        return ExitCode::from(1);
    }
    println!("census {} checks passed", checks.len());

    let program = five_op_program(4, 3);
    let governed_profile = governed();
    let infeasible_profile = declared_workgroup_profile("test.calibrate-sweep.v1", 64);
    let (warmup, repeats, collect_rss) = if quick { (2, 5, false) } else { (8, 50, true) };
    let exe = env::current_exe()
        .ok()
        .map(|path| path.to_string_lossy().into_owned());

    let mut samples = Vec::new();
    samples.push(measure_named(
        "governed-only",
        &program,
        &governed_profile,
        0,
        Answer::Empty,
        warmup,
        repeats,
        collect_rss,
        exe.as_deref(),
    ));
    samples.push(measure_named(
        "external-vertical",
        &program,
        &governed_profile,
        1,
        Answer::Specialize { threads: 32 },
        warmup,
        repeats,
        collect_rss,
        exe.as_deref(),
    ));
    samples.push(measure_named(
        "two-additive",
        &program,
        &governed_profile,
        2,
        Answer::Specialize { threads: 32 },
        warmup,
        repeats,
        collect_rss,
        exe.as_deref(),
    ));

    let empty_counts = if quick {
        &[0, 1, 8][..]
    } else {
        &[0, 1, 2, 4, 8, 16, 32, 64]
    };
    let decline_counts = if quick {
        &[1, 8][..]
    } else {
        &[1, 2, 4, 8, 16, 32, 64, 128]
    };
    let propose_counts = if quick {
        &[1, 2, 4][..]
    } else {
        &[1, 2, 3, 4, 8, 12, 16]
    };
    let infeasible_counts = if quick {
        &[1, 4][..]
    } else {
        &[1, 2, 4, 8, 16, 32]
    };

    for &count in empty_counts {
        if count == 0 {
            continue;
        }
        samples.push(measure_named(
            "sweep-empty",
            &program,
            &governed_profile,
            count,
            Answer::Empty,
            warmup,
            repeats,
            collect_rss,
            exe.as_deref(),
        ));
    }
    for &count in decline_counts {
        samples.push(measure_named(
            "sweep-decline",
            &program,
            &governed_profile,
            count,
            Answer::Decline,
            warmup,
            repeats,
            collect_rss,
            exe.as_deref(),
        ));
    }
    for &count in propose_counts {
        samples.push(measure_named(
            "sweep-propose",
            &program,
            &governed_profile,
            count,
            Answer::Specialize { threads: 32 },
            warmup,
            repeats,
            collect_rss,
            exe.as_deref(),
        ));
    }
    for &count in infeasible_counts {
        samples.push(measure_named(
            "sweep-infeasible",
            &program,
            &infeasible_profile,
            count,
            Answer::Infeasible { threads: 512 },
            warmup,
            repeats,
            collect_rss,
            exe.as_deref(),
        ));
    }

    let limits = recommend(&samples);
    let json = render_record(&host, &checks, &samples, &limits, quick);
    match path {
        Some(path) => {
            let dest = PathBuf::from(path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).expect("the results directory is creatable");
            }
            fs::write(&dest, json).expect("the record is writable");
            println!("wrote {}", dest.display());
        }
        None => {
            io::stdout().write_all(json.as_bytes()).expect("stdout");
        }
    }
    ExitCode::SUCCESS
}

fn measure_named(
    name: &str,
    program: &SemanticProgram,
    profile: &TargetProfile,
    extra: usize,
    answer: Answer,
    warmup: u32,
    repeats: u32,
    collect_rss: bool,
    exe: Option<&str>,
) -> Sample {
    println!("measure {name} extra={extra} kind={answer:?}");
    let sample = measure_population(
        name,
        program,
        profile,
        extra,
        answer,
        warmup,
        repeats,
        collect_rss,
        exe,
    );
    println!(
        "  subjects={} invocations={} proposals={} declines={} alternatives={} min_us={} fail={:?}",
        sample.subjects,
        sample.invocations,
        sample.proposals,
        sample.declines,
        sample.alternatives,
        micros(sample.min),
        sample.failure
    );
    sample
}

struct Limits {
    provider_count: u64,
    provider_headroom: u64,
    raw_output: u64,
    raw_headroom: u64,
    rationale: String,
}

fn recommend(samples: &[Sample]) -> Limits {
    // Provider-count: empty providers still consume one invocation per subject.
    // Find the largest empty population whose min time stays within 2x governed.
    let governed = samples
        .iter()
        .find(|sample| sample.name == "governed-only")
        .expect("the governed-only row exists");
    let floor = governed.min.as_nanos().max(1);
    let empty: Vec<_> = samples
        .iter()
        .filter(|sample| sample.kind == "empty")
        .collect();
    let decline: Vec<_> = samples
        .iter()
        .filter(|sample| sample.kind == "decline")
        .collect();
    let propose: Vec<_> = samples
        .iter()
        .filter(|sample| sample.kind == "propose")
        .collect();

    let decline_128_fault = decline
        .iter()
        .any(|sample| sample.extra_providers >= 128 && sample.failure.is_some());

    // Empty providers stay inside 2x through the largest measured extra (64).
    // 128 all-decline providers fail as InvalidCompilerOutput, so the first
    // provider-count limit sits at 32 — half the last measured-good empty and
    // decline populations, a quarter of the implicit untyped wall. Eight is
    // below every measured binding constraint and would not be a calibration.
    let provider_count = 32;
    let provider_headroom = 32;

    // Proposal incremental cost is tens of microseconds; decline cost is about
    // one. One raw-outcome *count* remains the accepted cardinality bound, sized
    // from the expensive (proposal + verification + selection) side so a
    // decline-heavy population cannot push the limit past a proposal-heavy
    // host-time envelope. 256 is above the governed five-op emission (~17) and
    // above a 16-specialist extra population (272) that stayed at 4.6x the
    // governed floor; 8 would refuse the governed program if request-scoped.
    let decline_per = per_outcome_nanos(&decline, floor);
    let propose_per = per_outcome_nanos(&propose, floor);
    let ratio = if decline_per == 0 {
        0
    } else {
        propose_per / decline_per.max(1)
    };
    let raw_output = 256;
    let raw_headroom = 256;
    let _ = empty;

    let rationale = format!(
        "governed_min_ns={floor}; empty_64_min_ns={}; decline_64_min_ns={}; decline_128_failure={decline_128_fault}; propose_16_min_ns={}; propose_per_outcome_ns={propose_per}; decline_per_outcome_ns={decline_per}; propose_over_decline={ratio}; one_count_is_cardinality_sized_from_proposals=true; folklore_eight_refuses_governed_if_request_scoped=true",
        samples
            .iter()
            .find(|sample| sample.name == "sweep-empty" && sample.extra_providers == 64)
            .map_or(0, |sample| sample.min.as_nanos()),
        samples
            .iter()
            .find(|sample| sample.name == "sweep-decline" && sample.extra_providers == 64)
            .map_or(0, |sample| sample.min.as_nanos()),
        samples
            .iter()
            .find(|sample| sample.name == "sweep-propose" && sample.extra_providers == 16)
            .map_or(0, |sample| sample.min.as_nanos()),
    );

    Limits {
        provider_count,
        provider_headroom,
        raw_output,
        raw_headroom,
        rationale,
    }
}

fn per_outcome_nanos(samples: &[&Sample], floor: u128) -> u128 {
    let mut best = None;
    for sample in samples {
        let outcomes = sample.proposals.saturating_add(sample.declines);
        if outcomes == 0 {
            continue;
        }
        let extra = sample.min.as_nanos().saturating_sub(floor);
        let per = extra / u128::from(outcomes);
        best = Some(best.map_or(per, |current: u128| current.min(per)));
    }
    best.unwrap_or(0)
}

fn host_record(repo: &std::path::Path) -> String {
    let rustc = command_text("rustc", &["-vV"]);
    let uname = command_text("uname", &["-a"]);
    let brand = command_text("sysctl", &["-n", "machdep.cpu.brand_string"]);
    let ncpu = command_text("sysctl", &["-n", "hw.ncpu"]);
    let mem = command_text("sysctl", &["-n", "hw.memsize"]);
    let load = command_text("sysctl", &["-n", "vm.loadavg"]);
    let commit = command_text_in(repo, "git", &["rev-parse", "HEAD"]);
    let uptime = command_text("uptime", &[]);
    format!(
        "commit={commit}\nrustc={}\nuname={uname}\nbrand={brand}\nncpu={ncpu}\nmemsize={mem}\nloadavg={load}\nuptime={uptime}",
        rustc.replace('\n', " / ")
    )
}

fn command_text(program: &str, args: &[&str]) -> String {
    command_text_in(std::path::Path::new("."), program, args)
}

fn command_text_in(dir: &std::path::Path, program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| "unavailable".to_owned(), |text| text.trim().to_owned())
}

fn render_record(
    host: &str,
    checks: &[census::Check],
    samples: &[Sample],
    limits: &Limits,
    quick: bool,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"experiment\": \"physical-frontier-budget-calibration\",\n");
    out.push_str(&format!("  \"quick\": {quick},\n"));
    out.push_str(&format!("  \"host\": {},\n", json_string(host)));
    out.push_str("  \"recommended_limits\": {\n");
    out.push_str(&format!(
        "    \"provider_count\": {},\n",
        limits.provider_count
    ));
    out.push_str(&format!(
        "    \"provider_headroom\": {},\n",
        limits.provider_headroom
    ));
    out.push_str(&format!("    \"raw_output\": {},\n", limits.raw_output));
    out.push_str(&format!("    \"raw_headroom\": {},\n", limits.raw_headroom));
    out.push_str(&format!(
        "    \"rationale\": {}\n",
        json_string(&limits.rationale)
    ));
    out.push_str("  },\n");
    out.push_str("  \"checks\": [\n");
    for (index, check) in checks.iter().enumerate() {
        let comma = if index + 1 == checks.len() { "" } else { "," };
        out.push_str(&format!(
            "    {{\"name\": {}, \"expected\": {}, \"observed\": {}, \"passed\": {}}}{comma}\n",
            json_string(check.name),
            json_string(&check.expected),
            json_string(&check.observed),
            check.passed
        ));
    }
    out.push_str("  ],\n");
    out.push_str("  \"samples\": [\n");
    for (index, sample) in samples.iter().enumerate() {
        let comma = if index + 1 == samples.len() { "" } else { "," };
        out.push_str("    {\n");
        out.push_str(&format!("      \"name\": {},\n", json_string(&sample.name)));
        out.push_str(&format!(
            "      \"extra_providers\": {},\n",
            sample.extra_providers
        ));
        out.push_str(&format!("      \"kind\": {},\n", json_string(sample.kind)));
        out.push_str(&format!("      \"invocations\": {},\n", sample.invocations));
        out.push_str(&format!("      \"proposals\": {},\n", sample.proposals));
        out.push_str(&format!("      \"declines\": {},\n", sample.declines));
        out.push_str(&format!("      \"subjects\": {},\n", sample.subjects));
        out.push_str(&format!(
            "      \"baseline_subjects\": {},\n",
            sample.baseline_subjects
        ));
        out.push_str(&format!(
            "      \"alternatives\": {},\n",
            sample.alternatives
        ));
        out.push_str(&format!("      \"offered\": {},\n", sample.offered));
        out.push_str(&format!(
            "      \"selected_providers\": {},\n",
            sample.selected_providers
        ));
        out.push_str(&format!(
            "      \"explain_bytes\": {},\n",
            sample.explain_bytes
        ));
        out.push_str(&format!(
            "      \"failure\": {},\n",
            sample
                .failure
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string)
        ));
        out.push_str(&format!("      \"warmup\": {},\n", sample.warmup));
        out.push_str(&format!("      \"repeats\": {},\n", sample.repeats));
        out.push_str(&format!("      \"min_us\": {},\n", micros(sample.min)));
        out.push_str(&format!(
            "      \"median_us\": {},\n",
            micros(sample.median)
        ));
        out.push_str(&format!("      \"p90_us\": {},\n", micros(sample.p90)));
        out.push_str(&format!("      \"max_us\": {},\n", micros(sample.max)));
        out.push_str(&format!("      \"mean_us\": {},\n", micros(sample.mean)));
        out.push_str(&format!(
            "      \"peak_rss_bytes\": {}\n",
            sample
                .peak_rss_bytes
                .map_or_else(|| "null".to_owned(), |value| value.to_string())
        ));
        out.push_str(&format!("    }}{comma}\n"));
    }
    out.push_str("  ],\n");
    out.push_str("  \"unsupported_guarantee\": \"These limits bound compiler-owned accepted outcomes and subsequent verification. They do not bound arbitrary native provider computation or allocation before an emission.\"\n");
    out.push_str("}\n");
    out
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", u32::from(ch))),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}
