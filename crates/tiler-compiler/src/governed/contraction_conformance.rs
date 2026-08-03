//! Bit-level conformance of the governed contraction lowering.
//!
//! # What is compared, and against what
//!
//! Three independent computations of the same contraction:
//!
//! 1. the **emitted index region** — the region `refine_index_region` actually
//!    returned for the occurrence, executed by `tiler-reference`'s independent
//!    index-region oracle;
//! 2. the **registered reference evaluator** for
//!    `tiler::strict-tensor-contraction-f32@1`, which decodes the family's
//!    fourteen-field numerical signature rather than restating it; and
//! 3. the **retained `result_sha256`** of the L3 realization probe's `direct`
//!    kernel, measured on an Apple M4 Max and recorded under
//!    `spikes/scheduling/metal_contraction_vertical/results/`.
//!
//! The third is what makes this more than an agreement between two host
//! implementations that could be wrong together. Its operands are reconstructed
//! here from the probe's own `SplitMix64` stream, so each digest is computed over
//! the same bytes the device consumed; a reconstruction that drifted would
//! disagree at every cell rather than pass silently.
//!
//! # The two comparisons have different reaches, and the difference is stated
//!
//! **The reference's unstaged fold reaches two profile cells in one call each.**
//! `w_decode_kv` and `w_vocab_slice` are evaluated at their own extents, and each
//! result digest is compared against the retained one. Its *staged* fold reaches
//! all six; the section after next states which of those this file drives, and
//! why not all of them.
//!
//! **The index-region oracle reaches the smaller of the two.**
//! `MAX_EVALUATION_STEPS` caps one region evaluation at 16,777,216 scalar and
//! index-expression steps, which admits `w_decode_kv`'s 1,048,576 contracted
//! points and refuses `w_vocab_slice`'s 8,388,608. So the emitted region is
//! compared against the reference's `w_decode_kv` result element by element —
//! and, because that result's digest is the retained one, that is a bit-for-bit
//! statement about the emitted region against a *measured device result* rather
//! than against a second host implementation. The vocabulary cell's refusal is
//! asserted rather than routed around: the budget lives in `tiler-reference`,
//! which this work does not own, and a bound that had quietly moved would turn
//! a stated boundary into an unnoticed one.
//!
//! # The reference's four-cell boundary was settled, and this file no longer states it
//!
//! `MAX_REFERENCE_TENSOR_ELEMENTS` bounds a *single* fold at 16,777,216 steps,
//! and `contract_operands` still refuses `output_count * contracted_count` above
//! it under `IterationStepsExceeded` — which four of the six L3 correctness cells
//! exceed: `w_prefill_q` at 20,971,520 steps, `w_prefill_mlp_in` and
//! `w_prefill_mlp_out` at 402,653,184, and `w_prefill_o` at 268,435,456. No
//! operand or output tensor exceeds a limit; only the fold's step count does.
//!
//! **`bound-the-reference-contraction-comparison-for-the-profile-cells` (done,
//! 2026-08-01) settled that boundary by staging the comparison rather than by
//! moving the bound.** `StagedStrictTensorContractionF32` folds one output slab
//! per call, each slab passing exactly the test the unstaged path applies, so all
//! six cells are reachable and the whole-program refusal that protects the host is
//! byte-for-byte unchanged. An earlier revision of this passage said the boundary
//! was "deliberately not settled here" and that four cells were uncompared; both
//! sentences are now false, and the two tests below are what make the correction
//! checked rather than asserted —
//! [`the_four_prefill_cells_are_refused_by_the_unstaged_fold_and_reached_by_the_staged_one`]
//! drives both halves on the cheapest of the four.
//!
//! # What is still uncompared here, and what bounds it
//!
//! The **emitted index region** is compared at `w_decode_kv` alone, and the bound
//! is the region oracle's cost rather than any refusal. **Measurement — Apple M4
//! Max, 2026-08-01, dev profile**, from `tiler-reference`'s own
//! `tests/contraction_profile_cells.rs`: the region oracle spends about 516 ns per
//! budget step against the contraction fold's 9, because it allocates a rank-zero
//! tensor per scalar value, resolves a registered capability per application, and
//! revalidates every result. The six cells' 1.1 × 10⁹ fold steps are therefore
//! hours of region walk, which is not a cost to put behind an `#[ignore]` and call
//! evidence. Closing that gap needs a dispatched device comparison rather than a
//! third host implementation, and
//! [`integrate-the-contraction-vertical-into-the-runtime`](../../../../tickets/integrate-the-contraction-vertical-into-the-runtime.md)
//! owns it.
//!
//! # Why the digest helper is written out here
//!
//! `sha2` is a workspace dependency, but adding it to this crate would edit
//! `Cargo.lock`, which this work does not own. The implementation below is
//! therefore local to this test module and is checked against the two published
//! FIPS 180-4 vectors before any comparison rests on it — a digest function that
//! silently computed something else would make every retained-value assertion
//! agree with itself.

use std::fmt::Write as _;

use tiler_ir::index::{IndexRefinementSubject, NumericalContractIdentity};
use tiler_ir::program::SemanticOccurrence;
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32TensorContraction, InputKey, OutputKey,
    SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, FrozenReferenceRegistry, FrozenScalarReferenceRegistry, IndexRegionAuthority,
    IndexRegionEvaluationError, IndexRegionEvaluator, IndexRegionInput, InputBinding,
    ReferenceElement, ReferenceEvaluator, StagedStrictTensorContractionF32, Tensor,
    TensorPayloadView,
};

use super::{governed_lowering_capabilities, governed_scalars};
use crate::capability::LoweringSignature;
use crate::legality::refine_index_region;

/// The probe's workload seed, `contraction_probe.py`'s `WORKLOAD_SEED`.
const WORKLOAD_SEED: u64 = 0x5445_524D;

/// The probe's right-operand seed derivation, `host.m`'s `fill_prng` call.
const RIGHT_SEED_MASK: u64 = 0xA5A5_A5A5_A5A5_A5A5;

/// One profile cell the reference evaluator admits, with its retained digest.
struct Cell {
    id: &'static str,
    m: u64,
    n: u64,
    k: u64,
    /// SHA-256 of the `direct` kernel's result bytes, little-endian `f32`,
    /// row-major, from the retained correctness record.
    result_sha256: &'static str,
}

/// The two cells of the L3 correctness profile whose fold the reference admits.
const ADMITTED_CELLS: [Cell; 2] = [
    Cell {
        id: "w_decode_kv",
        m: 1,
        n: 1024,
        k: 1024,
        result_sha256: "79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f",
    },
    Cell {
        id: "w_vocab_slice",
        m: 1,
        n: 8192,
        k: 1024,
        result_sha256: "88b01ae776f42bdb2f2d1092ddfd039e20e652d28393a6e2ec19e5cc1d9803c8",
    },
];

/// The four cells the *unstaged* fold refuses, with the exact fold each asks for.
const REFUSED_CELLS: [(&str, u64, u64, u64, usize); 4] = [
    ("w_prefill_q", 10, 2048, 1024, 20_971_520),
    ("w_prefill_mlp_in", 128, 3072, 1024, 402_653_184),
    ("w_prefill_mlp_out", 128, 1024, 3072, 402_653_184),
    ("w_prefill_o", 128, 1024, 2048, 268_435_456),
];

/// The cheapest of the four, which the staged fold reaches on every run.
///
/// One of the four rather than all of them, and the cheapest one deliberately: at
/// the measured 9 ns per step its 20,971,520-step fold costs about 0.2 s, where
/// the four together are 1.1 × 10⁹ steps. All six digests are checked in
/// `tiler-reference`'s own `tests/contraction_profile_cells.rs`; what this cell
/// buys *here* is that the boundary statement in this module's documentation
/// cannot go stale again without a test noticing.
const STAGED_CELL: Cell = Cell {
    id: "w_prefill_q",
    m: 10,
    n: 2048,
    k: 1024,
    result_sha256: "1c54f5cd7265ee288ec79bcd9254243b78a95d57c3c489e5ea90bcc4298073c0",
};

/// The reference's own fold bound, restated from `tiler-reference`'s limit.
const REFERENCE_STEP_LIMIT: usize = 16 * 1024 * 1024;

/// The decode cell's contracted extent and output width.
const DECODE_K: u64 = 1024;
const DECODE_N: u64 = 1024;

/// The vocabulary cell's output width.
const VOCAB_N: u64 = 8192;

fn activations_key() -> InputKey {
    InputKey::new("activations").expect("a governed key")
}

fn weights_key() -> InputKey {
    InputKey::new("weights").expect("a governed key")
}

fn splitmix64(x: u64) -> u64 {
    let x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let z = x;
    let z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The probe's operand value at one index: `m * 2^-24` with `m` an integer in
/// `[-2^23, 2^23)`.
///
/// Every such value is exactly representable in binary32, which is the property
/// the probe chose it for: the operands themselves introduce no rounding, so any
/// difference a comparison reports is a difference in how the contraction was
/// evaluated.
fn prng_value(seed: u64, index: u64) -> f32 {
    let bits = splitmix64(seed.wrapping_add(index.wrapping_mul(0x2545_F491_4F6C_DD1D)));
    let field =
        i64::from(u32::try_from((bits >> 40) & 0xFF_FFFF).expect("a 24-bit field fits in u32"));
    #[expect(
        clippy::cast_precision_loss,
        reason = "an integer in [-2^23, 2^23) is exactly representable in binary32"
    )]
    let magnitude = (field - 8_388_608) as f32;
    magnitude * (1.0 / 16_777_216.0)
}

fn prng_tensor(shape: Shape, seed: u64) -> Tensor {
    let count = shape.element_count().expect("a profile cell is bounded");
    let elements = (0..count)
        .map(|index| {
            ReferenceElement::from_float_bits(
                prng_value(seed, index as u64).to_bits().to_be_bytes(),
                FloatBitOrder::MostSignificantByteFirst,
            )
            .expect("a generated operand is a valid f32 pattern")
        })
        .collect();
    Tensor::dense(F32::resolved_type(), shape, elements).expect("the operand tensor is well formed")
}

/// The profile's index structure, `td,od->to`, spelled with the frontend's own
/// labels so the canonicalization is exercised rather than assumed.
fn projection_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .expect("the profile's structure passes every admission rule")
}

/// Builds `activations[m, k] x weights[n, k] -> projected[m, n]`.
fn projection_program(m: u64, n: u64, k: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the standard registry composes");
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").expect("a governed key"),
            Shape::from_dims([m, k]),
        )
        .expect("the first operand is declared");
    let weights = builder
        .input::<F32>(
            InputKey::new("weights").expect("a governed key"),
            Shape::from_dims([n, k]),
        )
        .expect("the second operand is declared");
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), activations, weights)
            .expect("the occurrence is well formed");
    builder
        .output(
            OutputKey::new("projected").expect("a governed key"),
            projected,
        )
        .expect("the result is named");
    builder.build().expect("the program verifies")
}

/// Returns the exact `f32` bit patterns of a dense reference tensor.
fn result_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a contraction result is a dense f32 tensor")
    };
    element_bits(elements)
}

/// Returns the exact `f32` bit patterns of a run of reference elements.
///
/// Split from [`result_bits`] because the staged fold hands back elements rather
/// than an assembled tensor, and reassembling one only to read its payload back
/// would put a shape and a dtype between the slabs and the digest they are
/// compared through.
fn element_bits(elements: &[ReferenceElement]) -> Vec<u32> {
    elements
        .iter()
        .map(|value| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(value.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

/// The probe's digest domain: little-endian `f32` bytes in row-major order.
fn result_digest(bits: &[u32]) -> String {
    let bytes: Vec<u8> = bits.iter().flat_map(|value| value.to_le_bytes()).collect();
    sha256_hex(&bytes)
}

/// Evaluates one contraction through the registered reference operation.
fn reference_result(program: &SemanticProgram, left: &Tensor, right: &Tensor) -> Vec<u32> {
    let evaluator = ReferenceEvaluator::new(
        FrozenReferenceRegistry::standard().expect("the governed value profile composes"),
    );
    let outputs = evaluator
        .evaluate(
            program,
            &[
                InputBinding::new(&activations_key(), left),
                InputBinding::new(&weights_key(), right),
            ],
        )
        .expect("the governed reference evaluates the profile cell");
    result_bits(&outputs[0])
}

/// Evaluates the region the governed lowering emitted for the same occurrence.
fn emitted_region_result(m: u64, n: u64, k: u64, left: &Tensor, right: &Tensor) -> Vec<u32> {
    emitted_region_evaluation(m, n, k, left, right)
        .expect("the emitted region executes on the oracle")
}

/// Returns the oracle's refusal for a region it declines to evaluate.
fn emitted_region_error(
    m: u64,
    n: u64,
    k: u64,
    left: &Tensor,
    right: &Tensor,
) -> IndexRegionEvaluationError {
    emitted_region_evaluation(m, n, k, left, right)
        .expect_err("this shape exceeds the oracle's evaluation budget")
}

fn emitted_region_evaluation(
    m: u64,
    n: u64,
    k: u64,
    left: &Tensor,
    right: &Tensor,
) -> Result<Vec<u32>, IndexRegionEvaluationError> {
    let scalars = governed_scalars().expect("the governed scalar authority composes");
    let registry =
        governed_lowering_capabilities(&scalars).expect("the governed capabilities compose");
    let realizations = super::governed_realization_laws(&scalars);
    let program = projection_program(m, n, k);
    let occurrence = IndexRefinementSubject::derive(
        &program,
        SemanticOccurrence::new(0),
        NumericalContractIdentity::try_from_key(
            crate::request::StrictF32NumericalContract::governed().key,
        )
        .unwrap(),
    )
    .expect("the verified contraction yields a refinement subject");
    let signature = LoweringSignature::new(
        occurrence.signature().operands().iter().cloned(),
        occurrence.signature().results().iter().cloned(),
    )
    .expect("the occurrence's signature is bounded");
    let resolved = registry
        .resolve_index_access(occurrence.operation(), &signature)
        .expect("the governed registry covers the contraction");
    let refinement = refine_index_region(&resolved, &occurrence, &realizations, &scalars)
        .expect("the governed contraction lowering refines")
        .into_refined()
        .expect("the lowering discharges every index-domain predicate");
    let evaluator = IndexRegionEvaluator::new(
        FrozenReferenceRegistry::standard().expect("the governed value profile composes"),
        FrozenScalarReferenceRegistry::standard().expect("the governed scalar oracle composes"),
    );
    let evaluation = evaluator.evaluate(
        refinement.region(),
        IndexRegionAuthority::new(&scalars),
        &[
            IndexRegionInput::new(refinement.operand_bindings()[0].input_tensor(), left),
            IndexRegionInput::new(refinement.operand_bindings()[1].input_tensor(), right),
        ],
    )?;
    Ok(result_bits(&evaluation.outputs()[0]))
}

/// The reference reproduces the retained `direct` measurement at both admitted
/// cells, and the emitted region reproduces the reference where the oracle can
/// run it.
///
/// Comparison is on exact bit patterns rather than `f32` equality: `-0.0 == 0.0`
/// holds and a NaN equals nothing, so float comparison would silently accept
/// exactly the results a numerical contract exists to pin.
#[test]
fn the_contraction_agrees_with_the_reference_and_the_retained_measurement() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "the digest helper reproduces the published empty-string vector",
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "and the published three-byte vector",
    );

    let mut decode_kv = None;
    for cell in &ADMITTED_CELLS {
        let left = prng_tensor(Shape::from_dims([cell.m, cell.k]), WORKLOAD_SEED);
        let right = prng_tensor(
            Shape::from_dims([cell.n, cell.k]),
            WORKLOAD_SEED ^ RIGHT_SEED_MASK,
        );
        let program = projection_program(cell.m, cell.n, cell.k);
        let reference = reference_result(&program, &left, &right);
        assert_eq!(
            reference.len(),
            usize::try_from(cell.m * cell.n).unwrap(),
            "{}: every output element is produced",
            cell.id
        );
        assert_eq!(
            result_digest(&reference),
            cell.result_sha256,
            "{}: the reference does not reproduce the retained `direct` result",
            cell.id
        );
        if cell.id == "w_decode_kv" {
            decode_kv = Some(reference);
        }
    }
    let decode_kv = decode_kv.expect("the decode cell is in the admitted set");

    // The whole of `w_decode_kv`, which is the larger of the two cells the
    // index-region oracle's step budget admits. The comparison is against the
    // reference's own result for that cell — whose digest matched the retained
    // measurement two assertions above — so this is a bit-for-bit statement
    // about the emitted region against a *measured device result*, element by
    // element, and not merely against a second host implementation.
    let left = prng_tensor(Shape::from_dims([1, DECODE_K]), WORKLOAD_SEED);
    let right = prng_tensor(
        Shape::from_dims([DECODE_N, DECODE_K]),
        WORKLOAD_SEED ^ RIGHT_SEED_MASK,
    );
    let emitted = emitted_region_result(1, DECODE_N, DECODE_K, &left, &right);
    assert_eq!(
        emitted, decode_kv,
        "the emitted region disagrees with the retained `w_decode_kv` measurement",
    );
    assert_eq!(
        result_digest(&emitted),
        ADMITTED_CELLS[0].result_sha256,
        "and therefore with its digest",
    );
}

/// The index-region oracle refuses the larger admitted cell, under its own
/// bound.
///
/// This is the other half of the comparison above: its reach is a property of
/// `tiler-reference`'s evaluation budget, not of the emitted region, and a
/// budget that had been raised — or a refusal reported under a different
/// resource — would silently change which cells "the oracle can evaluate"
/// names. Raising it belongs to `tiler-reference`, which this work does not own.
#[test]
fn the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget() {
    let left = prng_tensor(Shape::from_dims([1, DECODE_K]), WORKLOAD_SEED);
    let right = prng_tensor(
        Shape::from_dims([VOCAB_N, DECODE_K]),
        WORKLOAD_SEED ^ RIGHT_SEED_MASK,
    );
    let error = emitted_region_error(1, VOCAB_N, DECODE_K, &left, &right);
    assert!(
        format!("{error}").contains("EvaluationSteps"),
        "the refusal must name the oracle's step budget: {error}"
    );
}

/// One perturbed contributing element changes the compared result.
///
/// Without this the agreement above would be consistent with a comparison that
/// never looked at the operands — a digest over a constant, an oracle that
/// returned the reference's own answer, or an equality between two empty
/// vectors. Perturbing the *last* contributed element is deliberate: a fold that
/// stopped early, or one seeded at `+0.0` and therefore ignoring its first
/// contributor, would still be caught by a first-element perturbation, so the
/// last one is the position that discriminates the fold's completeness.
#[test]
fn a_single_perturbed_contributor_breaks_every_comparison() {
    let (m, n, k) = (1_u64, 4_u64, 4_u64);
    let left = prng_tensor(Shape::from_dims([m, k]), WORKLOAD_SEED);
    let right = prng_tensor(Shape::from_dims([n, k]), WORKLOAD_SEED ^ RIGHT_SEED_MASK);
    let program = projection_program(m, n, k);
    let baseline = reference_result(&program, &left, &right);
    assert_eq!(
        emitted_region_result(m, n, k, &left, &right),
        baseline,
        "the unperturbed pair agrees, so the perturbation below is the variable"
    );

    let TensorPayloadView::Dense(elements) = left.payload() else {
        panic!("a dense operand")
    };
    let mut perturbed: Vec<ReferenceElement> = elements.to_vec();
    let last = perturbed.len() - 1;
    let bits = u32::from_be_bytes(<[u8; 4]>::try_from(perturbed[last].as_bytes()).unwrap());
    // One representable value along, which is the smallest change the fold can
    // observe at all.
    perturbed[last] = ReferenceElement::from_float_bits(
        (bits + 1).to_be_bytes(),
        FloatBitOrder::MostSignificantByteFirst,
    )
    .unwrap();
    let left = Tensor::dense(F32::resolved_type(), Shape::from_dims([m, k]), perturbed).unwrap();

    assert_ne!(
        reference_result(&program, &left, &right),
        baseline,
        "the reference ignored a contributing element"
    );
    assert_ne!(
        emitted_region_result(m, n, k, &left, &right),
        baseline,
        "the emitted region ignored a contributing element"
    );
    assert_ne!(
        result_digest(&reference_result(&program, &left, &right)),
        result_digest(&baseline),
        "the digest ignored a changed result"
    );
}

/// The four prefill cells are refused by the unstaged fold and reached by the
/// staged one.
///
/// Both halves are asserted, and neither implies the other. The refusal is the
/// bound that protects a host from a whole-program fold it cannot afford, and a
/// bound that had quietly been raised would silently change what "the reference
/// refuses" names. The staged reach is the correction: a reader of this module
/// once learned that four cells were uncompared *because the reference could not
/// answer them*, and that has been false since
/// `bound-the-reference-contraction-comparison-for-the-profile-cells` landed.
/// Checking the reach against the same retained digest the two admitted cells are
/// checked against is what keeps the corrected sentence from going stale in the
/// other direction.
#[test]
fn the_four_prefill_cells_are_refused_by_the_unstaged_fold_and_reached_by_the_staged_one() {
    for (id, m, n, k, steps) in REFUSED_CELLS {
        assert!(
            steps > REFERENCE_STEP_LIMIT,
            "{id}: the recomputed fold must exceed the bound this refusal names"
        );
        let left = prng_tensor(Shape::from_dims([m, k]), WORKLOAD_SEED);
        let right = prng_tensor(Shape::from_dims([n, k]), WORKLOAD_SEED ^ RIGHT_SEED_MASK);
        let program = projection_program(m, n, k);
        let evaluator = ReferenceEvaluator::new(
            FrozenReferenceRegistry::standard().expect("the governed value profile composes"),
        );
        let error = evaluator
            .evaluate(
                &program,
                &[
                    InputBinding::new(&activations_key(), &left),
                    InputBinding::new(&weights_key(), &right),
                ],
            )
            .expect_err("the fold exceeds the reference's work bound");
        assert!(
            format!("{error}").contains("iteration space has"),
            "{id}: the refusal must name the work bound, not another limit: {error}"
        );
    }

    // The other half, on the cheapest of the four. The slabs are folded in index
    // order and concatenated, which is exactly the staged procedure: the planner
    // admits a slab width against the same bound the unstaged path is held to, and
    // the loop below is the only authorization any slab has.
    let cell = STAGED_CELL;
    let left = prng_tensor(Shape::from_dims([cell.m, cell.k]), WORKLOAD_SEED);
    let right = prng_tensor(
        Shape::from_dims([cell.n, cell.k]),
        WORKLOAD_SEED ^ RIGHT_SEED_MASK,
    );
    let staged = StagedStrictTensorContractionF32::governed(&projection_structure(), &left, &right)
        .expect("the staged planner admits the profile's structure");
    assert!(
        staged.slab_count() > 1,
        "a single-slab plan would fold {} steps at once and prove nothing about staging",
        staged.output_count() * staged.contracted_count(),
    );
    let mut elements = Vec::with_capacity(staged.output_count());
    for slab in 0..staged.slab_count() {
        elements.extend(
            staged
                .evaluate_slab(slab)
                .expect("every planned slab folds within the bound"),
        );
    }
    assert_eq!(
        elements.len(),
        usize::try_from(cell.m * cell.n).expect("the cell's output count fits in usize"),
        "{}: the slabs must cover the result exactly",
        cell.id
    );
    assert_eq!(
        result_digest(&element_bits(&elements)),
        cell.result_sha256,
        "{}: the staged fold does not reproduce the retained `direct` result",
        cell.id
    );
}

/// FIPS 180-4 SHA-256 over a byte string, as lowercase hexadecimal.
///
/// Local to this module for the reason the module documentation states, and
/// checked against the two published vectors before anything rests on it.
fn sha256_hex(message: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    let bit_length = u64::try_from(message.len())
        .expect("a message length fits in u64")
        .wrapping_mul(8);
    padded.extend_from_slice(&bit_length.to_be_bytes());

    let (blocks, remainder) = padded.as_chunks::<64>();
    debug_assert!(
        remainder.is_empty(),
        "the padding makes the length a multiple of 64"
    );
    for block in blocks {
        let mut schedule = [0_u32; 64];
        let (words, _) = block.as_chunks::<4>();
        for (slot, bytes) in schedule.iter_mut().zip(words) {
            *slot = u32::from_be_bytes(*bytes);
        }
        for index in 16..64 {
            let s0 = schedule[index - 15].rotate_right(7)
                ^ schedule[index - 15].rotate_right(18)
                ^ (schedule[index - 15] >> 3);
            let s1 = schedule[index - 2].rotate_right(17)
                ^ schedule[index - 2].rotate_right(19)
                ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }
        // The eight working variables, indexed rather than named: the standard
        // calls them `a` through `h`, and eight single-letter bindings is a
        // readability rule this workspace holds even where the source it
        // transcribes does not.
        let mut working = state;
        for index in 0..64 {
            let s1 = working[4].rotate_right(6)
                ^ working[4].rotate_right(11)
                ^ working[4].rotate_right(25);
            let choice = (working[4] & working[5]) ^ (!working[4] & working[6]);
            let temp1 = working[7]
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(schedule[index]);
            let s0 = working[0].rotate_right(2)
                ^ working[0].rotate_right(13)
                ^ working[0].rotate_right(22);
            let majority =
                (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let temp2 = s0.wrapping_add(majority);
            working = [
                temp1.wrapping_add(temp2),
                working[0],
                working[1],
                working[2],
                working[3].wrapping_add(temp1),
                working[4],
                working[5],
                working[6],
            ];
        }
        for (slot, value) in state.iter_mut().zip(working) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut hex = String::with_capacity(64);
    for byte in state.iter().flat_map(|word| word.to_be_bytes()) {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}
