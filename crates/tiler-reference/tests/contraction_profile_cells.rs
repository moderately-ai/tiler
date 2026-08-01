//! The reference as bit-exact oracle for all six L3 contraction profile cells.
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
//! run, in 0.31 s. [`the_staged_oracle_reproduces_every_retained_profile_digest`]
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
//! # Why the digest helper is written out here
//!
//! `sha2` is a workspace dependency, but adding it to this crate would edit
//! `Cargo.lock`, which this work does not own. The implementation below is
//! therefore local to this test and is checked against the two published FIPS
//! 180-4 vectors before any comparison rests on it — a digest function that
//! silently computed something else would make every retained-value assertion
//! agree with itself.

use std::fmt::Write as _;
use std::time::Instant;

use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32TensorContraction, InputKey, OutputKey,
    SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, ReferenceOperationError,
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

/// Evaluates one cell through the ordinary unstaged path, for its refusal or value.
fn unstaged_result(
    cell: &Cell,
    left: &Tensor,
    right: &Tensor,
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
    let error = unstaged_result(refused, &left, &right)
        .expect_err("the unstaged fold exceeds the reference's work bound");
    let message = format!("{error}");
    assert!(
        message.contains("iteration space has 20971520 steps, exceeding 16777216"),
        "the refusal must name the work bound and the exact fold: {message}"
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

    let baseline = unstaged_result(&cell, &left, &right).expect("a small fold is admitted");
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

    let moved = unstaged_result(&cell, &left, &right).expect("a small fold is admitted");
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

/// FIPS 180-4 SHA-256 over a byte string, as lowercase hexadecimal.
///
/// Local to this test for the reason the module documentation states, and checked
/// against the two published vectors before anything rests on it.
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
    assert!(
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
