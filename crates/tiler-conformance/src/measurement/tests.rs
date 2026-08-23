//! The two properties that make an unmeasurable host safe, watched separately.
//!
//! They are independent, and a check that reddened for both would not say which
//! one is load-bearing:
//!
//! 1. **An unavailable outcome has no path to a pass.** It is a *type* property
//!    of [`Measured`] and [`Reported`] — neither compares, neither has a
//!    fabricable value, and the only accessor answers `None` for a host that did
//!    not measure — plus the runtime property that a caller's policy is actually
//!    honoured. The type half is a source census, because a derive that is
//!    absent cannot be observed from inside a running test; the runtime half is
//!    [`apply_policy`] watched under both policies against one identical
//!    outcome.
//! 2. **The gate reads no ambient input.** It is a property of *where* the
//!    policy is decided rather than of any value, so it is a source census too:
//!    the crate's whole population of standard-environment accesses, classified,
//!    with the one policy read required to be in the module whose entire content
//!    is that read.
//!
//! # The needles are assembled at run time
//!
//! The second census walks every source file under `src`, this one included, so
//! a literal spelling of the path it searches for would match itself and this
//! file would classify as an unclassified reader of the environment. Assembled
//! at run time it cannot, which is the same reason `crate::portability` gives
//! for the same trick. The prose here therefore says "the standard environment
//! module" where it would rather write the path.

use std::path::{Path, PathBuf};

use super::ambient::REQUIRE_MEASUREMENT;
use super::{
    HostPolicy, Measured, MeasurementBoundary, Refused, Reported, Unavailable, apply_policy,
};

/// A row no device produced, for reaching the observed arm without a device.
///
/// Every field is a placeholder and says so: this value is never compared
/// against a retained record and never printed as evidence of anything. It
/// exists so the tests below can hold a completed outcome beside an unavailable
/// one and watch a policy treat them differently.
fn unmeasured_row() -> MeasurementBoundary {
    let placeholder = || "unobserved (this row was never measured)".to_owned();
    MeasurementBoundary {
        os_family: placeholder(),
        os_version: placeholder(),
        os_build: placeholder(),
        architecture: placeholder(),
        device_name: placeholder(),
        gpu_family: placeholder(),
        metal_compiler: placeholder(),
        metallib_linker: placeholder(),
        sdk: placeholder(),
        profile_key: placeholder(),
        aot_target: placeholder(),
        language_standard: placeholder(),
        metallib_bytes: 0,
    }
}

/// The policy a caller states when it will not accept an unmeasurable host.
const REQUIRING: HostPolicy = HostPolicy::Require {
    named: REQUIRE_MEASUREMENT,
};

/// An unavailable measured half is typed and has no path to a pass.
///
/// One identical [`Measured::Unavailable`] value is applied under both policies
/// and the answers differ; nothing in this test touches the environment, which
/// is what makes both halves reachable in an ordinary run on any host.
#[test]
fn an_unavailable_measured_half_is_typed_and_cannot_pass() {
    let unavailable = Unavailable::new(
        "no qualified Apple Metal toolchain resolved: this host offers no offline compiler"
            .to_owned(),
    );

    // The caller that reports unavailability receives an outcome, and the
    // outcome publishes nothing: `observed` answers `None`, and every consumer
    // of a measured half in this crate goes through it.
    let reported = apply_policy::<Vec<u16>>(
        HostPolicy::Report,
        Measured::Unavailable(unavailable.clone()),
    )
    .expect("a reporting caller receives an outcome rather than a refusal");
    match &reported {
        Reported::Boundary(boundary) => {
            eprintln!("measurement boundary unavailable: {boundary}");
            assert!(
                boundary
                    .reason()
                    .contains("no qualified Apple Metal toolchain"),
                "the reported boundary must name what the host could not supply rather than a \
                 rewriting of it; it said: {reason}",
                reason = boundary.reason(),
            );
        }
        Reported::Observed { .. } => {
            panic!("a host that could not measure reported an observation")
        }
    }
    assert!(
        reported.observed().is_none(),
        "an unavailable host publishes no observation, so no expression in this crate can compare \
         one against an expectation",
    );

    // The caller that requires the host receives a refusal. The measured half's
    // report is identical in both runs; only the caller's policy differs.
    let refused = apply_policy::<Vec<u16>>(REQUIRING, Measured::Unavailable(unavailable.clone()))
        .expect_err("a requiring caller does not accept an unavailable host");
    eprintln!("requiring caller: {refused}");
    assert_eq!(
        refused,
        Refused::Required {
            named: REQUIRE_MEASUREMENT,
            unavailable,
        },
        "the refusal must name the authority that required the measured half and carry what was \
         missing",
    );
    assert!(
        refused
            .to_string()
            .contains("is set and the measured half is unavailable"),
        "two landed tickets quote this sentence as the evidence that an unmeasurable host can be \
         made a hard failure; it said: {refused}",
    );

    // The same requiring policy publishes an observation the device did make,
    // so the refusal above is the policy answering rather than the policy being
    // the only answer.
    let observed = apply_policy(
        REQUIRING,
        Measured::Ran {
            boundary: Box::new(unmeasured_row()),
            observed: vec![0x3f80_u16],
        },
    )
    .expect("a measured half that ran is refused by no policy")
    .observed()
    .expect("a completed measurement publishes what the device observed");
    assert_eq!(observed, vec![0x3f80_u16]);
}

/// A refused stage is a defect under either policy, never a boundary.
///
/// The distinction the module header rests on: an absent machine is a boundary a
/// caller may choose to report, and a stage that was reached and said no is a
/// defect no policy may report away.
#[test]
fn a_refused_stage_is_a_defect_under_either_policy() {
    for policy in [HostPolicy::Report, REQUIRING] {
        let refused = apply_policy::<Vec<u16>>(
            policy,
            Measured::Failed("the dispatch did not complete".to_owned()),
        )
        .expect_err("a stage that was reached and refused is not an outcome any policy accepts");
        assert_eq!(
            refused,
            Refused::Defect("the dispatch did not complete".to_owned()),
            "{policy:?} turned a refused stage into something other than a defect",
        );
    }
}

/// The derives that would give a measured outcome a way to compare or fabricate.
///
/// `Eq` is a substring of `PartialEq`, so the first entry alone would already
/// catch both; both are named because the failure should say which one was
/// restored, and `Default` is an independent way to fabricate a value.
const FORBIDDEN_DERIVES: [&str; 3] = ["PartialEq", "Eq", "Default"];

/// Neither measured-outcome type admits a comparison or a fabricable value.
///
/// **What it takes for this to say no:** re-adding `PartialEq`, `Eq`, or
/// `Default` to either type's derive, deleting the derive, or renaming either
/// type without renaming it here — the last fails because the declaration is
/// looked up by its exact spelling and a lookup that finds nothing panics rather
/// than reporting a clean absence.
#[test]
fn the_measured_outcome_types_admit_no_comparison_and_no_default() {
    let gate = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/measurement.rs");
    let source = std::fs::read_to_string(&gate).expect("this module's parent source is readable");

    for declaration in [
        "pub(crate) enum Measured<T> {",
        "pub(crate) enum Reported<T> {",
    ] {
        let derive = sole_derive(&source, declaration);
        eprintln!("{declaration} derives: {derive}");
        for forbidden in FORBIDDEN_DERIVES {
            assert!(
                !derive.contains(forbidden),
                "`{declaration}` derives `{forbidden}`, so an unavailable outcome has acquired a \
                 way to be compared against or fabricated as a measured one. The whole reason \
                 these two carry only `Clone` and `Debug` is that the absence is the guarantee; \
                 it derives: {derive}",
            );
        }
    }
}

/// Returns the contents of the one derive attached to a named declaration.
///
/// Reads to the closing `)]` rather than to the end of the line, so a derive
/// that has grown long enough for the formatter to wrap it is still read whole.
fn sole_derive<'source>(source: &'source str, declaration: &str) -> &'source str {
    let block = source
        .split("\n\n")
        .find(|block| block.contains(declaration))
        .unwrap_or_else(|| {
            panic!(
                "`{declaration}` is not declared in `measurement.rs`. A census that cannot find \
                 its subject reports absence as cleanliness, so this is a failure rather than a \
                 skip: either the type was renamed and this list was not, or its declaration is \
                 no longer separated from the item above it by a blank line.",
            )
        });
    let opener = "#[derive(";
    let derives = block.matches(opener).count();
    assert_eq!(
        derives, 1,
        "`{declaration}` carries {derives} derive attribute(s); this census reads exactly one",
    );
    let after = &block[block.find(opener).expect("the count above found one") + opener.len()..];
    let end = after
        .find(")]")
        .unwrap_or_else(|| panic!("`{declaration}`'s derive attribute is not closed"));
    &after[..end]
}

/// The crate's whole population of standard-environment accesses, classified.
///
/// Every one must be a host fact, a scratch directory, or the single policy read
/// in the module whose entire content is that read. The gate's own source is
/// required to hold none.
///
/// **What it takes for this to say no:** any access to the standard environment
/// module anywhere under `src` that is not one of those three, an import of that
/// module (which would let an unqualified reader hide from a path search), or a
/// second policy read appearing beside the first. **What could stop it reaching
/// its subject:** a walk that found no files, refused below by the host-fact
/// floor; and a dependency that reads the environment on this crate's behalf,
/// which nothing here can see — the claim is about what this crate does, not
/// about the process.
#[test]
fn the_gate_reads_no_ambient_input_and_the_one_policy_read_is_named() {
    // Assembled so this scanner does not match its own source; see the module
    // header.
    let path_form = format!("{}{}", "env", "::");
    let import_form = format!("use std::{}", "env");
    let host_facts = format!("{path_form}consts::");
    let scratch = format!("{path_form}temp_dir");

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    crate::portability::collect_rust_sources(&root, &mut files);
    files.sort();

    let gate = root.join("measurement.rs");
    let resolver = root.join("measurement").join("ambient.rs");
    for required in [&gate, &resolver] {
        assert!(
            files.contains(required),
            "{}: the walk did not find this file, so the census below says nothing about it",
            required.display(),
        );
    }

    let mut host_fact_reads = 0_usize;
    let mut scratch_reads = 0_usize;
    let mut policy_reads: Vec<(PathBuf, usize)> = Vec::new();
    let mut unclassified: Vec<(PathBuf, usize)> = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a crate source file is readable");
        assert!(
            !text.contains(import_form.as_str()),
            "{}: imports the standard environment module. An import lets an unqualified reader \
             sit in this crate without naming the path this census searches for, so the \
             classification below would report it as clean.",
            path.display(),
        );
        let total = text.matches(path_form.as_str()).count();
        if total == 0 {
            continue;
        }
        let consts = text.matches(host_facts.as_str()).count();
        let temporary = text.matches(scratch.as_str()).count();
        host_fact_reads += consts;
        scratch_reads += temporary;
        let remainder = total - consts - temporary;
        if remainder == 0 {
            continue;
        }
        if path == &resolver {
            policy_reads.push((path.clone(), remainder));
        } else {
            unclassified.push((path.clone(), remainder));
        }
    }

    eprintln!(
        "ambient census: {} source file(s); {host_fact_reads} host-fact access(es), \
         {scratch_reads} scratch-directory access(es), and {policy_reads:?} policy read(s)",
        files.len(),
    );

    assert!(
        unclassified.is_empty(),
        "these source files access the standard environment module for something that is neither \
         a host fact nor a scratch directory: {unclassified:?}. The policy an unmeasurable host \
         gets is the caller's, and the one ambient input this crate reads lives in {}; a second \
         reader is a second unexercised path, and one inside the gate is the shape this module \
         exists to keep out.",
        resolver.display(),
    );
    assert_eq!(
        policy_reads,
        vec![(resolver.clone(), 1_usize)],
        "the policy read must be exactly one access, in {}",
        resolver.display(),
    );
    assert!(
        host_fact_reads >= 1,
        "the walk classified {host_fact_reads} host-fact access(es) across {} file(s). This crate \
         observes its own operating system and architecture, so zero means the scan stopped \
         matching and an empty population is being reported as a clean one.",
        files.len(),
    );
}
