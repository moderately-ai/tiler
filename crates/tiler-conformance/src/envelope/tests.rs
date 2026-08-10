//! The envelope route's runs, in two halves that fail for unrelated reasons.
//!
//! Everything above [`the_published_matrix_agrees_with_its_record_on_the_routed_row`]
//! is decidable without an envelope and runs on every host in the ordinary gate:
//! the interface recognizers and every way of missing them, the sidecar
//! payload-length refusal, the routed dtype rows, the member and class pins, the
//! digest helper against its published vectors and its own domain, the retained
//! comparison's two verdicts including the perturbation it must refuse, the six
//! correctness cells against the retained record's own `direct` rows, the
//! derivation of which cells a sidecar can carry, and the gate split.
//!
//! The routed runs publish their own members through [`crate::publication`] and
//! then route them, so on a host with an offline Apple toolchain and a device
//! they run under the ordinary gate. A host without either reports the
//! measurement boundary unavailable — one boundary rather than two, because
//! publishing and routing need the same environment — and never skips.
//!
//! **There are two routed contraction runs and the second is `#[ignore]`d.** The
//! ordinary one routes the adversarial member and `w_decode_kv`; the four L3
//! prefill cells are in [`the_prefill_cells_carry_their_retained_digests`],
//! because publishing them costs 1,094,713,344 reference fold steps under a
//! stated allowance. Which run a member belongs to is derived from the bounds
//! rather than listed — see
//! [`the_gate_routes_the_cells_the_default_reference_evaluator_admits`].

use std::path::Path;

use tiler_artifact::program::ArithmeticType;
use tiler_build::DTypeDispatchability;
use tiler_runtime::load::{DTypeDispatch, DTypeDispatchResolution};

use super::{
    CONTRACTION_ACTIVATIONS_KEY, CONTRACTION_CLASS, CONTRACTION_MEMBERS, CONTRACTION_OUTPUT_KEY,
    CONTRACTION_WEIGHTS_KEY, ContractionMember, DeclaredInput, DeclaredInterface, EnvelopeFailure,
    L3_CORRECTNESS_CELLS, L3CorrectnessCell, PLAN_ROLES, REDUCTION_CLASSES, RetainedComparison,
    SIDECAR_SUFFIX, declared_route_environment, decode_f32_bits, dtype_rows, expected_shape,
    host_dtype_dispatch, measured_contraction, measured_matrix, proof_member,
    require_contraction_interface, require_serial_sum_interface, result_digest, sha256_hex,
    sidecar_path,
};
use crate::ledger::{CompositionExtent, f32_subject, validate_fresh_f32_matrix};
use crate::measurement::{require_or_report, require_or_report_with_boundary};
use crate::serial_sum::{COLUMNS, declaration};

/// The extents the published contraction member is compiled and routed at.
///
/// **A literal rather than a read of [`CONTRACTION_MEMBERS`], and the redundancy
/// is the point.** These three numbers decide which operand table
/// `crate::publication::proof` may use, and a test that recomputed them from the
/// constant it is guarding would agree with any change. What it stops is a
/// silent move: `2 x 2` is the only published contraction shape whose result has
/// more than one row *and* more than one column, so repointing this member at a
/// `1 x N` cell would leave a kernel that confused the two operand access
/// relations still agreeing.
const FIXTURE_CONTRACTION: (u64, u64, u64) = (2, 2, 3);

/// Every L3 correctness cell, as `(class, extents, retained digest)`.
///
/// **Literals for the same reason [`FIXTURE_CONTRACTION`] is, and one stronger: a
/// `result_sha256` was retained for each of these cells at exactly these
/// extents**, so a cell published at any other shape would be compared against a
/// digest that never described it, and a class pointing at another cell's digest
/// would route and disagree in a way that reads as a device defect.
///
/// This is a second statement of `super::L3_CORRECTNESS_CELLS` on purpose. It is
/// the *first* statement that is checked against the retained record's own
/// `workload.tsv` by
/// [`the_pinned_cells_are_the_retained_records_own_direct_rows`]; this one is
/// checked against that, so a table edited in one place fails here and a table
/// edited in both fails against the record.
const FIXTURE_L3_CELLS: [(&str, (u64, u64, u64), &str); 6] = [
    (
        "contraction-w-decode-kv",
        (1, 1024, 1024),
        "79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f",
    ),
    (
        "contraction-w-prefill-q",
        (10, 2048, 1024),
        "1c54f5cd7265ee288ec79bcd9254243b78a95d57c3c489e5ea90bcc4298073c0",
    ),
    (
        "contraction-w-prefill-mlp-in",
        (128, 3072, 1024),
        "eb382840ac9e533f57e51a0ffed2d61608664ecc5869aaa9f93afa3c312696a0",
    ),
    (
        "contraction-w-prefill-mlp-out",
        (128, 1024, 3072),
        "124571de47ebff2f152b120afc9944b3465bffe94d8ac283a077677f61feb5f5",
    ),
    (
        "contraction-w-prefill-o",
        (128, 1024, 2048),
        "b99eff9042d9e4b25e3844ff0462e5e6303e57b146aa79400622885bffc5f2f6",
    ),
    (
        "contraction-w-vocab-slice",
        (1, 8192, 1024),
        "88b01ae776f42bdb2f2d1092ddfd039e20e652d28393a6e2ec19e5cc1d9803c8",
    ),
];

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

/// Every member of the matrix has a distinct name, and the derivation is pinned.
///
/// **What this catches is different now that one derivation serves both halves.**
/// It used to be one side of a pinned pair against a separate executable, which
/// is how a rename could break the slice while both halves compiled; the
/// publication and the route reach [`proof_member`] and [`sidecar_path`] now, so
/// a rename moves both together. What a shared derivation cannot protect is the
/// derivation *itself* — a base that appended nothing, or a class that collided
/// with another, would leave members silently overwriting each other and the
/// matrix routing six names that were four files. The names are asserted as
/// literals for that, and the reduced extents beside them because the
/// `nontrivial` class exists to publish the contributor count
/// `crate::serial_sum` reduces directly.
#[test]
fn every_member_name_is_distinct_and_derived_the_one_way() {
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
    let mut distinct = names.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        names.len(),
        "two members sharing a path would leave one silently overwriting the other",
    );

    assert_eq!(SIDECAR_SUFFIX, ".proof");
    assert_eq!(
        sidecar_path(Path::new(&names[0])).display().to_string(),
        "/tmp/a.tiler.empty-domain.selected.proof",
        "the record is named beside its envelope by appending, so the pair stays one unit on disk",
    );

    assert_eq!(
        REDUCTION_CLASSES,
        [
            ("empty-domain", 0),
            ("singleton", 1),
            ("nontrivial", COLUMNS)
        ],
        "the nontrivial class publishes the contributor count `crate::serial_sum` reduces",
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

/// The published contraction and L3 cell interfaces, pinned.
///
/// The extents are additionally compared against the ones each member declares,
/// which is what makes the two literals above a check on
/// [`CONTRACTION_MEMBERS`] rather than a restatement of nothing.
#[test]
fn the_published_contraction_members_are_the_ones_this_module_routes() {
    let mut expected = vec![Some(FIXTURE_CONTRACTION)];
    expected.extend(
        FIXTURE_L3_CELLS
            .iter()
            .filter(|(class, _, _)| *class != UNPUBLISHABLE_CELL_CLASS)
            .map(|(_, extents, _)| Some(*extents)),
    );
    assert_eq!(
        CONTRACTION_MEMBERS
            .iter()
            .map(|member| member.family.contraction_extents())
            .collect::<Vec<_>>(),
        expected,
        "a member published at extents this file does not pin would route a shape no assertion \
         here describes",
    );

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

    // Every cell's class, extents, and digest against this file's own literals,
    // and the member path each derives — six classes that collided would leave
    // members silently overwriting each other in one publication directory.
    assert_eq!(L3_CORRECTNESS_CELLS.len(), FIXTURE_L3_CELLS.len());
    let mut paths = Vec::new();
    for (cell, (class, extents, digest)) in L3_CORRECTNESS_CELLS.iter().zip(FIXTURE_L3_CELLS) {
        assert_eq!(cell.class, class, "{}: its class moved", cell.id);
        assert_eq!(cell.extents(), extents, "{}: its extents moved", cell.id);
        assert_eq!(
            cell.result_sha256, digest,
            "{}: its retained digest moved; the record is \
             spikes/scheduling/metal_contraction_vertical/results/\
             2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/workload.tsv",
            cell.id,
        );
        paths.push(
            proof_member(Path::new("/tmp/a.tiler"), cell.class, "selected")
                .display()
                .to_string(),
        );
    }
    assert_eq!(paths[0], "/tmp/a.tiler.contraction-w-decode-kv.selected");
    let mut distinct = paths.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        paths.len(),
        "two cells sharing a member path would leave one silently overwriting the other",
    );
}

/// Every routed member except the adversarial one carries a retained
/// measurement.
///
/// **The negative half is the load-bearing one.** A `Some(...)` added to the
/// adversarial member would compare its executed bytes against a digest no device
/// ever produced for those operands, and that comparison would fail on hardware
/// in a way that reads as a device defect. Pinned so the distinction between
/// "measured elsewhere" and "not measured at all" cannot be lost by editing a
/// table.
#[test]
fn every_member_but_the_adversarial_one_is_compared_against_a_retained_measurement() {
    let compared: Vec<&str> = CONTRACTION_MEMBERS
        .iter()
        .filter(|member| member.retained_result_sha256.is_some())
        .map(|member| member.class)
        .collect();
    assert_eq!(
        compared,
        FIXTURE_L3_CELLS
            .iter()
            .map(|(class, _, _)| *class)
            .filter(|class| *class != UNPUBLISHABLE_CELL_CLASS)
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        CONTRACTION_MEMBERS.len(),
        6,
        "every publishable contraction member must be routed; a member the producer writes and \
         this half never opens is a file nobody reads",
    );
    assert_eq!(CONTRACTION_MEMBERS[0].class, CONTRACTION_CLASS);
    assert!(
        CONTRACTION_MEMBERS[0].retained_result_sha256.is_none(),
        "the adversarial member's five operand classes were never measured on a device, so there \
         is nothing to compare its executed bytes against beyond its published reference",
    );

    // Each member's digest is its own cell's, matched through the class rather
    // than through the position: a table reordered without moving the digests
    // would keep every assertion above green and route the members against
    // wrong measurements.
    for (class, _, digest) in FIXTURE_L3_CELLS {
        if class == UNPUBLISHABLE_CELL_CLASS {
            continue;
        }
        let member = CONTRACTION_MEMBERS
            .iter()
            .find(|member| member.class == class)
            .unwrap_or_else(|| panic!("{class} is routed"));
        assert_eq!(member.retained_result_sha256, Some(digest));
    }
}

/// The one correctness cell no proof sidecar can carry.
///
/// Named once here, so the three tests that have to exclude it and the one that
/// explains why it is excluded all say the same word.
const UNPUBLISHABLE_CELL_CLASS: &str = "contraction-w-vocab-slice";

/// The routed members are exactly the cells a sidecar can carry.
///
/// **`CONTRACTION_MEMBERS` is a `const` and cannot filter, so its exclusion of
/// `w_vocab_slice` is a hand-written index — and this is what stops that index
/// from being a place a cell can be dropped quietly.** The expected set is
/// derived from the predicate rather than listed, so a cell that stopped fitting
/// must be removed from the array and one that started fitting must be added,
/// and either omission fails here rather than on the host that publishes.
#[test]
fn the_routed_members_are_exactly_the_publishable_cells() {
    let routed: Vec<&str> = CONTRACTION_MEMBERS
        .iter()
        .filter(|member| member.retained_result_sha256.is_some())
        .map(|member| member.class)
        .collect();
    let publishable: Vec<&str> = L3_CORRECTNESS_CELLS
        .iter()
        .filter(|cell| cell.fits_one_proof_payload())
        .map(|cell| cell.class)
        .collect();
    assert_eq!(
        routed, publishable,
        "the routed members and the cells a sidecar can carry have drifted apart",
    );
    assert_eq!(routed.len(), 5);
    assert_eq!(
        L3_CORRECTNESS_CELLS
            .iter()
            .filter(|cell| !cell.fits_one_proof_payload())
            .map(|cell| cell.class)
            .collect::<Vec<_>>(),
        [UNPUBLISHABLE_CELL_CLASS],
    );
}

/// The excluded cell is named against the exact bound that stops it.
///
/// **This is the ticket's remaining cell, held to arithmetic instead of to
/// prose.** `w_vocab_slice`'s `[8192, 1024]` weights operand is 33,554,432 bytes
/// and `tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES` admits 16,777,216 — a
/// factor of exactly two, so no rounding or framing overhead is involved and the
/// exclusion cannot be argued away. Nothing inside
/// `implementation/conformance` reaches it: the constant is another crate's
/// public surface, and splitting the operand across cases would publish a
/// different program rather than the cell the digest describes.
///
/// The five routable cells are asserted with margin in the same test, so a bound
/// that *fell* takes the whole set down rather than silently dropping members.
#[test]
fn the_unpublishable_cell_is_named_against_the_bound_that_stops_it() {
    let limit = u64::try_from(tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES)
        .expect("the artifact layer's payload bound fits a u64");
    assert_eq!(limit, 16_777_216);

    let excluded = L3_CORRECTNESS_CELLS
        .iter()
        .find(|cell| cell.class == UNPUBLISHABLE_CELL_CLASS)
        .expect("the record's vocabulary cell is still pinned");
    assert_eq!(excluded.largest_payload_bytes(), 33_554_432);
    assert_eq!(
        excluded.largest_payload_bytes(),
        limit * 2,
        "the excluded cell's operand is exactly twice the bound; if that is no longer the \
         arithmetic, the exclusion needs restating rather than keeping",
    );
    assert!(!excluded.fits_one_proof_payload());

    for cell in &L3_CORRECTNESS_CELLS {
        if cell.class == UNPUBLISHABLE_CELL_CLASS {
            continue;
        }
        assert!(
            cell.fits_one_proof_payload(),
            "{}: its largest payload is {} byte(s) against a bound of {limit}",
            cell.id,
            cell.largest_payload_bytes(),
        );
    }
    assert_eq!(
        L3_CORRECTNESS_CELLS
            .iter()
            .map(L3CorrectnessCell::largest_payload_bytes)
            .max(),
        Some(33_554_432),
    );
}

/// The pinned cells are the retained record's own `direct` rows.
///
/// **Device-free, and the only check in this crate that reads the measurement's
/// source rather than a transcription of it.** Six extents and six 64-character
/// digests are exactly the kind of thing a careful reader transcribes wrongly
/// once and nobody notices, because every routed comparison then agrees with the
/// wrong constant on both sides — the published reference is computed from the
/// same extents the digest is compared at. The record is the independent
/// statement, and this is where the two meet.
///
/// The `direct` realization and no other: the record carries six realizations per
/// cell and four of them are permitted but differently-grouped answers, so a
/// member compared against `ksplit_strided`'s digest would report a wrong answer
/// for a right reason.
#[test]
fn the_pinned_cells_are_the_retained_records_own_direct_rows() {
    let record = crate::retained_record::direct_digests().expect("the retained record reads");
    assert_eq!(
        record.len(),
        L3_CORRECTNESS_CELLS.len(),
        "the record states {} `direct` row(s) and this crate pins {}",
        record.len(),
        L3_CORRECTNESS_CELLS.len(),
    );

    let mut checked = 0_usize;
    for cell in &L3_CORRECTNESS_CELLS {
        let row = record
            .iter()
            .find(|row| row.id == cell.id)
            .unwrap_or_else(|| panic!("the retained record states a `direct` row for {}", cell.id));
        assert_eq!(
            (row.m, row.n, row.k),
            cell.extents(),
            "{}: this crate publishes it at extents the record did not measure",
            cell.id,
        );
        assert_eq!(
            row.result_sha256, cell.result_sha256,
            "{}: the pinned digest is not the record's own",
            cell.id,
        );
        assert_eq!(
            cell.m * cell.n * cell.k,
            cell.fold_steps,
            "{}: the pinned fold is not the product of its extents",
            cell.id,
        );
        checked += 1;
    }
    assert_eq!(
        checked, 6,
        "the loop must have reached every pinned cell; a lookup that silently matched none would \
         leave this green over an empty comparison",
    );
}

/// The gate routes the cells the *default* reference evaluator admits, and the
/// `#[ignore]`d run the rest.
///
/// **The split the gate is drawn on, stated where a reader looks for it.**
/// `crate::publication::proof` needs the reference's expected bytes to publish a
/// cell, and a fold above the evaluator's per-occurrence bound is only reachable
/// by stating a larger number. So the ordinary gate publishes only cells under
/// that bound, through exactly the evaluator every other consumer gets, and
/// authorizes nothing.
///
/// **Two cells are under the bound and only one of them is routable at all.**
/// `w_vocab_slice` folds 8,388,608 steps, comfortably under it, and is stopped
/// one layer lower by the sidecar payload bound — see
/// [`the_unpublishable_cell_is_named_against_the_bound_that_stops_it`]. The two
/// bounds are independent and a cell must clear both, which is why the gate set
/// is derived from the conjunction rather than from either one.
#[test]
fn the_gate_routes_the_cells_the_default_reference_evaluator_admits() {
    let (under, over): (Vec<&L3CorrectnessCell>, Vec<&L3CorrectnessCell>) = L3_CORRECTNESS_CELLS
        .iter()
        .partition(|cell| cell.folds_under_the_default_allowance());
    assert_eq!(
        under.iter().map(|cell| cell.id).collect::<Vec<_>>(),
        ["w_decode_kv", "w_vocab_slice"],
    );
    assert_eq!(
        over.iter().map(|cell| cell.id).collect::<Vec<_>>(),
        [
            "w_prefill_q",
            "w_prefill_mlp_in",
            "w_prefill_mlp_out",
            "w_prefill_o",
        ],
    );
    assert_eq!(
        over.iter().map(|cell| cell.fold_steps).sum::<u64>(),
        1_094_713_344,
        "the four cells behind the ignore fold 1,094,713,344 steps between them",
    );

    assert_eq!(
        gate_members()
            .iter()
            .map(|member| member.class)
            .collect::<Vec<_>>(),
        [CONTRACTION_CLASS, "contraction-w-decode-kv"],
        "the ordinary gate routes the adversarial member and the one cell that is both under the \
         reference's bound and carryable by a sidecar",
    );
    assert_eq!(
        ignored_members()
            .iter()
            .map(|member| member.class)
            .collect::<Vec<_>>(),
        [
            "contraction-w-prefill-q",
            "contraction-w-prefill-mlp-in",
            "contraction-w-prefill-mlp-out",
            "contraction-w-prefill-o",
        ],
    );
    assert_eq!(
        gate_members().len() + ignored_members().len(),
        CONTRACTION_MEMBERS.len(),
        "every routed member belongs to exactly one of the two runs; a member in neither would be \
         published by nothing and noticed by nobody",
    );
}

/// The contraction members the ordinary gate routes.
///
/// Derived rather than listed, so a cell that moved across either bound moves
/// with it instead of being left behind in a hand-written set.
fn gate_members() -> Vec<&'static ContractionMember> {
    CONTRACTION_MEMBERS
        .iter()
        .filter(|member| {
            member.retained_result_sha256.is_none()
                || L3_CORRECTNESS_CELLS.iter().any(|cell| {
                    cell.class == member.class && cell.folds_under_the_default_allowance()
                })
        })
        .collect()
}

/// The contraction members only the `#[ignore]`d run routes.
///
/// The complement of [`gate_members`] over the same array rather than a second
/// filter, so the two sets partition the routed members by construction.
fn ignored_members() -> Vec<&'static ContractionMember> {
    let gated = gate_members();
    CONTRACTION_MEMBERS
        .iter()
        .filter(|member| !gated.iter().any(|other| other.class == member.class))
        .collect()
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
    let retained = L3_CORRECTNESS_CELLS[0].result_sha256;
    let matching = RetainedComparison {
        executed: retained.to_owned(),
        embedded: retained.to_owned(),
        retained,
    };
    assert!(matching.executed_matches() && matching.embedded_matches());

    // The device disagreed with a record that asks the right question.
    let device_wrong = RetainedComparison {
        executed: sha256_hex(b"another result"),
        embedded: retained.to_owned(),
        retained,
    };
    assert!(!device_wrong.executed_matches() && device_wrong.embedded_matches());

    // The record asks a different question, and the device answered it faithfully
    // — the case that must not read as a device defect.
    let record_wrong = RetainedComparison {
        executed: sha256_hex(b"another workload"),
        embedded: sha256_hex(b"another workload"),
        retained,
    };
    assert!(!record_wrong.executed_matches() && !record_wrong.embedded_matches());

    // And a single flipped element in the executed bytes is refused, which is the
    // perturbation that makes the positive comparison worth anything: the digest
    // domain is over the whole result, so no partial agreement survives it.
    let one_element_off = RetainedComparison {
        executed: result_digest(&[0x3f80_0000, 0x4000_0000]),
        embedded: result_digest(&[0x3f80_0000, 0x4000_0001]),
        retained,
    };
    assert!(!one_element_off.executed_matches());
    assert!(!one_element_off.embedded_matches());
    assert_ne!(one_element_off.executed, one_element_off.embedded);

    // Another cell's digest is not this one's, which is what makes a per-cell
    // comparison a per-cell claim rather than six members agreeing with one
    // number.
    let wrong_cell = RetainedComparison {
        executed: retained.to_owned(),
        embedded: retained.to_owned(),
        retained: L3_CORRECTNESS_CELLS[1].result_sha256,
    };
    assert!(!wrong_cell.executed_matches() && !wrong_cell.embedded_matches());
}

/// The contraction interface is recognized, and every way of not being one is
/// refused.
///
/// **The negatives are the point, and they carry more now than they used to.**
/// The routed runs only ever see the artifact this crate published, so nothing
/// they route can exercise a rejection; a recognizer with no negatives beside it
/// would accept anything and never be caught. Each row below is a way an
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

/// The published matrix agrees with its own record, member by member and case by
/// case.
///
/// Each reduction class is published twice — once as the fused single-dispatch
/// plan the optimizer selects, once as the materialized plan that computes the
/// same function through two dispatches and one intermediate — and each is routed
/// over every operand class its record carries. Agreement between them is a
/// statement about the optimizer; agreement with the record's expected bytes is a
/// statement about both.
#[test]
fn the_published_matrix_agrees_with_its_record_on_the_routed_row() {
    let subject = f32_subject().expect("the retained F32 execution subject resolves");
    assert_eq!(subject.composition(), CompositionExtent::RoutedArtifact);
    let Some((boundary, members)) =
        require_or_report_with_boundary("envelope matrix", measured_matrix())
    else {
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
    validate_fresh_f32_matrix(
        &subject,
        &boundary,
        members.iter().map(|member| member.proved).sum(),
    );
}

/// Routes a set of contraction members and returns how many were compared
/// against a retained measurement.
///
/// **One procedure for both runs below**, so the `#[ignore]`d whole-profile run
/// cannot drift into asserting less than the gate does. `None` means this host
/// reported the measurement boundary unavailable, which
/// [`require_or_report`] has already printed.
///
/// A member whose retained comparison was *declined* — this hardware is not the
/// row the digest was measured on — is counted separately and its reason is
/// required rather than accepted as an absence. Collapsing the two would make an
/// unmeasured machine indistinguishable from a compared one, which is the silent
/// skip the crate header refuses.
fn route_and_compare(members: &[&ContractionMember]) -> Option<(usize, usize)> {
    let _subject = f32_subject().expect("the retained F32 execution subject resolves");
    let (mut compared, mut declined) = (0_usize, 0_usize);
    for member in members {
        let routed = require_or_report(
            &format!("envelope contraction {}", member.class),
            measured_contraction(member),
        )?;
        assert!(routed.proved > 0, "{} routed no operand case", routed.name);
        match (&routed.retained, member.retained_result_sha256) {
            (Some(comparison), Some(retained)) => {
                assert!(
                    routed.retained_declined.is_none(),
                    "{}: a comparison was made and a reason for declining one was reported",
                    routed.name,
                );
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
            (None, Some(retained)) => {
                let reason = routed.retained_declined.as_ref().unwrap_or_else(|| {
                    panic!(
                        "{}: the member declares the retained digest {retained} and the run made \
                         no comparison and gave no reason, which is the silent skip this crate \
                         refuses",
                        routed.name,
                    )
                });
                eprintln!("{}: retained comparison declined — {reason}", routed.name);
                declined += 1;
            }
            (None, None) => assert!(
                routed.retained_declined.is_none(),
                "{}: a member with nothing to compare reported a reason for not comparing",
                routed.name,
            ),
            (carried, declared) => panic!(
                "{}: the member declares {declared:?} and the run carried {carried:?}",
                routed.name,
            ),
        }
    }
    Some((compared, declined))
}

/// The gate's contraction members route, and each cell's executed bytes carry its
/// retained realization-probe digest.
///
/// The only comparison in this crate that reaches outside the workspace's own two
/// implementations. It is bounded to the host row the measurement was taken on,
/// which `crate::retained_record` compares field by field and prints before any
/// digest is compared; a difference in the *machine* declines the comparison by
/// name rather than making it anyway.
///
/// **Two of the six routed members, and the line is a property rather than a
/// budget** — see [`the_gate_routes_the_cells_the_default_reference_evaluator_admits`].
/// The other four are in the `#[ignore]`d run below.
#[test]
fn the_contraction_members_route_and_the_gates_cells_carry_their_retained_digests() {
    let members = gate_members();
    let Some((compared, declined)) = route_and_compare(&members) else {
        return;
    };
    assert_eq!(
        compared + declined,
        1,
        "the gate routes one cell carrying a retained measurement, and it must either be compared \
         or have said why it was not",
    );
}

/// The four prefill cells' executed bytes carry their retained digests.
///
/// `#[ignore]`d for cost, not for doubt, on exactly the terms
/// `crates/tiler-reference/tests/contraction_profile_cells.rs` uses for the same
/// cells: publishing them needs the reference oracle to fold 1,094,713,344
/// multiply-accumulate steps under a stated allowance, and the largest operand
/// stream is a twelve-megabyte record written and read back per member.
///
/// **Measurement — Apple M4 Max, macOS 27.0 `26A5388g`, Apple9, metal
/// `32023.921`, SDK macosx 27.0 `26A5388f`, test profile (`opt-level` 1 with
/// debuginfo).** All four reproduce their retained `direct` digests. The
/// invocation, and the run this was landed with:
///
/// ```sh
/// cargo nextest run -p tiler-conformance --run-ignored only --no-capture \
///     -E 'test(the_prefill_cells_carry_their_retained_digests)'
/// ```
///
/// The gate keeps more than the `#[ignore]` costs it: the route, the placement,
/// the fail-closed probes, the interface recognizers, the digest domain, the row
/// comparison, and `w_decode_kv` all run on every gate run, and every cell shares
/// one emitted kernel — so an arithmetic change that moved these four would move
/// the one the gate routes.
#[test]
#[ignore = "1.09e9 reference fold steps under a stated allowance, across four published members; run deliberately, see this test's documentation"]
fn the_prefill_cells_carry_their_retained_digests() {
    let members = ignored_members();
    assert_eq!(members.len(), 4, "the four prefill cells are routed here");
    let Some((compared, declined)) = route_and_compare(&members) else {
        return;
    };
    assert_eq!(
        compared + declined,
        4,
        "all four cells carry a retained measurement, and each must either be compared or have \
         said why it was not",
    );
}
