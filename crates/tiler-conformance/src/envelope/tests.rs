//! The envelope route's runs, in two halves that fail for unrelated reasons.
//!
//! Everything above [`the_published_matrix_agrees_with_its_record_on_the_routed_row`]
//! is decidable without an envelope and runs on every host in the ordinary gate:
//! the interface recognizers and every way of missing them, the sidecar
//! payload-length refusal, the routed dtype rows, the member and class pins, the
//! digest helper against its published vectors and its own domain, and the
//! retained comparison's two verdicts including the perturbation it must refuse.
//!
//! The routed runs need an envelope a *separate producer* wrote, and this crate
//! does not hold the scope that produces one. They therefore report the artifact
//! boundary as unavailable — naming the producer command — rather than skipping,
//! exactly as an unmeasured host reports its device row.

use std::path::Path;

use tiler_artifact::program::ArithmeticType;
use tiler_build::DTypeDispatchability;
use tiler_runtime::load::{DTypeDispatch, DTypeDispatchResolution};

use super::{
    ARTIFACT_BASE, CONTRACTION_ACTIVATIONS_KEY, CONTRACTION_CLASS, CONTRACTION_MEMBERS,
    CONTRACTION_OUTPUT_KEY, CONTRACTION_WEIGHTS_KEY, DeclaredInput, DeclaredInterface,
    EnvelopeFailure, L3_CELL_CLASS, L3_CELL_RESULT_SHA256, PLAN_ROLES, PRODUCER_COMMAND,
    REDUCTION_CLASSES, RetainedComparison, SIDECAR_SUFFIX, declared_route_environment,
    decode_f32_bits, dtype_rows, expected_shape, host_dtype_dispatch, measured_contraction,
    measured_matrix, proof_member, require_contraction_interface, require_or_report_base,
    require_serial_sum_interface, result_digest, sha256_hex,
};
use crate::measurement::require_or_report;
use crate::serial_sum::{COLUMNS, declaration};

/// The published contraction's extents, restated here as the consumer's half of
/// a pinned pair.
///
/// `prototypes/serial-sum-compile` states the same three numbers and the same
/// class name. Nothing links the two crates, so this pair is the only thing
/// comparing them — the same arrangement [`SIDECAR_SUFFIX`] and the reduction
/// matrix are under, and for the same reason: a producer that moved the published
/// contraction while this half kept opening the old shape would leave a green
/// gate over a member that cannot route.
const FIXTURE_CONTRACTION: (u64, u64, u64) = (2, 2, 3);

/// The published L3 cell's extents, restated here for the same reason.
const FIXTURE_L3_CELL: (u64, u64, u64) = (1, 1024, 1024);

/// Builds one declared interface literal, for the family-recognition cases.
fn interface_of(inputs: &[(&str, &[u64])], output: (&str, u64)) -> DeclaredInterface {
    DeclaredInterface {
        inputs: inputs
            .iter()
            .map(|(key, extents)| DeclaredInput {
                key: (*key).to_owned(),
                elements: extents.iter().product(),
                extents: extents.to_vec(),
            })
            .collect(),
        output_key: output.0.to_owned(),
        output_elements: output.1,
        abi: tiler_artifact::program::AbiFactBinder::new(
            tiler_artifact::program::AvailabilityPhase::LiveDevicePreflight,
        )
        .build(),
    }
}

/// This half of the member filename interface, pinned.
///
/// `prototypes/serial-sum-compile` derives the identical names and carries the
/// identical assertion. The two crates share no code, so this pair of tests is
/// the only thing that compares their idea of the matrix — both the names and
/// which classes exist. A producer that adds a class this consumer does not open,
/// or renames one it does, fails here.
#[test]
fn the_member_names_are_the_ones_the_producer_writes() {
    let base = Path::new("/tmp/a.tiler");
    let names: Vec<String> = REDUCTION_CLASSES
        .iter()
        .flat_map(|(class, _)| {
            PLAN_ROLES
                .iter()
                .map(move |role| proof_member(base, class, role))
        })
        .map(|path| path.display().to_string())
        .collect();
    assert_eq!(
        names,
        [
            "/tmp/a.tiler.empty-domain.selected",
            "/tmp/a.tiler.empty-domain.materialized",
            "/tmp/a.tiler.singleton.selected",
            "/tmp/a.tiler.singleton.materialized",
            "/tmp/a.tiler.nontrivial.selected",
            "/tmp/a.tiler.nontrivial.materialized",
        ],
    );
    assert_eq!(SIDECAR_SUFFIX, ".proof");

    // The reduced extents the producer publishes, and the one this crate's own
    // direct path uses, agree — which is what makes the `nontrivial` class the
    // shape `crate::serial_sum` reduces directly.
    assert_eq!(
        REDUCTION_CLASSES,
        [
            ("empty-domain", 0),
            ("singleton", 1),
            ("nontrivial", COLUMNS)
        ],
        "`prototypes/serial-sum-compile` states this same matrix; a class or reduced extent \
         changed there must change here too",
    );
}

/// Each role means a distinct dispatch shape, and that is the whole matrix.
///
/// If both roles expected the same shape, the matrix would compare a program
/// against itself and report agreement, which is true and worthless.
#[test]
fn the_two_roles_mean_different_dispatch_shapes() {
    assert_eq!(expected_shape("selected"), (1, 0));
    assert_eq!(expected_shape("materialized"), (2, 1));
    assert_ne!(
        expected_shape("selected"),
        expected_shape("materialized"),
        "a fused plan and a materialized plan agreeing is only evidence if they ran differently",
    );
}

/// This half of the published contraction and L3 cell interfaces, pinned.
#[test]
fn the_published_contraction_members_are_the_ones_the_producer_writes() {
    assert_eq!(CONTRACTION_CLASS, "contraction");
    assert_eq!(FIXTURE_CONTRACTION, (2, 2, 3));
    assert_eq!(
        (
            CONTRACTION_ACTIVATIONS_KEY,
            CONTRACTION_WEIGHTS_KEY,
            CONTRACTION_OUTPUT_KEY,
        ),
        ("activations", "weights", "projected"),
    );
    assert_eq!(
        proof_member(Path::new("/tmp/a.tiler"), CONTRACTION_CLASS, "selected")
            .display()
            .to_string(),
        "/tmp/a.tiler.contraction.selected",
    );

    assert_eq!(L3_CELL_CLASS, "contraction-w-decode-kv");
    assert_eq!(FIXTURE_L3_CELL, (1, 1024, 1024));
    assert_eq!(
        proof_member(Path::new("/tmp/a.tiler"), L3_CELL_CLASS, "selected")
            .display()
            .to_string(),
        "/tmp/a.tiler.contraction-w-decode-kv.selected",
    );
    assert_eq!(
        L3_CELL_RESULT_SHA256, "79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f",
        "the retained `w_decode_kv` `direct` digest, from this spike's own record: \
         spikes/scheduling/metal_contraction_vertical/results/\
         2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/workload.tsv",
    );
}

/// Exactly one routed member carries a retained measurement, and it is the L3
/// cell.
///
/// **The negative half is the load-bearing one.** A `Some(...)` added to the
/// adversarial member would compare its executed bytes against a digest no device
/// ever produced for those operands, and that comparison would fail on hardware
/// in a way that reads as a device defect. Pinned so the distinction between
/// "measured elsewhere" and "not measured at all" cannot be lost by editing a
/// table.
#[test]
fn only_the_l3_cell_is_compared_against_a_retained_measurement() {
    let compared: Vec<&str> = CONTRACTION_MEMBERS
        .iter()
        .filter(|member| member.retained_result_sha256.is_some())
        .map(|member| member.class)
        .collect();
    assert_eq!(compared, [L3_CELL_CLASS]);
    assert_eq!(
        CONTRACTION_MEMBERS.len(),
        2,
        "both published contraction members must be routed; a member the producer writes and this \
         half never opens is a file nobody reads",
    );
    assert_eq!(CONTRACTION_MEMBERS[0].class, CONTRACTION_CLASS);
}

/// The digest helper reproduces the published FIPS 180-4 vectors, and its domain
/// is the probe's.
///
/// **Both halves, because either alone is satisfiable by a wrong function.** The
/// vectors say this is SHA-256. The domain case says the bytes it is fed are the
/// ones the probe hashed — little-endian `f32`, row-major — which a correct
/// SHA-256 over big-endian words would fail while passing every vector.
#[test]
fn the_digest_helper_reproduces_the_published_vectors() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    // `1.0f32` is `0x3f800000`, whose little-endian bytes are `00 00 80 3f`.
    // Digesting the big-endian spelling instead would be a different message, and
    // this is the assertion that says which one.
    assert_eq!(
        result_digest(&[0x3f80_0000]),
        sha256_hex(&[0x00, 0x00, 0x80, 0x3f]),
    );
    assert_ne!(
        result_digest(&[0x3f80_0000]),
        sha256_hex(&[0x3f, 0x80, 0x00, 0x00]),
    );
    // Row-major order is element order: two results that differ only in the order
    // of their elements are different messages.
    assert_ne!(
        result_digest(&[0x3f80_0000, 0x4000_0000]),
        result_digest(&[0x4000_0000, 0x3f80_0000]),
    );
}

/// The retained comparison reports its two verdicts independently, and is
/// watched refusing before it is trusted.
///
/// **This is the perturbation the L3 cell's retained-digest comparison is held to
/// before any device result rests on it.** The pairing is what makes a mismatch
/// diagnosable, so it is asserted here rather than left to the mismatch that would
/// exercise it — which needs hardware and a defect at once.
#[test]
fn a_retained_comparison_separates_the_executed_bytes_from_the_published_record() {
    let matching = RetainedComparison {
        executed: L3_CELL_RESULT_SHA256.to_owned(),
        embedded: L3_CELL_RESULT_SHA256.to_owned(),
        retained: L3_CELL_RESULT_SHA256,
    };
    assert!(matching.executed_matches() && matching.embedded_matches());

    // The device disagreed with a record that asks the right question.
    let device_wrong = RetainedComparison {
        executed: sha256_hex(b"another result"),
        embedded: L3_CELL_RESULT_SHA256.to_owned(),
        retained: L3_CELL_RESULT_SHA256,
    };
    assert!(!device_wrong.executed_matches() && device_wrong.embedded_matches());

    // The record asks a different question, and the device answered it faithfully
    // — the case that must not read as a device defect.
    let record_wrong = RetainedComparison {
        executed: sha256_hex(b"another workload"),
        embedded: sha256_hex(b"another workload"),
        retained: L3_CELL_RESULT_SHA256,
    };
    assert!(!record_wrong.executed_matches() && !record_wrong.embedded_matches());

    // And a single flipped element in the executed bytes is refused, which is the
    // perturbation that makes the positive comparison worth anything: the digest
    // domain is over the whole result, so no partial agreement survives it.
    let one_element_off = RetainedComparison {
        executed: result_digest(&[0x3f80_0000, 0x4000_0000]),
        embedded: result_digest(&[0x3f80_0000, 0x4000_0001]),
        retained: L3_CELL_RESULT_SHA256,
    };
    assert!(!one_element_off.executed_matches());
    assert!(!one_element_off.embedded_matches());
    assert_ne!(one_element_off.executed, one_element_off.embedded);
}

/// The contraction interface is recognized, and every way of not being one is
/// refused.
///
/// **The negatives are the point.** A recognizer that only ever saw the artifact
/// its own producer writes would accept anything, and each row below is a way an
/// interface could be wrong that would otherwise reach the device: a missing
/// operand binds one buffer for a two-operand kernel, a contracted-extent
/// disagreement sizes one operand against the other's `K`, swapped keys write each
/// operand into the other's buffer, and a wrong output count reads back the wrong
/// number of elements.
#[test]
fn the_contraction_interface_is_recognized_and_every_miss_is_refused() {
    let good = interface_of(
        &[("activations", &[2, 3]), ("weights", &[2, 3])],
        ("projected", 4),
    );
    assert_eq!(
        require_contraction_interface(&good).expect("the published interface is recognized"),
        FIXTURE_CONTRACTION,
    );

    let misses: [(&str, DeclaredInterface); 5] = [
        (
            "one operand where the contraction declares two",
            interface_of(&[("activations", &[2, 3])], ("projected", 4)),
        ),
        (
            "operands that contract over different extents",
            interface_of(
                &[("activations", &[2, 3]), ("weights", &[2, 5])],
                ("projected", 4),
            ),
        ),
        (
            "operands under keys the contraction does not declare",
            interface_of(
                &[("weights", &[2, 3]), ("activations", &[2, 3])],
                ("projected", 4),
            ),
        ),
        (
            "an output element count that is not M times N",
            interface_of(
                &[("activations", &[2, 3]), ("weights", &[2, 3])],
                ("projected", 6),
            ),
        ),
        (
            "a rank-3 operand",
            interface_of(
                &[("activations", &[2, 3, 1]), ("weights", &[2, 3])],
                ("projected", 4),
            ),
        ),
    ];
    for (miss, interface) in misses {
        let refusal = require_contraction_interface(&interface)
            .expect_err(&format!("{miss} must be refused"));
        assert!(
            matches!(refusal, EnvelopeFailure::Interface(_)),
            "{miss} was refused as {refusal} rather than as an interface disagreement",
        );
    }

    // And the serial sum's own interface is not mistaken for a contraction, which
    // is what keeps the two families' recognizers separate rather than one
    // accepting the other's artifacts.
    require_contraction_interface(&interface_of(&[("input", &[1, 3])], ("result", 1)))
        .expect_err("a one-input reduction is not a contraction");
    require_serial_sum_interface(&good).expect_err("a contraction is not a serial sum");
}

/// The serial sum's interface is recognized, and every way of not being one is
/// refused.
#[test]
fn the_serial_sum_interface_is_recognized_and_every_miss_is_refused() {
    let good = interface_of(&[("input", &[4, 3])], ("result", 4));
    assert_eq!(
        require_serial_sum_interface(&good).expect("the published interface is recognized"),
        (4, 3),
    );

    let misses: [(&str, DeclaredInterface); 4] = [
        (
            "two operands where the reduction declares one",
            interface_of(&[("input", &[4, 3]), ("other", &[4, 3])], ("result", 4)),
        ),
        (
            "a rank-1 operand",
            interface_of(&[("input", &[12])], ("result", 4)),
        ),
        (
            "an operand under a key the reduction does not declare",
            interface_of(&[("operand", &[4, 3])], ("result", 4)),
        ),
        (
            "an output element count that is not the row count",
            interface_of(&[("input", &[4, 3])], ("result", 12)),
        ),
    ];
    for (miss, interface) in misses {
        let refusal =
            require_serial_sum_interface(&interface).expect_err(&format!("{miss} must be refused"));
        assert!(
            matches!(refusal, EnvelopeFailure::Interface(_)),
            "{miss} was refused as {refusal} rather than as an interface disagreement",
        );
    }
}

/// A payload that is not exactly the declared element count is refused as a
/// sidecar defect, not carried into the numerical comparison.
///
/// The three lengths are the three ways a record can disagree with the interface
/// it names, and the middle one is why this is a length check rather than a chunk
/// count: a payload one byte short of two elements has a whole first element, so
/// truncating to whole chunks would decode it and report the missing element as a
/// device disagreement.
#[test]
fn a_payload_that_is_not_the_declared_length_is_a_sidecar_defect() {
    assert_eq!(
        decode_f32_bits("input", 2, &[0, 0, 0, 1, 0, 0, 0, 2]).expect("the exact length decodes"),
        vec![1, 2],
    );
    for bytes in [
        &[0, 0, 0, 1, 0, 0, 0][..],       // one byte short of two elements
        &[0, 0, 0, 1][..],                // one element where two are declared
        &[0, 0, 0, 1, 0, 0, 0, 2, 0][..], // two elements and a trailing byte
    ] {
        let refusal = decode_f32_bits("input", 2, bytes)
            .expect_err("a payload of the wrong length is refused");
        assert!(
            matches!(refusal, EnvelopeFailure::SidecarShapeMismatch { .. }),
            "a malformed record must not be reported as arithmetic: {refusal}",
        );
    }
}

/// The routed environment's dtype rows are the declaration's own.
///
/// **What this pins is where the rows come from, and it is observable only under
/// a ledger that moved.** A transcribed literal and a read of
/// `dtype_dispatchability_rows` agree exactly while the declaration states what it
/// states today, so no assertion over today's *values* could tell the two apart.
/// Deriving the expectation from the declaration is what separates them: a
/// widened, narrowed, or retracted measurement moves both sides of this comparison
/// together and would leave a literal stating a verdict the profile stopped
/// holding.
///
/// The remaining assertions cover what that equality cannot. The verdict
/// translation is checked in both arms, because a [`host_dtype_dispatch`] that
/// answered `Dispatchable` for a refutation would satisfy the comparison above on
/// today's ledger — which refutes nothing — while inverting the profile's answer
/// on one that does. And `f16`, the dtype this ledger deliberately does not
/// measure, must be **absent** rather than present with a permissive verdict: that
/// is the fail-closed half, and it is asserted through `classify_dtype` because
/// resolving a missing key as `Unknown` is what the loader acts on.
#[test]
fn the_routed_dtype_rows_are_the_declarations_own() {
    let declaration = declaration().expect("the authoritative declaration assembles");
    let environment =
        declared_route_environment(&declaration).expect("the declared environment composes");
    let from_declaration: std::collections::BTreeMap<_, _> = declaration
        .dtype_dispatchability_rows()
        .into_iter()
        .map(|(arithmetic, verdict)| (arithmetic, host_dtype_dispatch(verdict)))
        .collect();
    assert_eq!(
        *dtype_rows(&environment),
        from_declaration,
        "the routed rows must be the declaration's own, so a moved ledger row moves this \
         environment rather than leaving a stale transcription beside it",
    );
    assert!(
        !from_declaration.is_empty(),
        "an empty row set would satisfy the comparison above while stating nothing",
    );

    assert_eq!(
        host_dtype_dispatch(DTypeDispatchability::Dispatchable),
        DTypeDispatch::Dispatchable,
    );
    assert_eq!(
        host_dtype_dispatch(DTypeDispatchability::Unsupported),
        DTypeDispatch::Unsupported,
        "a refuted measurement must reach the host as a stated negative rather than as permission",
    );

    assert_eq!(
        environment.classify_dtype(ArithmeticType::F16),
        DTypeDispatchResolution::Unknown,
        "a dtype this ledger does not measure earns no row, and the silence refuses",
    );
}

/// The unavailable path names the producer command rather than skipping.
///
/// **The reporting obligation, exercised rather than described.** A run without a
/// published base must be distinguishable from one that routed, and what makes it
/// distinguishable is that the reason names the exact command that produces the
/// missing input. Asserting the sentence is what stops a later edit from
/// shortening it into "not available".
#[test]
fn an_absent_published_base_names_the_producer_command() {
    let super::PublishedBase::Absent(reason) = super::published_base() else {
        // A run *with* a base supplied is the other half of the same rule, and
        // this case states it rather than failing: what must never happen is a
        // third outcome in which neither is reported.
        eprintln!("{ARTIFACT_BASE} is supplied; the routed runs below carry the boundary instead");
        return;
    };
    assert!(
        reason.contains(ARTIFACT_BASE),
        "the reason must name the ambient input that was missing: {reason}",
    );
    assert!(
        reason.contains(PRODUCER_COMMAND),
        "the reason must name the producer command that creates one: {reason}",
    );
}

/// The published matrix agrees with its own record, member by member and case by
/// case.
///
/// Each reduction class is routed twice — once as the fused single-dispatch plan
/// the optimizer selects, once as the materialized plan that computes the same
/// function through two dispatches and one intermediate — over every operand class
/// the producer published. Agreement between them is a statement about the
/// optimizer; agreement with the sidecar's expected bytes is a statement about
/// both.
#[test]
fn the_published_matrix_agrees_with_its_record_on_the_routed_row() {
    let Some(base) = require_or_report_base("envelope matrix") else {
        return;
    };
    let Some(members) = require_or_report("envelope matrix", measured_matrix(&base)) else {
        return;
    };

    assert_eq!(
        members.len(),
        REDUCTION_CLASSES.len() * PLAN_ROLES.len(),
        "every published member must be routed, not the ones that happened to open",
    );
    let mut fused = 0_usize;
    let mut materialized = 0_usize;
    for member in &members {
        assert!(member.proved > 0, "{} routed no operand case", member.name);
        if (member.entries, member.shared) == expected_shape("selected") {
            fused += 1;
        } else {
            assert_eq!(
                (member.entries, member.shared),
                expected_shape("materialized")
            );
            materialized += 1;
        }
    }
    assert_eq!(
        (fused, materialized),
        (REDUCTION_CLASSES.len(), REDUCTION_CLASSES.len()),
        "the two roles must not converge on one shape, or their agreement is one program agreeing \
         with itself",
    );
}

/// Every published contraction member routes, and the L3 cell's executed bytes
/// carry the retained realization-probe digest.
///
/// The only comparison in this crate that reaches outside the workspace's own two
/// implementations. It is bounded to the host row the measurement was taken on;
/// a reader who is not on that row must treat it as unmade rather than as
/// evidence.
#[test]
fn the_contraction_members_route_and_the_l3_cell_carries_its_retained_digest() {
    let Some(base) = require_or_report_base("envelope contraction") else {
        return;
    };
    let mut compared = 0_usize;
    for member in &CONTRACTION_MEMBERS {
        let Some(routed) = require_or_report(
            &format!("envelope contraction {}", member.class),
            measured_contraction(&base, member),
        ) else {
            return;
        };
        assert!(routed.proved > 0, "{} routed no operand case", routed.name);
        match (&routed.retained, member.retained_result_sha256) {
            (Some(comparison), Some(retained)) => {
                assert!(
                    comparison.executed_matches(),
                    "{}: the executed bytes hash to {} and the retained measurement is {retained}",
                    routed.name,
                    comparison.executed,
                );
                assert!(
                    comparison.embedded_matches(),
                    "{}: the producer's published expectation hashes to {}, so the fixture asks a \
                     different question than the retained measurement answered",
                    routed.name,
                    comparison.embedded,
                );
                compared += 1;
            }
            (None, None) => {}
            (carried, declared) => panic!(
                "{}: the member declares {declared:?} and the run carried {carried:?}",
                routed.name,
            ),
        }
    }
    assert_eq!(
        compared, 1,
        "exactly one routed member is compared against a retained measurement",
    );
}
