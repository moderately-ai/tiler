//! The BF16 vertical's runs, in two halves that fail for unrelated reasons.
//!
//! Everything above [`the_bf16_vertical_agrees_with_the_oracle_on_the_measured_row`]
//! is deterministic and runs on every host: the corpus and its hand-derived
//! encodings, the oracle under both readings, the elements the declared flush
//! moves, the region and its lowering, `bfloat` emission, the refusal a strict
//! contract meets on the measured row, and the composition perturbation's
//! host-visible half. The measured runs report their boundary — or its absence
//! — rather than skipping.

use std::path::Path;

use tiler_compiler::session::NumericalContract;
use tiler_ir::kernel::{KernelType, lower_scheduled_region};
use tiler_ir::program::StorageScalar;
use tiler_metal::emit::emit_translation_unit;
use tiler_reference::ReferenceNumericalConformance;

use super::{
    Case, OperandStride, SCALE_BITS, corpus, declared_conformance, declared_contract,
    declared_expectations, declared_realization, emit_vertical, flush_moved_indices, operands,
    pack, preserved_expectations, realization_of, reference_bits, region_under, unpack,
};
use crate::measurement::{measured_half, require_or_report};

/// Corpus positions this file names, so a reordering is a failure rather than a
/// silently different claim.
mod position {
    /// Positive zero.
    pub(super) const POSITIVE_ZERO: usize = 0;
    /// Negative zero — the add's execution witness.
    pub(super) const NEGATIVE_ZERO: usize = 1;
    /// The least positive subnormal.
    pub(super) const LEAST_POSITIVE_SUBNORMAL: usize = 2;
    /// The least negative subnormal.
    pub(super) const LEAST_NEGATIVE_SUBNORMAL: usize = 3;
    /// Half the least normal — finding 24's measured input-flush operand.
    pub(super) const HALF_MIN_NORMAL: usize = 4;
    /// Its negation — finding 24's measured sign row.
    pub(super) const NEG_HALF_MIN_NORMAL: usize = 5;
    /// The greatest subnormal.
    pub(super) const GREATEST_SUBNORMAL: usize = 6;
    /// The least normal, which the flush does not move.
    pub(super) const LEAST_NORMAL: usize = 7;
    /// One — the multiply's execution witness.
    pub(super) const ONE: usize = 8;
    /// A tie resolved to the even significand.
    pub(super) const TIE: usize = 9;
    /// An ordinary rounding.
    pub(super) const ORDINARY_ROUNDING: usize = 10;
    /// The greatest finite value, which overflows.
    pub(super) const OVERFLOW: usize = 11;
    /// Positive infinity.
    pub(super) const POSITIVE_INFINITY: usize = 12;
    /// Negative infinity.
    pub(super) const NEGATIVE_INFINITY: usize = 13;
    /// A non-canonical NaN that canonicalizes.
    pub(super) const NONCANONICAL_NAN: usize = 14;
}

/// The corpus covers every class the ticket names, and each is reached.
///
/// Named and counted rather than asserted as a length: a corpus whose subnormal
/// group silently emptied would still be fifteen elements long if something else
/// grew, and every comparison below would keep passing while measuring less.
#[test]
fn the_corpus_covers_every_class_the_ticket_names() {
    let cases = corpus();
    assert_eq!(cases.len(), 15, "the corpus is the one that was derived");

    // Both zeros with their signs.
    assert_eq!(cases[position::POSITIVE_ZERO].operand, 0x0000);
    assert_eq!(cases[position::NEGATIVE_ZERO].operand, 0x8000);
    // The least positive and least negative subnormals.
    assert_eq!(cases[position::LEAST_POSITIVE_SUBNORMAL].operand, 0x0001);
    assert_eq!(cases[position::LEAST_NEGATIVE_SUBNORMAL].operand, 0x8001);
    // The greatest subnormal, and the least normal beside it.
    assert_eq!(cases[position::GREATEST_SUBNORMAL].operand, 0x007f);
    assert_eq!(cases[position::LEAST_NORMAL].operand, 0x0080);
    // A tie resolved to even, and an ordinary rounding that is not a tie.
    assert_eq!(cases[position::TIE].operand, 0x3f81);
    assert_eq!(cases[position::TIE].declared, 0x3fc2);
    assert_eq!(cases[position::ORDINARY_ROUNDING].operand, 0x3fff);
    assert_eq!(cases[position::ORDINARY_ROUNDING].declared, 0x403f);
    // An overflow to infinity, and both infinities as operands.
    assert_eq!(cases[position::OVERFLOW].operand, 0x7f7f);
    assert_eq!(cases[position::OVERFLOW].declared, 0x7f80);
    assert_eq!(cases[position::POSITIVE_INFINITY].operand, 0x7f80);
    assert_eq!(cases[position::NEGATIVE_INFINITY].operand, 0xff80);
    // A non-canonical NaN that canonicalizes.
    assert_eq!(cases[position::NONCANONICAL_NAN].operand, 0x7fc1);
    assert_eq!(cases[position::NONCANONICAL_NAN].declared, 0x7fc0);
    assert_ne!(
        cases[position::NONCANONICAL_NAN].operand,
        cases[position::NONCANONICAL_NAN].declared,
        "a payload that survived would make the canonicalization unobservable",
    );

    // Finding 24's two measured input-flush operands are in the corpus by their
    // measured encodings, so the run is about the rows that were measured.
    assert_eq!(cases[position::HALF_MIN_NORMAL].operand, 0x0040);
    assert_eq!(cases[position::NEG_HALF_MIN_NORMAL].operand, 0x8040);

    // The two execution witnesses are on non-subnormal operands, which is what
    // the ticket requires of them.
    assert_eq!(cases[position::ONE].operand, 0x3f80);
    assert_eq!(cases[position::ONE].declared, SCALE_BITS);
    assert!(!is_subnormal(cases[position::ONE].operand));
    assert!(!is_subnormal(cases[position::NEGATIVE_ZERO].operand));
}

/// Returns whether one encoding is a BF16 subnormal.
///
/// A zero exponent field with a nonzero trailing significand. The two zeros are
/// excluded because they are not subnormal, which is why the flush leaves them
/// alone.
fn is_subnormal(bits: u16) -> bool {
    bits & 0x7f80 == 0 && bits & 0x007f != 0
}

/// The hand-derived corpus agrees with the oracle under both readings.
///
/// **The hand-derived encodings are the claim and the oracle is what is
/// checked**, not the other way round: every `preserved` and `declared` value
/// in the corpus was derived from BF16's parameters and the
/// round-to-nearest-ties-to-even rule before anything ran, and none was
/// recorded from an execution. Both readings are checked, because a corpus that
/// only stated the flushing answer could not show which elements the flush is
/// responsible for.
#[test]
fn the_hand_derived_corpus_agrees_with_the_oracle_under_both_readings() {
    assert_eq!(
        reference_bits(ReferenceNumericalConformance::strict()),
        preserved_expectations(),
        "the preserving reading disagrees with the hand-derived corpus",
    );
    assert_eq!(
        reference_bits(declared_conformance()),
        declared_expectations(),
        "the declared flushing reading disagrees with the hand-derived corpus",
    );
}

/// The declared flush moves exactly the subnormal operands, and they are named.
///
/// Finding 24 measures BF16 arithmetic on the macOS row flushing subnormal
/// operands and results, so bit equality against a *preserving* oracle on these
/// five elements would mean the device did not do what was measured. This is
/// what makes the comparison below a comparison against the declared contract
/// rather than against an unstated reading.
#[test]
fn the_declared_flush_moves_exactly_the_subnormal_operands() {
    let moved = flush_moved_indices();
    assert_eq!(
        moved,
        vec![
            position::LEAST_POSITIVE_SUBNORMAL,
            position::LEAST_NEGATIVE_SUBNORMAL,
            position::HALF_MIN_NORMAL,
            position::NEG_HALF_MIN_NORMAL,
            position::GREATEST_SUBNORMAL,
        ],
        "the elements the flush moves are not the ones this run names",
    );

    let cases = corpus();
    for (index, case) in cases.iter().enumerate() {
        assert_eq!(
            moved.contains(&index),
            is_subnormal(case.operand),
            "{}: moved-by-the-flush and subnormal-operand disagree",
            case.name,
        );
    }
    // The neighbour that keeps this about subnormals rather than about the
    // bottom of the range: the least normal is one quantum above the greatest
    // subnormal, its product is normal, and neither dimension touches it.
    assert!(!moved.contains(&position::LEAST_NORMAL));
    assert_eq!(
        cases[position::LEAST_NORMAL].preserved,
        cases[position::LEAST_NORMAL].declared,
    );
}

/// The oracle and the region are told one contract, not two transcriptions.
#[test]
fn the_oracle_and_the_region_are_told_one_contract() {
    let contract = declared_contract();
    let realization = declared_realization();
    let conformance = declared_conformance();

    assert_eq!(realization.input_subnormals, contract.input_subnormals());
    assert_eq!(realization.result_subnormals, contract.result_subnormals());
    assert_eq!(conformance.input_subnormals(), contract.input_subnormals());
    assert_eq!(
        conformance.result_subnormals(),
        contract.result_subnormals()
    );
    assert_eq!(realization.profile_key, contract.key());

    // The contract is genuinely the flushing one. Under the strict BF16
    // contract every corpus element would take its preserved reading and the
    // five moved elements above would be empty, so this is the assertion that
    // keeps the whole comparison about the measured behaviour.
    assert_ne!(
        contract.key(),
        NumericalContract::STRICT_BF16.key(),
        "the vertical must compare under the flushing contract",
    );
    assert_eq!(
        reference_bits(ReferenceNumericalConformance::new(
            NumericalContract::STRICT_BF16.input_subnormals(),
            NumericalContract::STRICT_BF16.result_subnormals(),
        )),
        preserved_expectations(),
        "the strict contract resolves to the preserving reading",
    );
}

/// The region lowers to a `bfloat` kernel bound to the authoritative row.
#[test]
fn the_emitted_unit_is_a_bfloat_kernel_bound_to_the_authoritative_row() {
    let elements = u64::try_from(corpus().len()).unwrap();
    let emitted = emit_vertical(elements).expect("the bf16 vertical emits");

    for buffer in emitted.kernel.buffers() {
        assert_eq!(
            buffer.element_type,
            KernelType::Bf16,
            "every boundary of a bf16 region is bf16",
        );
    }
    assert_eq!(emitted.element_count, elements);
    assert_eq!(emitted.grid_threads, elements);
    // The launch the *schedule* declares, not one this run chose. A dispatch
    // encoded at any other workgroup width would be executing a geometry the
    // region never asked for, whatever the arithmetic came out as.
    assert_eq!(emitted.threads_per_workgroup, 1);
    assert_eq!(emitted.operand_index, 0);
    assert_eq!(emitted.result_index, 1);

    let source = emitted.unit.source();
    assert!(source.contains("device const bfloat *b0"), "{source}");
    assert!(source.contains("device bfloat *b1"), "{source}");
    // The constant carrier the offline toolchain requires: `as_type` needs equal
    // widths and an unsuffixed MSL integer literal is `uint`.
    assert!(
        source.contains("as_type<bfloat>(ushort(0x3fc0u))"),
        "the scale is not the exact pattern this corpus was derived for: {source}",
    );
    assert!(
        source.contains("as_type<bfloat>(ushort(0x0000u))"),
        "the bias is not the exact pattern this corpus was derived for: {source}",
    );
    assert!(
        source.contains("tiler_canonicalize_nan_bf16_7fc0"),
        "the bf16 NaN canonicalization is absent: {source}",
    );
    assert!(!source.contains("fma("), "{source}");

    // The profile this run is bound to, named rather than inferred.
    assert_eq!(
        emitted.declaration.aot_target().triple(),
        "air64-apple-macos26.0",
    );
    assert_eq!(emitted.declaration.aot_target().std_token(), "metal4.0");
}

/// A strict BF16 region is refused on the measured macOS row, observed failing.
///
/// The perturbation that shows the flush-accepting unit is admitted for a
/// reason. The measured row flushes, so a region declaring preservation states
/// an obligation the target cannot realize, and emission's own conformance
/// check is what says so — not the compilation, which would accept the identical
/// source.
#[test]
fn a_strict_bf16_region_is_refused_on_the_measured_row() {
    let elements = u64::try_from(corpus().len()).unwrap();
    let declaration = tiler_build::BoundMetalCompileDeclaration::first_macos_apple9()
        .expect("the authoritative declaration assembles");

    let strict = region_under(elements, realization_of(NumericalContract::STRICT_BF16));
    let kernel = lower_scheduled_region(&strict).expect("the strict bf16 region lowers");
    let unit = emit_translation_unit(
        &[&kernel],
        declaration.metal_facts(),
        declaration.emission(),
    )
    .expect("emission succeeds even for an unrealizable contract");
    let refusal = unit
        .require_declared_realization()
        .expect_err("a subnormal-preserving bf16 contract is not deliverable on this row");
    assert_eq!(refusal.rule(), "unrealizable-numerical-obligation");

    // And the flush-accepting region on the very same target is admitted, so the
    // refusal is a decision about the contract rather than a blanket one.
    emit_vertical(elements).expect("the flush-accepting vertical emits");
}

/// An operand width derived from the neighbouring carrier changes what the
/// kernel reads, without a device.
///
/// The host-visible half of the composition perturbation. The kernel addresses
/// its buffer at `tiler::bf16@1`'s own two-byte stride; a host that packed at
/// the `f32` carrier's four bytes hands it a completely different operand
/// sequence, and every layer-local test would still pass because each of them
/// uses one width on both sides.
#[test]
fn a_wrongly_derived_operand_width_changes_what_the_kernel_reads() {
    assert_eq!(StorageScalar::Bf16.byte_width(), 2);
    assert_eq!(OperandStride::Declared.bytes(), 2);
    assert_eq!(OperandStride::NeighbouringF32.bytes(), 4);

    let corpus_operands = operands();
    let declared = OperandStride::Declared.bytes();

    // Packed at the declared width, the kernel reads the corpus back exactly.
    let honest = pack(&corpus_operands, declared);
    assert_eq!(honest.len(), corpus_operands.len() * 2);
    assert_eq!(
        unpack(&honest, declared, corpus_operands.len()),
        corpus_operands,
    );

    // Packed at the neighbouring width and addressed at the declared one — the
    // asymmetry a single mis-derived site produces — every other element the
    // kernel reads is a zero the corpus never contained.
    let perturbed = pack(&corpus_operands, OperandStride::NeighbouringF32.bytes());
    let seen = unpack(&perturbed, declared, corpus_operands.len());
    assert_ne!(
        seen, corpus_operands,
        "a four-byte packing must not present the same operands at a two-byte stride",
    );
    assert_eq!(seen[0], corpus_operands[0]);
    assert_eq!(seen[1], 0x0000, "the high half of the first f32 slot");
    assert_eq!(seen[2], corpus_operands[1]);

    // The symmetric error is the one a layer-local test cannot catch, and
    // stating it is why the perturbation is asymmetric: packing *and* unpacking
    // at four bytes round-trips the corpus unchanged.
    let symmetric = OperandStride::NeighbouringF32.bytes();
    assert_eq!(
        unpack(&perturbed, symmetric, corpus_operands.len()),
        corpus_operands,
        "a width error applied to both sides cancels, which is the whole reason this run \
         perturbs one side",
    );
}

/// The vertical agrees with the oracle, element for element, on the row that ran.
///
/// The one comparison this crate exists for. The device executes the emitted
/// `bfloat` kernel over the corpus and the result is compared against the
/// oracle's evaluation of the *semantic* program under the declared flushing
/// conformance — two independent implementations of one declared contract, with
/// no shared host expression between them.
///
/// The expected encodings are the corpus's own hand-derived `declared` column,
/// which
/// [`the_hand_derived_corpus_agrees_with_the_oracle_under_both_readings`] has
/// already held the oracle to, so a disagreement here names the device rather
/// than leaving two implementations to agree for unexamined reasons.
#[test]
fn the_bf16_vertical_agrees_with_the_oracle_on_the_measured_row() {
    let elements = u64::try_from(corpus().len()).unwrap();
    let emitted = emit_vertical(elements).expect("the bf16 vertical emits");
    let expected = declared_expectations();
    assert_eq!(
        expected,
        reference_bits(declared_conformance()),
        "the oracle and the hand-derived column must already agree before a device is asked",
    );

    let Some(observed) = require_or_report(
        "bf16 vertical",
        measured_half(&emitted, OperandStride::Declared),
    ) else {
        return;
    };

    let cases = corpus();
    let mut disagreements = Vec::new();
    for (index, (case, (want, got))) in cases.iter().zip(expected.iter().zip(&observed)).enumerate()
    {
        if want != got {
            disagreements.push(format!(
                "[{index}] {}: operand {:#06x}, expected {want:#06x}, device returned {got:#06x}",
                case.name, case.operand,
            ));
        }
    }
    assert_eq!(
        observed.len(),
        expected.len(),
        "the device returned {} element(s) for a {}-element corpus",
        observed.len(),
        expected.len(),
    );
    assert!(disagreements.is_empty(), "{disagreements:#?}");

    // The execution witnesses, named and checked individually rather than left
    // to the bulk comparison. Without them `flushed` and `the arithmetic was
    // optimized away` are the same observation.
    assert_eq!(
        observed[position::ONE],
        SCALE_BITS,
        "the multiply did not run: operand 0x3f80 returned unchanged",
    );
    assert_eq!(
        observed[position::NEGATIVE_ZERO],
        0x0000,
        "the add did not run: operand 0x8000 returned unchanged",
    );
    eprintln!(
        "bf16 vertical: execution witnesses — multiply 0x3f80 -> {:#06x}, add 0x8000 -> {:#06x}",
        observed[position::ONE],
        observed[position::NEGATIVE_ZERO],
    );

    // And the flush is what the device delivered, not the preserving reading:
    // the five named elements returned the flushed answer and none returned the
    // preserved one.
    let preserved = preserved_expectations();
    for index in flush_moved_indices() {
        assert_eq!(observed[index], cases[index].declared);
        assert_ne!(
            observed[index], preserved[index],
            "[{index}] {}: the device returned the *preserving* answer, which contradicts the \
             measured flush this run compares under",
            cases[index].name,
        );
    }
    eprintln!(
        "bf16 vertical: the declared flush moved elements {:?}; each returned its flushed answer",
        flush_moved_indices(),
    );
}

/// The composition perturbation is observed failing on the measured row.
///
/// The device half of
/// [`a_wrongly_derived_operand_width_changes_what_the_kernel_reads`]: the same
/// dispatch, with the operand payload laid out at the neighbouring carrier's
/// width while the kernel keeps addressing at its own. Every layer-local test
/// passes this defect; the composition does not.
#[test]
fn the_composition_perturbation_is_observed_failing_on_the_measured_row() {
    let elements = u64::try_from(corpus().len()).unwrap();
    let emitted = emit_vertical(elements).expect("the bf16 vertical emits");

    let Some(observed) = require_or_report(
        "bf16 width perturbation",
        measured_half(&emitted, OperandStride::NeighbouringF32),
    ) else {
        return;
    };

    let expected = declared_expectations();
    assert_ne!(
        observed, expected,
        "an operand payload strided at the f32 carrier's width produced the correct result, so \
         this run cannot observe a wrongly derived element width at all",
    );
    let differing: Vec<_> = expected
        .iter()
        .zip(&observed)
        .enumerate()
        .filter(|(_, (want, got))| want != got)
        .map(|(index, (want, got))| format!("[{index}] expected {want:#06x}, got {got:#06x}"))
        .collect();
    eprintln!(
        "bf16 width perturbation: watched failing at {} of {} elements:\n{differing:#?}",
        differing.len(),
        expected.len(),
    );
    // The perturbation must be visible on more than a boundary element, or a
    // narrower corpus could stop detecting it.
    assert!(
        differing.len() >= 5,
        "the perturbation moved only {} element(s)",
        differing.len(),
    );
}

/// The unsafe site population is the two named ones, and nothing else.
///
/// Tom's rule requires the population to be named and counted where a reader
/// will find it, so that a later addition is visible rather than absorbed. This
/// is the check that makes a third site a red test: it walks every Rust source
/// file under `src/`, so a *new file* containing `unsafe` is caught as well as a
/// new site in an existing one.
///
/// The needle is assembled at run time from two pieces, because a literal
/// spelling of it in this file would be a match against this scanner's own
/// source.
#[test]
fn the_unsafe_site_population_is_the_two_named_ones() {
    let needle = format!("{}{}", "unsafe", " {");
    let allow = format!("{}{}", "unsafe", "_code,");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    let mut files = Vec::new();
    collect_rust_sources(&root, &mut files);
    files.sort();
    assert!(
        files.len() >= 12,
        "the scan found {} source file(s), which is fewer than this crate has; a walk that \
         stopped finding files would report an empty population as a clean one. The floor rises \
         with the crate rather than tracking it exactly, so adding a module is not an edit here \
         and losing most of them still is.",
        files.len(),
    );

    let mut blocks = 0_usize;
    let mut allows = 0_usize;
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a crate source file is readable");
        let file_blocks = text.matches(needle.as_str()).count();
        let file_allows = text.matches(allow.as_str()).count();
        let owner = path.file_name().and_then(|name| name.to_str()) == Some("device_buffer.rs");
        assert!(
            owner || file_blocks == 0,
            "{}: unsafe belongs in device_buffer.rs alone, and this file has {file_blocks} \
             block(s)",
            path.display(),
        );
        assert!(
            owner || file_allows == 0,
            "{}: an unsafe-code allow outside device_buffer.rs",
            path.display(),
        );
        blocks += file_blocks;
        allows += file_allows;
    }

    assert_eq!(
        blocks, 2,
        "the admitted population is two sites — device_buffer::write_bytes and \
         device_buffer::read_bytes — and this scan found {blocks}. A third site needs Tom's \
         decision under the rule on \
         `decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`, not an \
         updated number here.",
    );
    assert_eq!(
        allows, 2,
        "each unsafe site carries its own reasoned allow; found {allows}",
    );
}

/// Collects every `.rs` file beneath one directory.
fn collect_rust_sources(directory: &Path, into: &mut Vec<std::path::PathBuf>) {
    let entries = std::fs::read_dir(directory).expect("the crate's source directory is readable");
    for entry in entries {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, into);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            into.push(path);
        }
    }
}

/// Every corpus case is reachable by name, so an unnamed one cannot hide.
#[test]
fn every_corpus_case_is_named() {
    let mut names: Vec<&'static str> = corpus().iter().map(|case: &Case| case.name).collect();
    let stated = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), stated, "two corpus cases share one name");
    assert!(names.iter().all(|name| !name.is_empty()));
}
