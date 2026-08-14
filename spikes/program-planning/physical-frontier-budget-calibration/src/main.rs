//! Census and host-runtime calibration of physical-frontier provider and raw-outcome budgets.

mod census;
mod custody;
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
use custody::{
    CUSTODY_SCHEMA, annotate_record, export_raw_artifacts, verify_evidence, verify_record_path,
    verify_record_text,
};
use measure::{
    Sample, child_measure, child_request_measure, measure_population, measure_request_population,
    micros, nanos, verify_sample_custody,
};
use profile::{declared_workgroup_profile, governed, request_profiles};
use program::{compile_request, five_op_program, tensor_add_chain};
use providers::{Answer, as_dyn, flock, shared_tally};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None | Some("record") => record(args.next(), false),
        Some("--quick") => record(args.next(), true),
        Some("census") => census_only(Perturb::None),
        Some("request-census") => request_census_only(),
        Some("verify-record") => {
            let Some(path) = args.next() else {
                eprintln!("usage: physical-frontier-budget-calibration verify-record <path>");
                return ExitCode::from(2);
            };
            match verify_record_path(&PathBuf::from(path)) {
                Ok(summary) => {
                    println!(
                        "PASS custody samples={} timing_values={} rss_rows={}",
                        summary.samples, summary.timing_values, summary.rss_rows
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("FAIL custody {error}");
                    ExitCode::from(1)
                }
            }
        }
        Some("export-raw") => {
            let paths = args.map(PathBuf::from).collect::<Vec<_>>();
            if let [record, timing, rss] = paths.as_slice() {
                report_custody_result(export_raw_artifacts(record, timing, rss))
            } else {
                eprintln!(
                    "usage: physical-frontier-budget-calibration export-raw <record> <timing.tsv> <rss.jsonl>"
                );
                ExitCode::from(2)
            }
        }
        Some("annotate-record") => {
            let paths = args.map(PathBuf::from).collect::<Vec<_>>();
            if let [generated, annotated, timing, rss] = paths.as_slice() {
                report_custody_result(annotate_record(generated, annotated, timing, rss))
            } else {
                eprintln!(
                    "usage: physical-frontier-budget-calibration annotate-record <generated.json> <annotated.json> <timing.tsv> <rss.jsonl>"
                );
                ExitCode::from(2)
            }
        }
        Some("verify-evidence") => {
            let paths = args.map(PathBuf::from).collect::<Vec<_>>();
            if let [generated, annotated, timing, rss] = paths.as_slice() {
                report_custody_result(verify_evidence(generated, annotated, timing, rss))
            } else {
                eprintln!(
                    "usage: physical-frontier-budget-calibration verify-evidence <generated.json> <annotated.json> <timing.tsv> <rss.jsonl>"
                );
                ExitCode::from(2)
            }
        }
        Some("request-boundary") => {
            let maximum = args
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(31);
            request_boundary(maximum)
        }
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
        Some("child-request-measure") => {
            let _name = args.next().unwrap_or_else(|| "child-request".to_owned());
            let targets = args
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(1);
            let extra = args
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            let kind = args.next().unwrap_or_else(|| "empty".to_owned());
            let program_kind = args.next().unwrap_or_else(|| "five-op".to_owned());
            let (program, contracts, profiles) =
                request_measurement_subject(&program_kind, targets, "test.child-request-measure");
            child_request_measure(&program, &contracts, &profiles, extra, &kind);
            ExitCode::SUCCESS
        }
        Some("help" | "--help") => {
            println!(
                "physical-frontier-budget-calibration [record [path]|verify-record <path>|export-raw <record> <timing.tsv> <rss.jsonl>|annotate-record <generated> <annotated> <timing.tsv> <rss.jsonl>|verify-evidence <generated> <annotated> <timing.tsv> <rss.jsonl>|census|request-census|request-boundary [maximum-specialists]|perturb <name>|--quick [path]]"
            );
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("unknown command {other}");
            ExitCode::from(2)
        }
    }
}

fn report_custody_result(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => {
            println!("PASS custody evidence");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("FAIL custody evidence {error}");
            ExitCode::from(1)
        }
    }
}

fn request_boundary(maximum_specialists: usize) -> ExitCode {
    let program = five_op_program(4, 3);
    for specialists in 1..=maximum_specialists {
        let tally = shared_tally();
        let providers = flock(
            "request-boundary",
            specialists,
            Answer::Specialize { threads: 32 },
            &tally,
        );
        let environment = tiler_compiler::physical_provider::InstalledPhysicalProviders::installed(
            as_dyn(&providers),
        )
        .expect("the boundary-control providers install");
        let profile =
            declared_workgroup_profile(&format!("test.request-boundary-{specialists}.v1"), 64);
        let compiled = compile_request(
            &program,
            [tiler_compiler::session::NumericalContract::STRICT_F32],
            [profile],
            &environment,
        );
        let tally = tally.borrow();
        println!(
            "specialists={specialists} successes={} invocations={} proposals={} declines={} raw={} alternatives={} explain_record_lines={} explain_bytes={} failure={:?} failure_tail={:?}",
            compiled.successes,
            tally.invocations,
            tally.proposals,
            tally.declines,
            tally.raw_outcomes(),
            compiled.alternatives,
            compiled.explain_record_lines,
            compiled.explain_bytes,
            compiled.failure,
            compiled.failure_explain_last_line,
        );
    }
    ExitCode::SUCCESS
}

#[derive(Debug)]
struct RequestPopulation {
    name: &'static str,
    targets: usize,
    target_keys: Vec<String>,
    resolved_contracts: Vec<String>,
    successes: usize,
    invocations: u64,
    subjects: u64,
    proposals: u64,
    declines: u64,
    alternatives: usize,
    rendered_explain_bytes: usize,
    failure: Option<String>,
}

fn request_census_only() -> ExitCode {
    for row in request_populations() {
        println!(
            "{} targets={} successes={} invocations={} subjects={} proposals={} declines={} alternatives={} rendered_explain_bytes={} contracts={:?} keys={:?} failure={:?}",
            row.name,
            row.targets,
            row.successes,
            row.invocations,
            row.subjects,
            row.proposals,
            row.declines,
            row.alternatives,
            row.rendered_explain_bytes,
            row.resolved_contracts,
            row.target_keys,
            row.failure,
        );
    }
    ExitCode::SUCCESS
}

fn request_populations() -> Vec<RequestPopulation> {
    let five = five_op_program(4, 3);
    let add = tensor_add_chain();
    let mut rows = Vec::new();
    for count in [1, 2, 8, 16] {
        let profiles = (0..count)
            .map(|index| {
                declared_workgroup_profile(&format!("test.request-strict-{count}-{index}.v1"), 64)
            })
            .collect::<Vec<_>>();
        rows.push(census_request(
            "five-op-strict",
            &five,
            [tiler_compiler::session::NumericalContract::STRICT_F32],
            profiles,
        ));
    }
    for count in [1, 2, 8, 16] {
        rows.push(census_request(
            "add-chain-four-contract-groups",
            &add,
            [
                tiler_compiler::session::NumericalContract::STRICT_F32,
                tiler_compiler::session::NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
                tiler_compiler::session::NumericalContract::REASSOCIATE_F32,
                tiler_compiler::session::NumericalContract::FLUSH_AND_REASSOCIATE_F32,
            ],
            request_profiles(&format!("test.request-contract-{count}"), count),
        ));
    }
    let mut reversed = request_profiles("test.request-reversed", 16);
    reversed.reverse();
    rows.push(census_request(
        "add-chain-reversed-target-order",
        &add,
        [
            tiler_compiler::session::NumericalContract::STRICT_F32,
            tiler_compiler::session::NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            tiler_compiler::session::NumericalContract::REASSOCIATE_F32,
            tiler_compiler::session::NumericalContract::FLUSH_AND_REASSOCIATE_F32,
        ],
        reversed,
    ));
    rows
}

fn census_request(
    name: &'static str,
    program: &SemanticProgram,
    contracts: impl IntoIterator<Item = tiler_compiler::session::NumericalContract>,
    profiles: Vec<TargetProfile>,
) -> RequestPopulation {
    let targets = profiles.len();
    let tally = shared_tally();
    let providers = flock(
        &format!("request-{}", name.replace('-', "_")),
        1,
        Answer::Specialize { threads: 32 },
        &tally,
    );
    let environment = tiler_compiler::physical_provider::InstalledPhysicalProviders::installed(
        as_dyn(&providers),
    )
    .expect("the request census provider installs");
    let compiled = compile_request(program, contracts, profiles, &environment);
    let tally = tally.borrow();
    RequestPopulation {
        name,
        targets,
        target_keys: compiled.target_keys,
        resolved_contracts: compiled.resolved_contracts,
        successes: compiled.successes,
        invocations: tally.invocations,
        subjects: tally.distinct_subjects(),
        proposals: tally.proposals,
        declines: tally.declines,
        alternatives: compiled.alternatives,
        rendered_explain_bytes: compiled.explain_bytes,
        failure: compiled.failure.map(|class| format!("{class:?}")),
    }
}

fn census_only(perturb: Perturb) -> ExitCode {
    let populations = request_populations();
    let mut checks = run_checks(&repo_root(), perturb);
    checks.extend(request_population_checks(&populations, perturb));
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
    let executable_evidence_commit = command_text_in(&repo, "git", &["rev-parse", "HEAD"]);
    println!("host {}", host.replace('\n', " | "));

    let populations = request_populations();
    let mut checks = run_checks(&repo, Perturb::None);
    checks.extend(request_population_checks(&populations, Perturb::None));
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

    let request_counts: &[usize] = if quick { &[1, 16] } else { &[1, 2, 8, 16] };
    for &count in request_counts {
        let (five, strict, strict_profiles) =
            request_measurement_subject("five-op", count, "test.measure-request-five");
        samples.push(measure_request_named(
            "request-five-governed",
            "five-op",
            &five,
            &strict,
            &strict_profiles,
            0,
            Answer::Empty,
            warmup,
            repeats,
            collect_rss,
            exe.as_deref(),
        ));
        samples.push(measure_request_named(
            "request-five-specialist",
            "five-op",
            &five,
            &strict,
            &strict_profiles,
            1,
            Answer::Specialize { threads: 32 },
            warmup,
            repeats,
            collect_rss,
            exe.as_deref(),
        ));
        if count == 16 {
            for extra in [2, 31] {
                samples.push(measure_request_named(
                    if extra == 2 {
                        "request-five-two-specialists"
                    } else {
                        "request-five-full-32-provider-population"
                    },
                    "five-op",
                    &five,
                    &strict,
                    &strict_profiles,
                    extra,
                    Answer::Specialize { threads: 32 },
                    warmup,
                    repeats,
                    collect_rss,
                    exe.as_deref(),
                ));
            }
        }
        let (add, contracts, grouped_profiles) =
            request_measurement_subject("add-chain", count, "test.measure-request-add");
        samples.push(measure_request_named(
            "request-add-four-groups",
            "add-chain",
            &add,
            &contracts,
            &grouped_profiles,
            1,
            Answer::Specialize { threads: 32 },
            warmup,
            repeats,
            collect_rss,
            exe.as_deref(),
        ));
    }
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

    for sample in &samples {
        if let Err(error) = verify_sample_custody(sample, collect_rss) {
            eprintln!("FAIL in-memory custody {error}");
            return ExitCode::from(1);
        }
    }

    let limits = recommend(&samples, &populations);
    let json = render_record(
        &host,
        &executable_evidence_commit,
        &checks,
        &populations,
        &samples,
        &limits,
        quick,
    );
    let custody = match verify_record_text(&json) {
        Ok(summary) => summary,
        Err(error) => {
            eprintln!("FAIL rendered custody {error}");
            return ExitCode::from(1);
        }
    };
    println!(
        "custody verified samples={} timing_values={} rss_rows={}",
        custody.samples, custody.timing_values, custody.rss_rows
    );
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

#[allow(
    clippy::too_many_arguments,
    reason = "the measurement call states every workload and noise control explicitly"
)]
fn measure_request_named(
    name: &str,
    program_kind: &'static str,
    program: &SemanticProgram,
    contracts: &[tiler_compiler::session::NumericalContract],
    profiles: &[TargetProfile],
    extra: usize,
    answer: Answer,
    warmup: u32,
    repeats: u32,
    collect_rss: bool,
    exe: Option<&str>,
) -> Sample {
    println!(
        "measure {name} targets={} extra={extra} kind={answer:?}",
        profiles.len()
    );
    let sample = measure_request_population(
        name,
        program_kind,
        program,
        contracts,
        profiles,
        extra,
        answer,
        warmup,
        repeats,
        collect_rss,
        exe,
    );
    println!(
        "  targets={} subjects={} invocations={} proposals={} declines={} alternatives={} min_us={} fail={:?}",
        sample.targets,
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

fn request_measurement_subject(
    program_kind: &str,
    targets: usize,
    prefix: &str,
) -> (
    SemanticProgram,
    Vec<tiler_compiler::session::NumericalContract>,
    Vec<TargetProfile>,
) {
    match program_kind {
        "five-op" => (
            five_op_program(4, 3),
            vec![tiler_compiler::session::NumericalContract::STRICT_F32],
            (0..targets)
                .map(|index| {
                    declared_workgroup_profile(&format!("{prefix}-{targets}-{index}.v1"), 64)
                })
                .collect(),
        ),
        "add-chain" => (
            tensor_add_chain(),
            vec![
                tiler_compiler::session::NumericalContract::STRICT_F32,
                tiler_compiler::session::NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
                tiler_compiler::session::NumericalContract::REASSOCIATE_F32,
                tiler_compiler::session::NumericalContract::FLUSH_AND_REASSOCIATE_F32,
            ],
            request_profiles(&format!("{prefix}-{targets}"), targets),
        ),
        other => panic!("unknown request measurement program {other}"),
    }
}

struct Limits {
    provider_count: u64,
    provider_headroom: u64,
    raw_output: Option<u64>,
    narrow_raw_output: u64,
    narrow_raw_headroom: u64,
    full_provider_raw_output: u64,
    full_provider_raw_headroom: u64,
    rationale: String,
}

fn recommend(samples: &[Sample], populations: &[RequestPopulation]) -> Limits {
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
    // host-time envelope. The compiler-owned census pins 304 governed outcomes
    // over sixteen strict targets; each installed specialist adds the public
    // harness's 272. Two specialists therefore produce 848, whose next power
    // of two is 1,024. The competing complete-cardinality reading is governed
    // plus 31 installed specialists: 8,736 outcomes, whose next power of two is
    // 16,384. The harness reports both until policy says whether three or more
    // active specialists are intentionally refused; measurement cannot decide
    // that support boundary.
    let decline_per = per_outcome_nanos(&decline, floor);
    let propose_per = per_outcome_nanos(&propose, floor);
    let ratio = if decline_per == 0 {
        0
    } else {
        propose_per / decline_per.max(1)
    };
    let installed_request_outcomes = populations
        .iter()
        .find(|row| row.name == "five-op-strict" && row.targets == 16)
        .map_or(0, |row| row.proposals.saturating_add(row.declines));
    let governed_request_outcomes = 304_u64;
    let narrow_population =
        governed_request_outcomes.saturating_add(installed_request_outcomes.saturating_mul(2));
    let narrow_raw_output = narrow_population.next_power_of_two();
    let narrow_raw_headroom = narrow_raw_output.saturating_sub(narrow_population);
    let full_provider_population = governed_request_outcomes
        .saturating_add(installed_request_outcomes.saturating_mul(provider_count - 1));
    let full_provider_raw_output = full_provider_population.next_power_of_two();
    let full_provider_raw_headroom =
        full_provider_raw_output.saturating_sub(full_provider_population);
    let _ = empty;

    let rationale = format!(
        "governed_min_ns={floor}; empty_64_min_ns={}; decline_64_min_ns={}; decline_128_failure={decline_128_fault}; propose_16_min_ns={}; propose_per_outcome_ns={propose_per}; decline_per_outcome_ns={decline_per}; propose_over_decline={ratio}; request_governed_outcomes={governed_request_outcomes}; request_installed_specialist_outcomes={installed_request_outcomes}; two_specialist_population={narrow_population}; three_specialist_population={}; full_32_provider_population={full_provider_population}; raw_value_held_for_population_policy=true; one_count_is_request_scoped=true",
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
        governed_request_outcomes.saturating_add(installed_request_outcomes.saturating_mul(3)),
    );

    Limits {
        provider_count,
        provider_headroom,
        raw_output: None,
        narrow_raw_output,
        narrow_raw_headroom,
        full_provider_raw_output,
        full_provider_raw_headroom,
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
    executable_evidence_commit: &str,
    checks: &[census::Check],
    populations: &[RequestPopulation],
    samples: &[Sample],
    limits: &Limits,
    quick: bool,
) -> String {
    let timing_values = samples
        .iter()
        .map(|sample| sample.timed_durations.len())
        .sum::<usize>();
    let rss_rows = samples.iter().filter(|sample| sample.rss.is_some()).count();
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"experiment\": \"physical-frontier-budget-calibration\",\n");
    out.push_str("  \"behavior_under_test_base\": \"4fb0427319b1504e1549e03ba023ac486343a743\",\n");
    out.push_str(&format!(
        "  \"executable_evidence_commit\": {},\n",
        json_string(executable_evidence_commit)
    ));
    out.push_str(&format!("  \"quick\": {quick},\n"));
    out.push_str(&format!("  \"host\": {},\n", json_string(host)));
    out.push_str("  \"custody\": {\n");
    out.push_str(&format!(
        "    \"schema\": {},\n",
        json_string(CUSTODY_SCHEMA)
    ));
    out.push_str("    \"timing_unit\": \"nanoseconds\",\n");
    out.push_str("    \"timing_order\": \"execution-order\",\n");
    out.push_str("    \"summary_algorithm\": \"sort a copy; min=0, median=n/2, p90=(9n-1)/10, max=n-1, mean=floor(sum/n); convert each result to integer microseconds by floor division\",\n");
    out.push_str("    \"rss_source\": \"complete macOS /usr/bin/time -l stderr\",\n");
    out.push_str(&format!("    \"verified_samples\": {},\n", samples.len()));
    out.push_str(&format!(
        "    \"verified_timing_values\": {timing_values},\n"
    ));
    out.push_str(&format!("    \"verified_rss_rows\": {rss_rows}\n"));
    out.push_str("  },\n");
    out.push_str("  \"candidate_limits\": {\n");
    out.push_str(&format!(
        "    \"provider_count\": {},\n",
        limits.provider_count
    ));
    out.push_str(&format!(
        "    \"provider_headroom\": {},\n",
        limits.provider_headroom
    ));
    out.push_str(&format!(
        "    \"raw_output\": {},\n",
        limits
            .raw_output
            .map_or_else(|| "null".to_owned(), |value| value.to_string())
    ));
    out.push_str(&format!(
        "    \"narrow_raw_output\": {},\n",
        limits.narrow_raw_output
    ));
    out.push_str(&format!(
        "    \"narrow_raw_headroom\": {},\n",
        limits.narrow_raw_headroom
    ));
    out.push_str(&format!(
        "    \"full_provider_raw_output\": {},\n",
        limits.full_provider_raw_output
    ));
    out.push_str(&format!(
        "    \"full_provider_raw_headroom\": {},\n",
        limits.full_provider_raw_headroom
    ));
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
    out.push_str("  \"request_populations\": [\n");
    for (index, row) in populations.iter().enumerate() {
        let comma = if index + 1 == populations.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!(
            "    {{\"name\": {}, \"targets\": {}, \"successes\": {}, \"invocations\": {}, \"subjects\": {}, \"proposals\": {}, \"declines\": {}, \"alternatives\": {}, \"rendered_explain_bytes\": {}, \"resolved_contracts\": {}, \"target_keys\": {}, \"failure\": {}}}{comma}\n",
            json_string(row.name),
            row.targets,
            row.successes,
            row.invocations,
            row.subjects,
            row.proposals,
            row.declines,
            row.alternatives,
            row.rendered_explain_bytes,
            json_strings(&row.resolved_contracts),
            json_strings(&row.target_keys),
            row.failure.as_deref().map_or_else(|| "null".to_owned(), json_string),
        ));
    }
    out.push_str("  ],\n");
    out.push_str("  \"samples\": [\n");
    for (index, sample) in samples.iter().enumerate() {
        let comma = if index + 1 == samples.len() { "" } else { "," };
        out.push_str("    {\n");
        out.push_str(&format!("      \"sample_index\": {index},\n"));
        out.push_str(&format!(
            "      \"series_key\": {},\n",
            json_string(&series_key(index, sample))
        ));
        out.push_str(&format!("      \"name\": {},\n", json_string(&sample.name)));
        out.push_str(&format!(
            "      \"program_kind\": {},\n",
            json_string(sample.program_kind)
        ));
        out.push_str(&format!(
            "      \"request_wide\": {},\n",
            sample.request_wide
        ));
        out.push_str(&format!("      \"targets\": {},\n", sample.targets));
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
        out.push_str(&format!(
            "      \"timed_durations_ns\": {},\n",
            json_durations(&sample.timed_durations)
        ));
        out.push_str(&format!("      \"min_us\": {},\n", micros(sample.min)));
        out.push_str(&format!(
            "      \"median_us\": {},\n",
            micros(sample.median)
        ));
        out.push_str(&format!("      \"p90_us\": {},\n", micros(sample.p90)));
        out.push_str(&format!("      \"max_us\": {},\n", micros(sample.max)));
        out.push_str(&format!("      \"mean_us\": {},\n", micros(sample.mean)));
        out.push_str(&format!(
            "      \"peak_rss_bytes\": {},\n",
            sample
                .peak_rss_bytes
                .map_or_else(|| "null".to_owned(), |value| value.to_string())
        ));
        match &sample.rss {
            Some(rss) => {
                out.push_str("      \"rss\": {\n");
                out.push_str(&format!(
                    "        \"command\": {},\n",
                    json_strings(&rss.command)
                ));
                out.push_str(&format!(
                    "        \"child_exit_success\": {},\n",
                    rss.child_exit_success
                ));
                out.push_str(&format!(
                    "        \"child_exit_code\": {},\n",
                    rss.child_exit_code
                        .map_or_else(|| "null".to_owned(), |code| code.to_string())
                ));
                out.push_str(&format!(
                    "        \"time_stderr\": {},\n",
                    json_string(&rss.time_stderr)
                ));
                out.push_str(&format!(
                    "        \"parsed_peak_rss_bytes\": {}\n",
                    rss.parsed_peak_rss_bytes
                        .map_or_else(|| "null".to_owned(), |value| value.to_string())
                ));
                out.push_str("      }\n");
            }
            None => out.push_str("      \"rss\": null\n"),
        }
        out.push_str(&format!("    }}{comma}\n"));
    }
    out.push_str("  ],\n");
    out.push_str("  \"unsupported_guarantee\": \"These limits bound compiler-owned accepted outcomes and subsequent verification. They do not bound arbitrary native provider computation or allocation before an emission.\"\n");
    out.push_str("}\n");
    out
}

fn request_population_checks(
    populations: &[RequestPopulation],
    perturb: Perturb,
) -> Vec<census::Check> {
    let strict_rows = populations
        .iter()
        .filter(|row| row.name == "five-op-strict")
        .collect::<Vec<_>>();
    let strict_sixteen = strict_rows
        .iter()
        .copied()
        .find(|row| row.targets == 16)
        .expect("the sixteen-target row exists");
    let grouped = populations
        .iter()
        .find(|row| row.name == "add-chain-four-contract-groups" && row.targets == 16)
        .expect("the grouped sixteen-target row exists");
    let reversed = populations
        .iter()
        .find(|row| row.name == "add-chain-reversed-target-order")
        .expect("the reversed-order row exists");
    let distinct_contracts = grouped
        .resolved_contracts
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let expected_reversed = (0..16)
        .rev()
        .map(|index| format!("test.request-reversed-{index}.v1"))
        .collect::<Vec<_>>();
    let specialists = if perturb == Perturb::LimitRecommendationPopulation {
        3_u64
    } else {
        2_u64
    };
    let narrow_candidate = 304_u64
        .saturating_add(
            strict_sixteen
                .proposals
                .saturating_add(strict_sixteen.declines)
                .saturating_mul(specialists),
        )
        .next_power_of_two();
    let full_specialists = if perturb == Perturb::FullLimitPopulation {
        29_u64
    } else {
        31_u64
    };
    let full_provider_candidate = 304_u64
        .saturating_add(
            strict_sixteen
                .proposals
                .saturating_add(strict_sixteen.declines)
                .saturating_mul(full_specialists),
        )
        .next_power_of_two();
    vec![
        check_eq(
            "request-target-count-population",
            "1,2,8,16",
            strict_rows
                .iter()
                .map(|row| row.targets.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ),
        check_eq(
            "request-strict-16-invocations",
            272,
            strict_sixteen.invocations,
        ),
        check_eq("request-strict-16-proposals", 48, strict_sixteen.proposals),
        check_eq("request-strict-16-declines", 224, strict_sixteen.declines),
        check_eq(
            "request-strict-16-rendered-explain-bytes",
            1_651_952,
            strict_sixteen.rendered_explain_bytes,
        ),
        check_eq("request-grouped-contract-count", 4, distinct_contracts),
        check_eq(
            "request-grouped-candidate-invocations",
            248,
            grouped.invocations,
        ),
        check_eq("request-grouped-proposals", 24, grouped.proposals),
        check_eq("request-grouped-declines", 224, grouped.declines),
        check_eq(
            "request-grouped-rendered-explain-bytes",
            1_030_032,
            grouped.rendered_explain_bytes,
        ),
        check_eq(
            "request-reversed-target-order",
            format!("{expected_reversed:?}"),
            format!("{:?}", reversed.target_keys),
        ),
        check_eq("request-narrow-limit-calculation", 1024, narrow_candidate),
        check_eq(
            "request-full-provider-limit-calculation",
            16_384,
            full_provider_candidate,
        ),
    ]
}

#[allow(clippy::needless_pass_by_value)]
fn check_eq(name: &'static str, expected: impl ToString, observed: impl ToString) -> census::Check {
    let expected = expected.to_string();
    let observed = observed.to_string();
    census::Check {
        name,
        passed: expected == observed,
        expected,
        observed,
    }
}

fn json_strings(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn json_durations(values: &[std::time::Duration]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .copied()
            .map(nanos)
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn series_key(index: usize, sample: &Sample) -> String {
    format!(
        "{index}:{}:targets={}:providers={}:kind={}:program={}",
        sample.name, sample.targets, sample.extra_providers, sample.kind, sample.program_kind
    )
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
