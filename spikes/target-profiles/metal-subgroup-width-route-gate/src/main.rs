//! Demonstrates the prepared subgroup-width equality gate on the real Metal
//! route: an artifact-carried `ObservedEqualsRequired` requirement per entry,
//! answered from the exact prepared pipeline's `threadExecutionWidth`, with
//! exact equality committing and every refusal arriving before the routing
//! commit.
//!
//! The required width is derived from
//! `tiler_build::BoundMetalSubgroupDeclaration::first_m3_pro_apple9` — the
//! evidence-backed M3 Pro declaration — not from a literal written here, so the
//! green case is the declaration's row confirmed against the pipelines the
//! committed route dispatches. Runs only on the declaration's own execution
//! row (`Apple M3 Pro`, macOS build `26A5388g`) under its ledger toolchain;
//! any other host or toolchain is refused by name rather than measured.

mod device_io;
#[path = "../../../../crates/tiler-runtime/tests/adapter_route/fixture.rs"]
mod fixture;
#[path = "../../../../crates/tiler-runtime/tests/adapter_route/image.rs"]
mod image;
mod route;

use std::process::{Command, ExitCode};

use metal::{Device, MTLGPUFamily};
use sha2::{Digest, Sha256};
use tiler_artifact::program::DeferredPredicateSpec;
use tiler_build::BoundMetalSubgroupDeclaration;
use tiler_ir::program::abi::{
    AvailabilityPhase, PreparedEntryTargetRequirement, TargetPropertyKey,
    TargetPropertyProviderIdentity, TargetPropertyQuery, TargetPropertyRequirementRelation,
};
use tiler_runtime::load::LoadRejection;

use route::{CaseOutcome, ObserverMode};

/// The declaration's execution row this demonstration is scoped to.
const REQUIRED_HARDWARE: &str = "Apple M3 Pro";
/// The declaration's execution OS build.
const REQUIRED_OS_BUILD: &str = "26A5388g";
/// The declaration's offline compiler line, as `xcrun metal --version` opens.
const REQUIRED_METAL_VERSION: &str = "Apple metal version 32023.883 (metalfe-32023.883)";

/// The profile-strict offline flag vector the declaration's evidence used.
const PROFILE_STRICT_FLAGS: [&str; 5] = [
    "-std=metal4.0",
    "-target",
    "air64-apple-macos26.0",
    "-fmetal-math-mode=safe",
    "-fmetal-math-fp32-functions=precise",
];
/// The remaining strict flag, separated only because `-ffp-contract=off` and
/// `-fmetal-math-fp32-functions=precise` read better verified as one vector.
const PROFILE_STRICT_TAIL: [&str; 1] = ["-ffp-contract=off"];

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("demonstrate") => demonstrate(),
        _ => {
            eprintln!("usage: metal-subgroup-width-route-gate demonstrate");
            ExitCode::from(2)
        }
    }
}

/// Builds one governed subgroup-width row bound to `entry` at `required`.
fn subgroup_row(entry: u32, required: u64) -> DeferredPredicateSpec {
    row_with_key(entry, required, route::GOVERNED_SUBGROUP_KEY)
}

/// Builds a row naming a key this adapter does not own.
fn foreign_key_row(entry: u32, required: u64) -> DeferredPredicateSpec {
    row_with_key(
        entry,
        required,
        "tiler.target.prepared-entry.subgroup-width.v2",
    )
}

fn row_with_key(entry: u32, required: u64, key: &str) -> DeferredPredicateSpec {
    let query = TargetPropertyQuery::new(
        TargetPropertyKey::new(key).expect("a governed property key"),
        AvailabilityPhase::PreparedKernelPreflight,
        TargetPropertyProviderIdentity::new(
            route::PROVIDER_NAMESPACE,
            route::PROVIDER_NAME,
            route::PROVIDER_REVISION,
        )
        .expect("the provider identity"),
    )
    .expect("a well-formed target property query");
    DeferredPredicateSpec {
        requirement: PreparedEntryTargetRequirement::new(
            query,
            required,
            TargetPropertyRequirementRelation::ObservedEqualsRequired,
        )
        .expect("a well-formed prepared-entry requirement"),
        entry,
    }
}

fn run(program: &str, arguments: &[&str]) -> String {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("{program} {arguments:?} is runnable: {error}"));
    assert!(
        output.status.success(),
        "{program} {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Compiles both route kernels into one metallib under the profile-strict
/// flags, reporting each tool invocation.
fn compile_metallib() -> Vec<u8> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scratch = std::env::temp_dir().join(format!("tiler-subgroup-gate-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).expect("the scratch directory is creatable");
    let mut airs = Vec::new();
    for kernel in ["route_pointwise_f32", "route_reduce_f32"] {
        let source = root.join("kernels").join(format!("{kernel}.metal"));
        let bytes = std::fs::read(&source).expect("the kernel source is readable");
        println!("kernel {kernel}.metal sha256 {}", digest(&bytes));
        let air = scratch.join(format!("{kernel}.air"));
        let mut arguments: Vec<&str> = vec!["--sdk", "macosx", "metal"];
        arguments.extend(PROFILE_STRICT_FLAGS);
        arguments.extend(PROFILE_STRICT_TAIL);
        let source = source.to_str().expect("a UTF-8 source path").to_owned();
        let air_path = air.to_str().expect("a UTF-8 air path").to_owned();
        arguments.extend(["-c", &source, "-o", &air_path]);
        run("xcrun", &arguments);
        airs.push(air);
    }
    let metallib = scratch.join("route_gate.metallib");
    let mut arguments: Vec<&str> = vec!["--sdk", "macosx", "metallib"];
    let air_paths: Vec<String> = airs
        .iter()
        .map(|air| air.to_str().expect("a UTF-8 air path").to_owned())
        .collect();
    arguments.extend(air_paths.iter().map(String::as_str));
    let out = metallib.to_str().expect("a UTF-8 metallib path").to_owned();
    arguments.extend(["-o", &out]);
    run("xcrun", &arguments);
    let bytes = std::fs::read(&metallib).expect("the linked metallib is readable");
    std::fs::remove_dir_all(&scratch).expect("the scratch directory is removable");
    println!("metallib sha256 {}", digest(&bytes));
    bytes
}

/// Refuses to demonstrate on any host or toolchain but the declaration's own.
fn require_declared_row(device: &Device) -> Result<(), String> {
    let hardware = run("sysctl", &["-n", "machdep.cpu.brand_string"]);
    let build = run("sw_vers", &["-buildVersion"]);
    let metal_version = run("xcrun", &["--sdk", "macosx", "metal", "--version"]);
    let metal_version = metal_version.lines().next().unwrap_or_default();
    if hardware != REQUIRED_HARDWARE {
        return Err(format!(
            "this demonstration is scoped to the declaration's execution row: hardware is \
             {hardware:?}, the declared row is {REQUIRED_HARDWARE:?}",
        ));
    }
    if build != REQUIRED_OS_BUILD {
        return Err(format!(
            "this demonstration is scoped to the declaration's execution row: OS build is \
             {build:?}, the declared row is {REQUIRED_OS_BUILD:?}",
        ));
    }
    if metal_version != REQUIRED_METAL_VERSION {
        return Err(format!(
            "this demonstration is scoped to the declaration's offline toolchain: the host \
             answers {metal_version:?}, the declared row is {REQUIRED_METAL_VERSION:?}",
        ));
    }
    if !device.supports_family(MTLGPUFamily::Apple9) {
        return Err("the default device does not state Apple9".to_owned());
    }
    Ok(())
}

fn demonstrate() -> ExitCode {
    let Some(device) = Device::system_default() else {
        eprintln!("no default Metal device; this demonstration needs the declared host");
        return ExitCode::from(1);
    };
    println!(
        "host: device {:?}, registryID {:#x}, apple9 {}, os {} build {}, arch {}",
        device.name(),
        device.registry_id(),
        device.supports_family(MTLGPUFamily::Apple9),
        run("sw_vers", &["-productVersion"]),
        run("sw_vers", &["-buildVersion"]),
        run("uname", &["-m"]),
    );
    println!(
        "offline: {} | {} | xcode {} | sdk {} ({})",
        run("xcrun", &["--sdk", "macosx", "metal", "--version"])
            .lines()
            .next()
            .unwrap_or_default(),
        run("xcrun", &["--sdk", "macosx", "metallib", "-version"])
            .lines()
            .next()
            .unwrap_or_default(),
        run("xcodebuild", &["-version"]).replace('\n', " "),
        run("xcrun", &["--sdk", "macosx", "--show-sdk-version"]),
        run("xcrun", &["--sdk", "macosx", "--show-sdk-build-version"]),
    );
    println!("rustc: {}", run("rustc", &["--version"]));
    if let Err(reason) = require_declared_row(&device) {
        eprintln!("refused: {reason}");
        return ExitCode::from(1);
    }

    let declaration = BoundMetalSubgroupDeclaration::first_m3_pro_apple9()
        .expect("the evidence-backed M3 Pro subgroup declaration assembles");
    let subject = declaration.realized_subject();
    let declared_width = u64::from(subject.width().get());
    println!(
        "declaration: {} states Realized {{ width: {}, arithmetic: {:?}, transfer: {} }}",
        declaration.profile().profile_key().as_str(),
        subject.width().get(),
        subject.arithmetic(),
        subject.transfer().key(),
    );
    let descriptor =
        String::from_utf8_lossy(declaration.profile().canonical_descriptor()).into_owned();
    assert!(
        descriptor.contains(route::GOVERNED_SUBGROUP_KEY),
        "the declaration's descriptor names the governed prepared-width key",
    );

    let metallib = compile_metallib();
    let queue = device.new_command_queue();
    let mut failures = 0_usize;

    // ---- 1. exact equality on every entry commits and dispatches --------
    let outcome = route::attempt(
        &device,
        &queue,
        &metallib,
        vec![
            subgroup_row(0, declared_width),
            subgroup_row(1, declared_width),
        ],
        ObserverMode::Exact,
    );
    report("exact-equality-routes", &outcome);
    match &outcome.result {
        Ok(run)
            if run.output == route::expected()
                && run.widths.iter().all(|w| *w == declared_width) =>
        {
            println!(
                "  PASS: widths {:?} equal the declared {declared_width}, the route committed, \
                 and the dispatched output {:?} equals the strict reference",
                run.widths, run.output,
            );
        }
        Ok(run) => {
            println!(
                "  FAIL: committed but disagreed: widths {:?}, output {:?}, expected {:?}",
                run.widths,
                run.output,
                route::expected(),
            );
            failures += 1;
        }
        Err(rejection) => {
            println!("  FAIL: the exact-equality route was refused: {rejection:?}");
            failures += 1;
        }
    }

    // ---- 2. a width mismatch refuses before the commit ------------------
    let outcome = route::attempt(
        &device,
        &queue,
        &metallib,
        vec![subgroup_row(0, declared_width), subgroup_row(1, 16)],
        ObserverMode::Exact,
    );
    report("mismatch-refuses-pre-commit", &outcome);
    failures += expect_unsatisfied(&outcome, 1, declared_width);

    // ---- 3. an unrecognized key refuses before the commit ---------------
    let outcome = route::attempt(
        &device,
        &queue,
        &metallib,
        vec![
            subgroup_row(0, declared_width),
            foreign_key_row(1, declared_width),
        ],
        ObserverMode::Exact,
    );
    report("unknown-key-refuses-pre-commit", &outcome);
    failures += expect_unowned(&outcome);

    // ---- 4. an adapter without the dispatch refuses before the commit ---
    let outcome = route::attempt(
        &device,
        &queue,
        &metallib,
        vec![
            subgroup_row(0, declared_width),
            subgroup_row(1, declared_width),
        ],
        ObserverMode::PreSubgroupAdapter,
    );
    report("missing-dispatch-refuses-pre-commit", &outcome);
    failures += expect_unowned(&outcome);

    // ---- 5. the cross-pipeline substitution, both halves ----------------
    // 5a: with a requirement the substituted width cannot satisfy, the
    // second entry's own row refuses before the commit even though the
    // reported number is a genuine prepared width — of the wrong pipeline.
    let outcome = route::attempt(
        &device,
        &queue,
        &metallib,
        vec![subgroup_row(0, declared_width), subgroup_row(1, 16)],
        ObserverMode::FirstEntry,
    );
    report("cross-pipeline-substitution-refuses", &outcome);
    failures += expect_unsatisfied(&outcome, 1, declared_width);
    // 5b: the measured boundary, stated rather than hidden. On this host
    // every prepared pipeline reports the same width, so a substitution that
    // satisfies the equality is value-invisible here; the width-diverse
    // discrimination is held by the device-free loader tests
    // (`a_subgroup_width_row_is_a_per_entry_equality_and_never_a_floor`,
    // widths 4 and 8).
    let outcome = route::attempt(
        &device,
        &queue,
        &metallib,
        vec![
            subgroup_row(0, declared_width),
            subgroup_row(1, declared_width),
        ],
        ObserverMode::FirstEntry,
    );
    report("cross-pipeline-substitution-boundary", &outcome);
    match &outcome.result {
        Ok(_) => println!(
            "  BOUNDARY: on this host the substitution is value-invisible (every prepared \
             width is {declared_width}); the value-level discrimination stays device-free",
        ),
        Err(rejection) => {
            println!("  FAIL: the boundary case unexpectedly refused: {rejection:?}");
            failures += 1;
        }
    }

    if failures == 0 {
        println!("route gate demonstration: every case behaved as required");
        ExitCode::SUCCESS
    } else {
        eprintln!("route gate demonstration: {failures} case(s) failed");
        ExitCode::from(1)
    }
}

fn report(name: &str, outcome: &CaseOutcome) {
    println!("case {name}:");
    for observation in &outcome.observations {
        println!(
            "  loader asked entry {} for {:?}; answered {}",
            observation.entry, observation.key, observation.answer,
        );
    }
}

/// Requires a pre-commit `UnsatisfiedDeferredPredicate` naming the entry and
/// the observed width, and quotes it.
fn expect_unsatisfied(outcome: &CaseOutcome, entry: usize, observed_width: u64) -> usize {
    match &outcome.result {
        Err(
            rejection @ LoadRejection::UnsatisfiedDeferredPredicate {
                entry: named,
                observed,
                ..
            },
        ) if *named == entry && *observed == observed_width => {
            println!("  PASS, refused before the routing commit: {rejection:?}");
            0
        }
        Err(rejection) => {
            println!("  FAIL: refused, but not as the named entry's width row: {rejection:?}");
            1
        }
        Ok(_) => {
            println!("  FAIL: the route committed; the gate did not fire");
            1
        }
    }
}

/// Requires a pre-commit `UnownedPreparedEntryProperty`, and quotes it.
fn expect_unowned(outcome: &CaseOutcome) -> usize {
    match &outcome.result {
        Err(rejection @ LoadRejection::UnownedPreparedEntryProperty { .. }) => {
            println!("  PASS, refused before the routing commit: {rejection:?}");
            0
        }
        Err(rejection) => {
            println!("  FAIL: refused under the wrong class: {rejection:?}");
            1
        }
        Ok(_) => {
            println!("  FAIL: the route committed; the gate did not fire");
            1
        }
    }
}
