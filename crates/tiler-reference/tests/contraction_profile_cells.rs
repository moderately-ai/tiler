//! Both reference oracles as bit-exact checks on the L3 contraction profile cells.
//!
//! Two independent implementations answer the same six cells here: the
//! registered `tiler::tensor-contraction-f32@1` evaluator, whose fold is
//! staged in output slabs, and the verified index-region oracle, whose walk is
//! staged in spans of root points. Each was blocked by a *different* bound
//! and each is reached without moving one; the second half starts at
//! [`contraction_region`].
//!
//! # The question this file closes
//!
//! `contract_operands` refuses a fold of more than `MAX_REFERENCE_TENSOR_ELEMENTS`
//! multiply-accumulate steps, and four of the six correctness cells of the L3
//! language-model contraction profile exceed it — `w_prefill_q` at 20,971,520
//! steps, `w_prefill_o` at 268,435,456, and `w_prefill_mlp_in` and
//! `w_prefill_mlp_out` at 402,653,184 each. Until this file, "the reference is the
//! oracle" was a claim about two cells and a boundary statement about four.
//!
//! [`StagedStrictTensorContractionF32`] reaches all six **without moving that
//! bound**: each slab's fold passes exactly the test the unstaged path applies,
//! and the total is the loop below. `w_prefill_q` is the demonstration that this
//! is a live oracle rather than a re-description — the same operands that the
//! unstaged evaluator refuses are folded in slabs to a result whose SHA-256 is the
//! one an Apple M4 Max produced.
//!
//! **The same cell now also arrives through the whole-*program* path.**
//! [`a_whole_program_evaluation_reaches_the_cell_its_default_evaluator_refuses`]
//! drives `w_prefill_q` through [`ReferenceEvaluator::evaluate`] — a verified
//! program, the registered capability, the registry's own dispatch — under an
//! evaluator whose caller stated an iteration-step allowance, and reproduces the
//! same device digest. The default evaluator's refusal is asserted in the same
//! test on the same operands, so authorizing the work is watched as a number that
//! can still say no rather than as a check that was removed. The two halves are
//! not the same claim: the staged type is what a caller holding two tensors uses,
//! and the evaluator is what a consumer holding a program calls.
//!
//! # Where the numbers come from
//!
//! - **Operands**: reconstructed from the probe's own `SplitMix64` stream,
//!   transcribed from `spikes/scheduling/metal_contraction_vertical/host.m`
//!   (`splitmix64`, `prng_value`, `fill_prng`), with the right operand's seed
//!   derivation `seed ^ 0xA5A5A5A5A5A5A5A5` from the same file's dispatch loop.
//!   Every generated value is `m * 2^-24` for an integer `m` in `[-2^23, 2^23)`
//!   and is therefore exactly representable in binary32, so the operands
//!   introduce no rounding of their own.
//! - **Extents and digests**: the `direct` rows of
//!   `spikes/scheduling/metal_contraction_vertical/results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/workload.tsv`.
//!   `result_sha256` there is `CC_SHA256` over the output buffer's raw bytes —
//!   little-endian `f32`, row-major — which is what [`digest_of`] reproduces.
//!
//! Reconstructing rather than recording is what makes a pass meaningful: the
//! digest is computed over the same bytes the device consumed, so a
//! reconstruction that had drifted would disagree at every cell rather than pass
//! silently.
//!
//! # What runs in the gate, and what does not
//!
//! This is an exact scalar oracle with a coordinate decode per step, so a cell's
//! cost is linear in its fold. [`the_staged_oracle_reaches_the_cheapest_refused_cell`]
//! runs `w_decode_kv` (1,048,576 steps) and `w_prefill_q` (20,971,520) on every
//! run, in 0.31 s, and
//! [`a_whole_program_evaluation_reaches_the_cell_its_default_evaluator_refuses`]
//! folds `w_prefill_q` twice more — once through the evaluator and once as a
//! seven-slab partition to compare it against — in 0.49 s.
//! [`the_staged_oracle_reproduces_every_retained_profile_digest`]
//! covers all six — 1,104,150,528 steps — and is `#[ignore]`d for cost. Its
//! invocation, and the run this file was landed with:
//!
//! ```text
//! cargo nextest run -p tiler-reference --run-ignored only --no-capture \
//!     -E 'binary(contraction_profile_cells)'
//! ```
//!
//! **Measurement — Apple M4 Max, 2026-08-01, nightly-2026-07-19.** All six cells
//! reproduce their retained digests. Dev profile: 10.8 s total, 9 ns per step,
//! 484 MB peak resident set (`/usr/bin/time -l`). Release profile, adding
//! `--release`: 5.5 s total, 4 ns per step. Per cell in the dev run —
//! `w_decode_kv` 10 ms, `w_prefill_q` 198 ms, `w_prefill_mlp_in` 3,799 ms,
//! `w_prefill_mlp_out` 3,796 ms, `w_prefill_o` 2,525 ms, `w_vocab_slice` 79 ms.
//!
//! The `#[ignore]` costs less drift detection than it looks like. Every helper
//! below — the operand reconstruction, the digest, the staged loop, the structure
//! — is shared with the default test, so only the four extra retained digests go
//! unchecked between deliberate runs, and all six cells share one fold: an
//! arithmetic change that moved them would move the two the gate runs.
//!
//! # The index-region oracle costs about fifty times more per step
//!
//! [`the_staged_index_region_oracle_reaches_the_vocabulary_cell`] is `#[ignore]`d
//! on the same terms and for the same reason, with a worse constant: the region
//! oracle allocates a rank-zero tensor per scalar value, resolves a registered
//! capability per application and revalidates every result, where the
//! contraction evaluator reads two floats and multiplies. It shares the
//! invocation above, and both `#[ignore]`d tests run under it.
//!
//! **Measurement — Apple M4 Max, 2026-08-01, nightly-2026-07-19, dev profile,
//! `--no-capture` (which nextest runs serially).** The region step rate is read
//! off the refusal rather than estimated, because the refusal's step count is
//! exact: one span over `w_vocab_slice`'s region is declined after 16,777,216
//! steps in 8.66 s, which is **516 ns per step** against the contraction fold's
//! 9. Walking the same region's 8,192 root points in 16 spans of 512 takes
//! 55.6 s and reproduces the retained `direct` digest; the whole test is 64.4 s.
//! The same run reproduced this file's other `#[ignore]`d test at its recorded
//! per-cell times (`w_prefill_mlp_in` 4,019 ms against 3,799), so the two
//! measurements are on one footing rather than one host apart.
//!
//! What runs by default is [`span_boundaries_do_not_change_any_region_value`] and
//! [`an_incomplete_staged_walk_is_refused_rather_than_finished`], on a 33-point
//! region, in 26 ms together. The step budget's refusal is deliberately not
//! restated here at gate cost: `tiler-compiler`'s
//! `the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget`
//! already spends 8.3 s per run watching it, on the region the governed lowering
//! actually emits, and a second 16,777,216-step refusal would buy time and
//! nothing else.
//!
//! # Where the digest comes from
//!
//! [`tiler_digest::DigestAlgorithm::digest_external_record`], reached through
//! this crate's development dependency on `tiler-digest`. The retained
//! `result_sha256` is an externally specified raw record — the probe's host
//! handed the output buffer to `CC_SHA256` — so it carries no Tiler domain and
//! its algorithm is fixed by the record rather than by this build's writer
//! policy. [ADR 0111](../../../docs/decisions/0111-separate-externally-specified-raw-hashes-from-governed-tiler-digests.md)
//! is what gave that subject a typed path and deleted the copy this test used to
//! carry; the variant is spelled `Sha256` rather than `GOVERNED` because the
//! record means SHA-256 permanently.

use std::time::Instant;

use tiler_digest::DigestAlgorithm;
use tiler_ir::index::{
    DomainRole, FrozenScalarRegistry, IndexInteger, IndexRegionBuilder, ScalarAttributes,
    TensorRole, VerifiedIndexRegion, add_f32_scalar_op, multiply_f32_scalar_op,
};
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32TensorContraction, InputKey, OutputKey,
    SemanticProgramBuilder,
};
use tiler_ir::shape::{Extent, Shape};
use tiler_reference::{
    FloatBitOrder, FrozenReferenceRegistry, FrozenScalarReferenceRegistry, IndexReferenceResource,
    IndexRegionAuthority, IndexRegionEvaluation, IndexRegionEvaluationError, IndexRegionEvaluator,
    IndexRegionInput, InputBinding, ReferenceElement, ReferenceEvaluator, ReferenceOperationError,
    StagedContractionError, StagedStrictTensorContractionF32, Tensor, TensorPayloadView,
};

/// The probe's workload seed, `contraction_probe.py`'s `WORKLOAD_SEED`.
const WORKLOAD_SEED: u64 = 0x5445_524D;

/// The probe's right-operand seed derivation, `host.m`'s `fill_prng` call.
const RIGHT_SEED_MASK: u64 = 0xA5A5_A5A5_A5A5_A5A5;

/// The reference's own fold bound, restated from `tiler-reference`'s limit.
///
/// Restated rather than imported because it is `pub(crate)`. The staged planner
/// derives its slab width from the real constant, so a bound that moved would
/// change `slab_count()` and this file's arithmetic assertions would notice.
const REFERENCE_STEP_LIMIT: usize = 16 * 1024 * 1024;

/// One cell of the L3 correctness profile, with its retained `direct` digest.
struct Cell {
    id: &'static str,
    m: u64,
    n: u64,
    k: u64,
    /// `output_count * contracted_count`, the fold the unstaged path is asked for.
    steps: usize,
    /// SHA-256 of the `direct` kernel's result bytes, little-endian `f32`,
    /// row-major, from the retained correctness record.
    result_sha256: &'static str,
}

/// All six correctness cells, in the record's own order.
const PROFILE_CELLS: [Cell; 6] = [
    Cell {
        id: "w_decode_kv",
        m: 1,
        n: 1024,
        k: 1024,
        steps: 1_048_576,
        result_sha256: "79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f",
    },
    Cell {
        id: "w_prefill_q",
        m: 10,
        n: 2048,
        k: 1024,
        steps: 20_971_520,
        result_sha256: "1c54f5cd7265ee288ec79bcd9254243b78a95d57c3c489e5ea90bcc4298073c0",
    },
    Cell {
        id: "w_prefill_mlp_in",
        m: 128,
        n: 3072,
        k: 1024,
        steps: 402_653_184,
        result_sha256: "eb382840ac9e533f57e51a0ffed2d61608664ecc5869aaa9f93afa3c312696a0",
    },
    Cell {
        id: "w_prefill_mlp_out",
        m: 128,
        n: 1024,
        k: 3072,
        steps: 402_653_184,
        result_sha256: "124571de47ebff2f152b120afc9944b3465bffe94d8ac283a077677f61feb5f5",
    },
    Cell {
        id: "w_prefill_o",
        m: 128,
        n: 1024,
        k: 2048,
        steps: 268_435_456,
        result_sha256: "b99eff9042d9e4b25e3844ff0462e5e6303e57b146aa79400622885bffc5f2f6",
    },
    Cell {
        id: "w_vocab_slice",
        m: 1,
        n: 8192,
        k: 1024,
        steps: 8_388_608,
        result_sha256: "88b01ae776f42bdb2f2d1092ddfd039e20e652d28393a6e2ec19e5cc1d9803c8",
    },
];

/// `td,od->to` over `[M, K] x [N, K] -> [M, N]`, the profile's index structure.
///
/// Spelled with the frontend's own labels rather than the canonical numbering, so
/// the structure constructor's renumbering is exercised rather than assumed.
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

fn splitmix64(x: u64) -> u64 {
    let x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let z = x;
    let z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The probe's operand value at one index: `m * 2^-24` with `m` in `[-2^23, 2^23)`.
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

/// Returns the probe's operand pair for one cell: `[M, K]` and `[N, K]`.
fn operands(cell: &Cell) -> (Tensor, Tensor) {
    (
        prng_tensor(Shape::from_dims([cell.m, cell.k]), WORKLOAD_SEED),
        prng_tensor(
            Shape::from_dims([cell.n, cell.k]),
            WORKLOAD_SEED ^ RIGHT_SEED_MASK,
        ),
    )
}

/// The probe's digest domain: little-endian `f32` bytes in row-major order.
fn digest_of(elements: &[ReferenceElement]) -> String {
    let bytes: Vec<u8> = elements
        .iter()
        .flat_map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
            .to_le_bytes()
        })
        .collect();
    sha256_hex(&bytes)
}

/// Folds one whole cell through the staged oracle, slab by slab.
///
/// This *is* the procedure the file claims covers every cell: plan once, ask the
/// planner how many slabs the bound admits, and walk them. The loop is the
/// authorization — no call inside it walks more steps than the unstaged path
/// would have been allowed to.
fn staged_result(
    structure: &ContractionIndexStructure,
    left: &Tensor,
    right: &Tensor,
) -> Vec<ReferenceElement> {
    let staged = StagedStrictTensorContractionF32::governed(structure, left, right)
        .expect("the governed contraction plans over the profile's operands");
    let mut elements = Vec::with_capacity(staged.output_count());
    for slab in 0..staged.slab_count() {
        elements.extend(
            staged
                .evaluate_slab(slab)
                .expect("every planned slab is admitted"),
        );
    }
    assert_eq!(
        elements.len(),
        staged.output_count(),
        "the slabs must cover the result exactly once"
    );
    elements
}

/// Evaluates one cell as a whole program, for its refusal or its value.
///
/// The allowance is the caller's, so one helper serves both halves of the claim:
/// at [`REFERENCE_STEP_LIMIT`] this is the ordinary evaluator every other consumer
/// gets, and at a stated larger number it is the same evaluator authorized to
/// spend more bounded windows on one occurrence.
fn program_result(
    cell: &Cell,
    left: &Tensor,
    right: &Tensor,
    iteration_step_allowance: usize,
) -> Result<Vec<ReferenceElement>, tiler_reference::EvaluationError> {
    let structure = projection_structure();
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the standard registry composes");
    let left_key = InputKey::new("activations").expect("a governed key");
    let right_key = InputKey::new("weights").expect("a governed key");
    let activations = builder
        .input::<F32>(left_key.clone(), Shape::from_dims([cell.m, cell.k]))
        .expect("the first operand is declared");
    let weights = builder
        .input::<F32>(right_key.clone(), Shape::from_dims([cell.n, cell.k]))
        .expect("the second operand is declared");
    let projected = F32TensorContraction::apply(&mut builder, &structure, activations, weights)
        .expect("the occurrence is well formed");
    builder
        .output(
            OutputKey::new("projected").expect("a governed key"),
            projected,
        )
        .expect("the result is named");
    let program = builder.build().expect("the program verifies");
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .with_iteration_step_allowance(iteration_step_allowance)
        .evaluate(
            &program,
            &[
                InputBinding::new(&left_key, left),
                InputBinding::new(&right_key, right),
            ],
        )?;
    let TensorPayloadView::Dense(elements) = outputs[0].payload() else {
        panic!("a contraction result is a dense f32 tensor")
    };
    Ok(elements.to_vec())
}

/// Compares one cell against its retained digest and reports the wall clock.
fn compare_cell(cell: &Cell) {
    let structure = projection_structure();
    let (left, right) = operands(cell);

    let staged = StagedStrictTensorContractionF32::governed(&structure, &left, &right)
        .expect("the governed contraction plans over the profile's operands");
    assert_eq!(
        staged.output_count() * staged.contracted_count(),
        cell.steps,
        "{}: the recomputed fold must be the one the record states",
        cell.id
    );
    assert!(
        staged.slab_output_count() * staged.contracted_count() <= REFERENCE_STEP_LIMIT,
        "{}: no planned slab may exceed the bound the unstaged fold is held to",
        cell.id
    );

    let started = Instant::now();
    let elements = staged_result(&structure, &left, &right);
    let elapsed = started.elapsed();
    assert_eq!(
        elements.len(),
        usize::try_from(cell.m * cell.n).expect("a profile cell's result is bounded"),
        "{}: every output element is produced",
        cell.id
    );
    assert_eq!(
        digest_of(&elements),
        cell.result_sha256,
        "{}: the staged reference does not reproduce the retained `direct` result",
        cell.id
    );
    println!(
        "{}: {} steps in {} slabs of {} outputs, {} ms ({} ns/step)",
        cell.id,
        cell.steps,
        staged.slab_count(),
        staged.slab_output_count(),
        elapsed.as_millis(),
        elapsed.as_nanos() / u128::try_from(cell.steps).expect("a step count fits in u128"),
    );
}

/// The digest helper reproduces both published FIPS 180-4 vectors.
///
/// Run before anything rests on it, and separately so a broken helper reports
/// itself rather than reporting six disagreeing cells.
#[test]
fn the_digest_helper_reproduces_the_published_vectors() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

/// The staged oracle reaches a cell the unstaged fold refuses, and agrees with
/// the device.
///
/// Two cells, and the pairing is the content. `w_decode_kv` is a control: the
/// unstaged path admits it, so a failure there is a broken harness rather than a
/// broken staging. `w_prefill_q` is the claim: the *same operands* are refused by
/// the unstaged evaluator under the work bound and folded by the staged one to
/// the SHA-256 an Apple M4 Max produced. A frozen golden could assert the second
/// half; only a live oracle can assert both against one pair of tensors.
#[test]
fn the_staged_oracle_reaches_the_cheapest_refused_cell() {
    the_digest_helper_reproduces_the_published_vectors();

    let control = &PROFILE_CELLS[0];
    assert_eq!(control.id, "w_decode_kv");
    assert!(control.steps <= REFERENCE_STEP_LIMIT);
    compare_cell(control);

    let refused = &PROFILE_CELLS[1];
    assert_eq!(refused.id, "w_prefill_q");
    assert!(refused.steps > REFERENCE_STEP_LIMIT);
    compare_cell(refused);

    // The protection, watched on the cell the staging just reached: the ordinary
    // whole-program path still refuses these operands, and refuses them by naming
    // the work bound rather than a storage one.
    let (left, right) = operands(refused);
    let error = program_result(refused, &left, &right, REFERENCE_STEP_LIMIT)
        .expect_err("the unstaged fold exceeds the reference's work bound");
    let message = format!("{error}");
    assert!(
        message.contains("iteration space has 20971520 steps, exceeding 16777216"),
        "the refusal must name the work bound and the exact fold: {message}"
    );
}

/// The whole-*program* path reaches the cell its default evaluator refuses.
///
/// The test above reaches `w_prefill_q` with no evaluator, program, or registry in
/// the picture — a caller holding two tensors and writing the slab loop itself.
/// This one reaches the same cell through [`ReferenceEvaluator::evaluate`], which
/// is what a consumer with a verified program in hand actually calls, and where
/// the fold's windows are the evaluator's own rather than the caller's.
///
/// **Three asks against one pair of operands, which is the content.** At the
/// default allowance the same refusal fires, unchanged and quoted. One step short
/// of the fold, a *stated* allowance refuses it too — so authorizing work is a
/// number that can still say no rather than a switch that turns the check off. At
/// the fold's own step count it evaluates, and what arrives is compared against
/// the SHA-256 an Apple M4 Max produced for these bytes.
///
/// The bit-identity claim is against a **different partition** of the same fold:
/// the evaluator spends two windows of 16,384 output elements and the comparison
/// spends seven slabs of 3,000, and the two payloads must agree element for
/// element. A comparison against a single-window fold is not available at this
/// size, because a 20,971,520-step fold has no single-window form — so what is
/// compared is two partitions and a device digest rather than one partition
/// against itself. [`slab_boundaries_do_not_change_any_folded_value`] is where a
/// single window and several are compared, at a size that admits both.
#[test]
fn a_whole_program_evaluation_reaches_the_cell_its_default_evaluator_refuses() {
    let cell = &PROFILE_CELLS[1];
    assert_eq!(cell.id, "w_prefill_q");
    assert!(cell.steps > REFERENCE_STEP_LIMIT);
    let structure = projection_structure();
    let (left, right) = operands(cell);

    // The evaluator every other consumer gets, unchanged.
    let refused = program_result(cell, &left, &right, REFERENCE_STEP_LIMIT)
        .expect_err("a default evaluator still refuses this fold");
    assert!(
        format!("{refused}").contains("iteration space has 20971520 steps, exceeding 16777216"),
        "the default refusal must be byte-for-byte the one it always was: {refused}"
    );

    // A stated allowance one step short of the fold, so the check is watched
    // saying no about the number it was actually given.
    let refused = program_result(cell, &left, &right, cell.steps - 1)
        .expect_err("a stated allowance below the fold refuses it");
    assert!(
        format!("{refused}").contains(&format!(
            "iteration space has {} steps, exceeding {}",
            cell.steps,
            cell.steps - 1
        )),
        "a stated allowance must name itself and the fold it declined: {refused}"
    );

    // The plan the evaluator walks, as a number rather than as an inference from
    // the result having arrived: `StagedStrictTensorContractionF32::governed` and
    // the registered operation take their window width from one place.
    let planned = StagedStrictTensorContractionF32::governed(&structure, &left, &right)
        .expect("the governed contraction plans");
    assert_eq!(planned.slab_output_count(), REFERENCE_STEP_LIMIT / 1024);
    assert_eq!(
        planned.slab_count(),
        2,
        "this fold needs more than one bounded window, which is what is being staged"
    );

    let started = Instant::now();
    let elements = program_result(cell, &left, &right, cell.steps)
        .expect("the fold's own step count admits it");
    let elapsed = started.elapsed();
    assert_eq!(
        digest_of(&elements),
        cell.result_sha256,
        "the whole-program staged fold does not reproduce the retained `direct` result"
    );

    // A seven-slab partition of the identical fold, so the two windows above are
    // established as unobservable in the values at this cell's own size.
    let alternative = StagedStrictTensorContractionF32::governed_with_slab_output_count(
        &structure, &left, &right, 3_000,
    )
    .expect("3,000 output elements per slab is under the work bound");
    assert_eq!(alternative.slab_count(), 7);
    let mut staged = Vec::with_capacity(alternative.output_count());
    for slab in 0..alternative.slab_count() {
        staged.extend(
            alternative
                .evaluate_slab(slab)
                .expect("every planned slab is admitted"),
        );
    }
    assert_eq!(
        elements, staged,
        "the window partition changed a folded value"
    );

    println!(
        "{}: {} steps through the whole-program path in {} windows, {} ms",
        cell.id,
        cell.steps,
        planned.slab_count(),
        elapsed.as_millis(),
    );
}

/// Every one of the six cells reproduces its retained digest.
///
/// `#[ignore]`d for cost, not for doubt: the six folds total 1,104,150,528 steps.
/// The module documentation records the invocation and the outcome of the run
/// this file was landed with.
#[test]
#[ignore = "1.1e9 exact scalar fold steps; run deliberately, see the module documentation"]
fn the_staged_oracle_reproduces_every_retained_profile_digest() {
    the_digest_helper_reproduces_the_published_vectors();
    for cell in &PROFILE_CELLS {
        compare_cell(cell);
    }
}

/// Slab boundaries are unobservable in the folded values.
///
/// The registered signature's argument is in
/// [`StagedStrictTensorContractionF32`]'s documentation; this is its executable
/// half. Five partitions of one `[3, 7] x [11, 7] -> [3, 11]` result — including
/// a width of one, a width that divides the output exactly, two that do not, and
/// one wider than the whole result — produce the identical bit patterns, and the
/// unstaged evaluator produces them too.
///
/// The perturbation is what stops this from passing vacuously: with one operand
/// element advanced by a single representable value, every partition must move,
/// and must still agree. Without it an implementation returning a constant, or
/// one whose slabs were all empty, would satisfy the equalities above.
#[test]
fn slab_boundaries_do_not_change_any_folded_value() {
    let structure = projection_structure();
    let cell = Cell {
        id: "slab-equivalence",
        m: 3,
        n: 11,
        k: 7,
        steps: 231,
        result_sha256: "",
    };
    let (left, right) = operands(&cell);

    let baseline = program_result(&cell, &left, &right, REFERENCE_STEP_LIMIT)
        .expect("a small fold is admitted");
    assert!(
        baseline.windows(2).any(|pair| pair[0] != pair[1]),
        "a degenerate constant result would satisfy every equality below"
    );

    for width in [1_usize, 3, 5, 11, 64] {
        let staged = StagedStrictTensorContractionF32::governed_with_slab_output_count(
            &structure, &left, &right, width,
        )
        .expect("every one of these widths is under the work bound");
        let mut elements = Vec::new();
        for slab in 0..staged.slab_count() {
            elements.extend(staged.evaluate_slab(slab).expect("a planned slab"));
        }
        assert_eq!(
            elements, baseline,
            "a slab width of {width} changed a folded value"
        );
    }

    // One contributing element advanced by one representable value.
    let TensorPayloadView::Dense(elements) = left.payload() else {
        panic!("a dense operand")
    };
    let mut perturbed = elements.to_vec();
    let last = perturbed.len() - 1;
    let bits = u32::from_be_bytes(
        <[u8; 4]>::try_from(perturbed[last].as_bytes()).expect("an f32 element is four bytes"),
    );
    perturbed[last] = ReferenceElement::from_float_bits(
        (bits + 1).to_be_bytes(),
        FloatBitOrder::MostSignificantByteFirst,
    )
    .expect("the perturbed pattern is four bytes");
    let left = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([cell.m, cell.k]),
        perturbed,
    )
    .expect("the perturbed operand is well formed");

    let moved = program_result(&cell, &left, &right, REFERENCE_STEP_LIMIT)
        .expect("a small fold is admitted");
    assert_ne!(moved, baseline, "the fold ignored a contributing element");
    for width in [1_usize, 3, 5, 11, 64] {
        let staged = StagedStrictTensorContractionF32::governed_with_slab_output_count(
            &structure, &left, &right, width,
        )
        .expect("every one of these widths is under the work bound");
        let mut elements = Vec::new();
        for slab in 0..staged.slab_count() {
            elements.extend(staged.evaluate_slab(slab).expect("a planned slab"));
        }
        assert_eq!(
            elements, moved,
            "a slab width of {width} changed a perturbed folded value"
        );
    }
}

/// A slab the work bound does not admit is refused, not narrowed and not walked.
///
/// This is the protection the staging leaves standing, watched on the path that
/// can reach it. The planner's own width is admissible by construction, so the
/// case that discriminates is a caller asking for a wider one: at `k = 1024` the
/// bound admits 16,384 output elements per slab, and 16,385 is refused under the
/// work bound's own variant carrying the exact fold it declined.
///
/// The fixture costs two small operands because every refusal here lands during
/// planning, before a step is taken.
#[test]
fn a_slab_wider_than_the_work_bound_admits_is_refused() {
    let structure = projection_structure();
    let cell = Cell {
        id: "slab-refusal",
        m: 64,
        n: 64,
        k: 1024,
        steps: 4_194_304,
        result_sha256: "",
    };
    let (left, right) = operands(&cell);

    let planned = StagedStrictTensorContractionF32::governed(&structure, &left, &right)
        .expect("the governed contraction plans");
    assert_eq!(planned.contracted_count(), 1024);
    assert_eq!(planned.slab_output_count(), REFERENCE_STEP_LIMIT / 1024);

    // The widest admitted width, so the refusal below discriminates the bound
    // rather than the fixture.
    assert!(
        StagedStrictTensorContractionF32::governed_with_slab_output_count(
            &structure, &left, &right, 16_384,
        )
        .is_ok()
    );
    assert_eq!(
        StagedStrictTensorContractionF32::governed_with_slab_output_count(
            &structure, &left, &right, 16_385,
        )
        .unwrap_err(),
        StagedContractionError::Operation(ReferenceOperationError::IterationStepsExceeded {
            limit: REFERENCE_STEP_LIMIT,
            actual: 16_385 * 1024,
        })
    );
    assert_eq!(
        StagedStrictTensorContractionF32::governed_with_slab_output_count(
            &structure, &left, &right, 0,
        )
        .unwrap_err(),
        StagedContractionError::Operation(ReferenceOperationError::InvalidApplication)
    );
    // A slab index past the plan is refused rather than folding a short window or
    // reading past the result.
    assert_eq!(
        planned.evaluate_slab(planned.slab_count()),
        Err(ReferenceOperationError::InvalidApplication)
    );
}

// The second oracle: the same cells through the verified index region.

/// Builds the region `tiler_compiler::governed`'s contraction lowering emits.
///
/// A hand-written mirror for the same reason `index_region_oracle.rs`'s
/// `governed` module states: `tiler-reference` is a dependency of
/// `tiler-compiler` and cannot import it, and inverting that edge would put the
/// reference oracle downstream of the compiler. It follows
/// `GovernedStrictTensorContractionF32::lower` step for step for a contracted
/// space of rank one, which is what every cell of this profile has:
///
/// - the parallel domain is the output shape `[m, n]`, in that order;
/// - the accumulator seeds at the product at contracted offset zero, never at
///   `+0.0` — the family declares no seed, and the two differ observably where
///   every product is `-0.0`;
/// - the reduction runs over a tail dimension of `k - 1` whose contributor is the
///   product at offset `tail + 1`, which for a rank-one contracted space is the
///   contracted coordinate itself (`decode_contracted` neither wraps nor divides
///   at position zero with stride one);
/// - the product and the sum are separate governed applications, so each rounds
///   separately; and
/// - no result-boundary canonicalization is emitted, because every value the
///   region can commit is already a governed multiply's or add's result.
fn contraction_region(
    scalars: &FrozenScalarRegistry,
    cell: &Cell,
) -> Result<VerifiedIndexRegion, Box<dyn std::error::Error>> {
    assert!(
        cell.k > 1,
        "a rank-one contracted space of one point folds nothing, and this mirror does not emit the governed singleton form"
    );
    let f32_type = F32::resolved_type();
    let mut builder = IndexRegionBuilder::new(scalars.clone())?;
    let t = builder.dimension(DomainRole::Parallel, Extent::new(cell.m))?;
    let o = builder.dimension(DomainRole::Parallel, Extent::new(cell.n))?;
    let left = builder.tensor(
        TensorRole::Input,
        f32_type.clone(),
        Shape::from_dims([cell.m, cell.k]),
    )?;
    let right = builder.tensor(
        TensorRole::Input,
        f32_type.clone(),
        Shape::from_dims([cell.n, cell.k]),
    )?;
    let out = builder.tensor(
        TensorRole::Output,
        f32_type,
        Shape::from_dims([cell.m, cell.n]),
    )?;
    let row = builder.dimension_expr(t)?;
    let column = builder.dimension_expr(o)?;

    let zero = builder.constant(IndexInteger::from_u64(0))?;
    let seed_left = builder.read(left, &[t, o], &[row, zero])?;
    let seed_right = builder.read(right, &[t, o], &[column, zero])?;
    let seed = builder
        .apply(
            multiply_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[seed_left, seed_right],
        )?
        .get(0)
        .ok_or("the governed multiply produces one result")?;

    let tail = builder.dimension(DomainRole::Reduction, Extent::new(cell.k - 1))?;
    let induction = builder.dimension_expr(tail)?;
    let one = IndexInteger::from_u64(1);
    let offset = builder.linear_combination(one.clone(), &[(one, induction)])?;
    let tail_left = builder.read(left, &[t, o, tail], &[row, offset])?;
    let tail_right = builder.read(right, &[t, o, tail], &[column, offset])?;
    let contributor = builder
        .apply(
            multiply_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[tail_left, tail_right],
        )?
        .get(0)
        .ok_or("the governed multiply produces one result")?;

    let folded = builder
        .reduce(&[tail], &[seed], &[contributor], |body| {
            let accumulated = body.apply(
                add_f32_scalar_op(),
                ScalarAttributes::empty(),
                &[
                    body.state(0).expect("one state parameter"),
                    body.contributor(0).expect("one contributor parameter"),
                ],
            )?;
            body.yield_values(&[accumulated
                .get(0)
                .expect("the governed add produces one result")])
        })?
        .get(0)
        .ok_or("the reduction produces one result")?;
    let write = builder.write(out, &[t, o], &[row, column])?;
    builder.output(write, folded)?;
    Ok(builder.build()?)
}

fn region_evaluator() -> IndexRegionEvaluator {
    IndexRegionEvaluator::new(
        FrozenReferenceRegistry::standard().expect("the governed value profile composes"),
        FrozenScalarReferenceRegistry::standard().expect("the governed scalar oracle composes"),
    )
}

fn region_inputs<'a>(
    region: &VerifiedIndexRegion,
    left: &'a Tensor,
    right: &'a Tensor,
) -> Vec<IndexRegionInput<'a>> {
    let ids: Vec<_> = region
        .tensors()
        .filter(|tensor| tensor.role() == TensorRole::Input)
        .map(tiler_ir::index::TensorRef::id)
        .collect();
    vec![
        IndexRegionInput::new(ids[0], left),
        IndexRegionInput::new(ids[1], right),
    ]
}

/// Returns the dense elements of one region evaluation's single output.
fn region_elements(evaluation: &IndexRegionEvaluation) -> Vec<ReferenceElement> {
    let TensorPayloadView::Dense(elements) = evaluation.outputs()[0].payload() else {
        panic!("a contraction region writes a dense f32 output")
    };
    elements.to_vec()
}

/// Walks one region in spans of `span` root points and returns its output.
///
/// This *is* the staged procedure: stage once, then loop until the walk reports
/// it is done. The loop is the authorization — no call inside it is allowed more
/// steps than the whole-region path would have been.
///
/// A contraction region has one write root over the whole parallel dimension
/// set, so its root points and its parallel points are the same points; the
/// staged surface is stated in the former because a partitioned region's roots
/// need not agree on a domain.
fn staged_region_result(
    region: &VerifiedIndexRegion,
    scalars: &FrozenScalarRegistry,
    left: &Tensor,
    right: &Tensor,
    span: u64,
) -> Vec<ReferenceElement> {
    let evaluator = region_evaluator();
    let inputs = region_inputs(region, left, right);
    let mut staged = evaluator
        .stage(region, IndexRegionAuthority::new(scalars), &inputs)
        .expect("the governed authority admits the region");
    let expected = staged
        .root_point_count()
        .expect("a profile cell's root points are counted");
    while staged
        .evaluate_root_points(span)
        .expect("every span of this width is under the step budget")
        > 0
    {}
    assert_eq!(
        staged.evaluated_root_points(),
        expected,
        "the spans must cover the root points exactly once"
    );
    region_elements(&staged.finish().expect("the walked region finishes"))
}

/// Span boundaries are unobservable in the values a region commits.
///
/// The executable half of the argument in `StagedIndexRegionEvaluation`'s
/// documentation. Five partitions of one `[3, 7] x [11, 7] -> [3, 11]` region —
/// a width of one, a width dividing the 33 root points exactly, two
/// that do not, and one wider than the whole space — commit identical bit
/// patterns, and the whole-region path commits them too.
///
/// The cross-check against `program_result` is the part a second implementation
/// of the same arithmetic could not fake: the region and the registered
/// contraction reach the same bits by different code.
///
/// The perturbation is what stops this passing vacuously. With one operand
/// element advanced by a single representable value every partition must move
/// and must still agree; without it, a region evaluator returning a constant, or
/// one whose spans walked nothing, would satisfy every equality above.
#[test]
fn span_boundaries_do_not_change_any_region_value() {
    let scalars = FrozenScalarRegistry::standard().expect("the governed scalar authority composes");
    let cell = Cell {
        id: "span-equivalence",
        m: 3,
        n: 11,
        k: 7,
        steps: 231,
        result_sha256: "",
    };
    let region = contraction_region(&scalars, &cell).expect("the mirrored region verifies");
    let (left, right) = operands(&cell);

    let baseline = program_result(&cell, &left, &right, REFERENCE_STEP_LIMIT)
        .expect("a small fold is admitted");
    assert!(
        baseline.windows(2).any(|pair| pair[0] != pair[1]),
        "a degenerate constant result would satisfy every equality below"
    );
    for span in [1_u64, 3, 5, 11, 64] {
        assert_eq!(
            staged_region_result(&region, &scalars, &left, &right, span),
            baseline,
            "a span of {span} root points changed a committed value"
        );
    }

    let TensorPayloadView::Dense(elements) = left.payload() else {
        panic!("a dense operand")
    };
    let mut perturbed = elements.to_vec();
    let last = perturbed.len() - 1;
    let bits = u32::from_be_bytes(
        <[u8; 4]>::try_from(perturbed[last].as_bytes()).expect("an f32 element is four bytes"),
    );
    perturbed[last] = ReferenceElement::from_float_bits(
        (bits + 1).to_be_bytes(),
        FloatBitOrder::MostSignificantByteFirst,
    )
    .expect("the perturbed pattern is four bytes");
    let left = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([cell.m, cell.k]),
        perturbed,
    )
    .expect("the perturbed operand is well formed");

    let moved = program_result(&cell, &left, &right, REFERENCE_STEP_LIMIT)
        .expect("a small fold is admitted");
    assert_ne!(moved, baseline, "the region ignored a contributing element");
    for span in [1_u64, 3, 5, 11, 64] {
        assert_eq!(
            staged_region_result(&region, &scalars, &left, &right, span),
            moved,
            "a span of {span} root points changed a perturbed committed value"
        );
    }
}

/// A staged walk that does not cover its root points cannot be finished.
///
/// Two caller errors that would otherwise produce a partial result wearing a
/// whole one's type. A span of no points is refused where it is asked for, and a
/// walk stopped early is refused at `finish` — before the output planner's own
/// `IncompleteWrite` would have reported the same gap as a region defect, which
/// it is not.
#[test]
fn an_incomplete_staged_walk_is_refused_rather_than_finished() {
    let scalars = FrozenScalarRegistry::standard().expect("the governed scalar authority composes");
    let cell = Cell {
        id: "span-refusal",
        m: 3,
        n: 11,
        k: 7,
        steps: 231,
        result_sha256: "",
    };
    let region = contraction_region(&scalars, &cell).expect("the mirrored region verifies");
    let (left, right) = operands(&cell);
    let evaluator = region_evaluator();
    let inputs = region_inputs(&region, &left, &right);
    let mut staged = evaluator
        .stage(&region, IndexRegionAuthority::new(&scalars), &inputs)
        .expect("the governed authority admits the region");

    assert_eq!(staged.root_point_count(), Some(33));
    assert_eq!(
        staged.evaluate_root_points(0),
        Err(IndexRegionEvaluationError::EmptyStagedSpan),
        "a span that walks nothing is refused at the call that asked for it"
    );
    assert_eq!(staged.evaluate_root_points(30), Ok(30));
    assert!(!staged.is_exhausted());
    assert_eq!(
        staged
            .finish()
            .expect_err("three root points remain, so there is no whole result to return"),
        IndexRegionEvaluationError::IncompleteStagedWalk { evaluated: 30 },
    );

    // The neighbour: the same walk completed. Without it the refusal above would
    // be consistent with a `finish` that never returns a result at all.
    let mut staged = evaluator
        .stage(&region, IndexRegionAuthority::new(&scalars), &inputs)
        .expect("the governed authority admits the region");
    assert_eq!(staged.evaluate_root_points(30), Ok(30));
    assert_eq!(staged.evaluate_root_points(30), Ok(3));
    assert_eq!(staged.evaluate_root_points(30), Ok(0));
    assert!(staged.is_exhausted());
    assert_eq!(
        region_elements(&staged.finish().expect("the covered walk finishes")),
        program_result(&cell, &left, &right, REFERENCE_STEP_LIMIT)
            .expect("a small fold is admitted"),
    );
}

/// The staged region oracle reaches the vocabulary cell, which one span refuses.
///
/// The cell this file's sibling boundary statement named as out of reach.
/// `w_vocab_slice`'s region walks 8,192 root points, each folding 1,023
/// contributors through separate governed scalar applications, and the whole walk
/// costs far more than `MAX_EVALUATION_STEPS` admits in one span — so
/// `IndexRegionEvaluator::evaluate` refuses it, and
/// `tiler-compiler`'s `the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget`
/// asserts that refusal on the region the governed lowering actually emits.
///
/// The pairing is the content: **the same region and the same operands** are
/// refused in one span and walked in spans of 512 root points to a result
/// whose SHA-256 is the one an Apple M4 Max produced for the `direct` kernel. A
/// frozen golden could assert the second half; only a live oracle can assert both
/// against one region.
///
/// `#[ignore]`d for cost, not for doubt — see the module documentation for the
/// invocation and the measured run.
#[test]
#[ignore = "~64 s of exact scalar region evaluation at 516 ns a step; run deliberately, see the module documentation"]
fn the_staged_index_region_oracle_reaches_the_vocabulary_cell() {
    the_digest_helper_reproduces_the_published_vectors();
    let cell = &PROFILE_CELLS[5];
    assert_eq!(cell.id, "w_vocab_slice");
    let scalars = FrozenScalarRegistry::standard().expect("the governed scalar authority composes");
    let region = contraction_region(&scalars, cell).expect("the mirrored region verifies");
    let (left, right) = operands(cell);

    // The protection, watched on the cell the staging reaches: one span over the
    // whole walk is still refused, under the step budget's own variant
    // carrying the exact step it declined at rather than another resource.
    let inputs = region_inputs(&region, &left, &right);
    let refused = Instant::now();
    let error = region_evaluator()
        .evaluate(&region, IndexRegionAuthority::new(&scalars), &inputs)
        .expect_err("one span over this region exceeds the step budget");
    let refusal = refused.elapsed();
    assert_eq!(
        error,
        IndexRegionEvaluationError::ResourceExceeded {
            resource: IndexReferenceResource::EvaluationSteps,
            limit: 16_777_216,
            actual: 16_777_217,
        },
        "the refusal must name the step budget at its first excess step"
    );

    let started = Instant::now();
    let elements = staged_region_result(&region, &scalars, &left, &right, 512);
    let elapsed = started.elapsed();
    assert_eq!(
        elements.len(),
        usize::try_from(cell.m * cell.n).expect("a profile cell's result is bounded"),
        "every output element is committed"
    );
    assert_eq!(
        digest_of(&elements),
        cell.result_sha256,
        "the staged region does not reproduce the retained `direct` result"
    );
    println!(
        "{}: one span refused after 16777216 steps in {} ms ({} ns/step); \
         8192 root points walked in 16 spans of 512 in {} ms",
        cell.id,
        refusal.as_millis(),
        refusal.as_nanos() / 16_777_216,
        elapsed.as_millis(),
    );
}

/// Reproduces the retained record's raw SHA-256 over `message`.
///
/// One line over [`tiler_digest`], kept as a named helper because the call sites
/// above read as digests of a subject rather than as algorithm selection. The
/// variant is spelled explicitly: [`DigestAlgorithm::GOVERNED`] tracks whatever
/// this build of Tiler writes, while the retained record means SHA-256.
fn sha256_hex(message: &[u8]) -> String {
    DigestAlgorithm::Sha256
        .digest_external_record(message)
        .label()
}
