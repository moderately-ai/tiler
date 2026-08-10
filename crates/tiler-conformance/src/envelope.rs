#![cfg_attr(
    not(target_os = "macos"),
    allow(
        dead_code,
        reason = "everything downstream of opening a published envelope has `apple` as its only caller, and an envelope needs the offline Apple toolchain to exist at all: reading the pair and its sidecar, binding and placing the declared interface, compiling the declared shape to name the packaged program, the live-device requirement adapter, and the eight fail-closed probes with their subject and failure vocabulary. They are compiled on every host rather than gated, so that the recognizers, the classifications, and the comparisons a device merely supplies numbers to keep running where the workspace's tests do; a gate would move them onto hardware and shrink what a non-Apple host is held to. Named here rather than at each of the thirty-odd items because the reason is one reason, and stated under `not(target_os = \"macos\")` so an item that becomes genuinely unused is still a red build on the host that does use it."
    )
)]

//! The artifact-delivered route: dispatching from a published envelope alone.
//!
//! # Why this exists beside `crate::serial_sum`
//!
//! `crate::serial_sum` compiles a program in this process, emits it, links it,
//! and dispatches those in-memory bytes. Nothing is packaged, encoded, decoded,
//! or validated, and it is evidence about the *compiler and the emitter*.
//!
//! This module dispatches from an artifact written to a file and read back:
//! `tiler-runtime` decodes it, discharges every host obligation, commits a
//! route, and the device loads the object bytes *the envelope carries*. The
//! entry symbol, the argument-table index of every buffer, the bytes each must
//! reach, and the launch geometry all come from the decoded dispatch record. It
//! is evidence about the *delivery mechanism*.
//!
//! **The file is written by [`crate::publication`], in this run**, and the
//! distinction that survives is the one that was always load-bearing: the route
//! takes every fact from the decoded envelope rather than from compiler state
//! this process is holding. The object the device loads is the one the envelope
//! carries, and the only thing compiled on the routing side is used to *name* the
//! packaged program by canonical identity — never to supply a symbol, a slot, a
//! byte window, or a launch extent. What it no longer establishes is agreement
//! between two independently maintained halves; that claim lives in the
//! `prototypes/serial-sum-compile` and `prototypes/serial-sum-run` pair, and
//! `crate::publication`'s own header states what was traded for putting this
//! module into the gate.
//!
//! Keeping both is what makes a disagreement diagnosable. If the direct runs
//! match the reference and these do not, the envelope is at fault; if both fail
//! together, the compiler is. Collapsing them would leave only "the bits are
//! wrong".
//!
//! # This is producer-declared equality, NOT host-earned eligibility
//!
//! Read that literally. [`declared_route_environment`] states the profile
//! `tiler-build` *declares* for this Metal target; nothing about this host
//! earned the right to offer it. `ExecutionEnvironment::classify` therefore
//! answers a real question — does this artifact name the profile the producer
//! declared, under the same exact descriptor — and does not answer the question
//! ADR 0086 gates. `crate::applicability` is where that second question is
//! asked, and it refuses.
//!
//! # Where the routing commit falls
//!
//! ADR 0051 permits a fallback only before the commit, so every question this
//! host can answer about whether it can *carry out* a route is answered while
//! the `Preflight` is still held. [`plan_route`] resolves each routed ABI slot to
//! storage this run can supply and refuses a launch that covers no threads; the
//! device stage then discharges the library, the function, the pipeline, the
//! launch capacity, and the allocations; only then is `commit` called. A refusal
//! after the commit is a failure reported, never a fallback taken. There is no
//! fallback path here at all.
//!
//! # Where the envelope comes from, and the one boundary that remains
//!
//! [`crate::publication`] writes it, in this run, into a private directory it
//! then removes. The routed runs below therefore have exactly one boundary
//! rather than two: an envelope needs the same offline Apple toolchain a device
//! result needs, so a host that cannot measure equally cannot publish, and both
//! facts arrive as a single [`crate::measurement::Measured::Unavailable`] naming
//! what was missing. It never skips silently, it never claims a route it did not
//! take, and with `TILER_REQUIRE_METAL_CONFORMANCE` set the unavailability is a
//! failure.
//!
//! The ambient input `TILER_CONFORMANCE_ARTIFACT_BASE` used to name a base a
//! separate executable had published beneath, and it is **retired** rather than
//! kept as an override: nothing in `make full` set it, so every routed run below
//! reported its boundary unavailable and only the device-free half ran, and an
//! override nothing exercises is a second unexercised path rather than a
//! capability. `prototypes/serial-sum-compile` still publishes for
//! `prototypes/serial-sum-run` across a file interface no code crosses; it is no
//! longer this crate's input.
//!
//! What still runs on a host that cannot measure is everything decidable without
//! an envelope: the interface recognizers and every way of missing them, the
//! sidecar payload-length refusal, the routed dtype rows, the member and class
//! pins, the digest helper against its published vectors, the retained
//! comparison's two verdicts including the perturbation it must refuse, the six
//! correctness cells checked against the retained record's own `direct` rows,
//! the derivation that decides which of them a sidecar can carry, and the split
//! between the ordinary gate's cells and the `#[ignore]`d run's.
//!
//! # Which cells this routes, and the two bounds that decide it
//!
//! The retained record states six correctness cells and this module routes five.
//! Two independent bounds decide the shape of what runs:
//!
//! - **`tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES`** admits one case payload
//!   of 16 MiB, and `w_vocab_slice`'s `[8192, 1024]` weights operand is exactly
//!   twice that. No arrangement here reaches it — the constant is another crate's
//!   public surface — so the cell is pinned, excluded, and held to that
//!   arithmetic by a test rather than to a sentence.
//! - **The reference evaluator's per-occurrence fold bound** decides which of the
//!   remaining five the *ordinary gate* publishes. `w_decode_kv` folds under it
//!   and is published by the evaluator every other consumer gets; the four
//!   prefill cells need a stated allowance and 1,094,713,344 steps, so they are
//!   routed by an `#[ignore]`d run measured at 30.8 s.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ArtifactCodecFailure, AvailabilityPhase, BackendKey, BindingTarget,
    RecordedArtifactProgramIdentity, RepresentationKey, RouteRequirement, RouteResourceDimension,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
};
use tiler_artifact::proof::{
    DecodedProofSidecar, ProofAssociationError, ProofCaseRef, ProofCodecError, decode_proof_sidecar,
};
use tiler_build::{BoundMetalCompileDeclaration, DTypeDispatchability};
use tiler_compiler::session::{Compilation, NumericalContract};
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32TensorContraction, InputKey, OutputKey,
    SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_metal::applicability::{MetalGpuFamily, MetalGpuFamilySupport};
use tiler_runtime::load::{
    DTypeDispatch, DecodedProgram, ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest,
    LoadRejection, Preflight, VariantIneligibility,
};

use crate::applicability::ProbedGpuFamily;
use crate::device_preflight::PreflightRefusal;
use crate::serial_sum::{F32_BYTES, INPUT_KEY, OUTPUT_KEY, compile_under, serial_sum_program};

/// The aggregate executed records constraining IEEE F32's conformance-evidence
/// ledger cell.
///
/// The stable identifiers name the migrations whose retained runs this module
/// routes. The declaration is data for `crate::ledger`'s private comparison;
/// it assigns no maturity or evidence class.
pub(crate) const LEDGER_CELL: crate::ledger::CellDeclaration = crate::ledger::CellDeclaration {
    cell: crate::ledger::ConformanceCell::F32,
    run_ids: &[
        "carry-device-executed-value-proof@0f948637",
        "route-five-l3-realization-cells@2026-08-07",
    ],
    operation_extent: "serial-sum and contraction device runs (30 routed cases plus five retained L3 cells)",
    environment: crate::ledger::EnvironmentRow::APPLE9_2026_08_07,
    measured_half: crate::ledger::MeasuredHalf::Ran,
    composition: crate::ledger::CompositionExtent::RoutedArtifact,
};

/// The one delivery position every artifact here is built for.
///
/// A delivery position is the ordered slot a consumer's build target resolves
/// to, and these artifacts are built for a single target, so the sole position
/// is zero. Named rather than written as a bare `0` at each call, because the
/// argument decides *which compiled object* is loaded and a literal there says
/// nothing about why that one.
const SOLE_DELIVERY: usize = 0;

/// Governed backend family key this host executes.
pub(crate) const BACKEND_KEY: &str = "tiler.metal";
/// Governed executable-representation key this host consumes.
pub(crate) const REPRESENTATION_KEY: &str = "metallib";

/// Suffix appended to a member path to name its proof-case sidecar.
///
/// One constant, reached by [`sidecar_path`] from both halves — the publication
/// that writes the record and [`read_artifact`] that opens it. It used to be a
/// pinned pair against a separate executable's own spelling, because that
/// executable wrote `.proof` while the consumer still opened `.identity` for a
/// whole commit and no compilation could see it. Producing in process replaces
/// the pair with a shared derivation, which is the stronger arrangement: there is
/// no longer a second spelling to drift.
pub(crate) const SIDECAR_SUFFIX: &str = ".proof";

/// Returns the proof-case sidecar path beside one envelope path.
///
/// Derived by appending rather than by replacing an extension, so the two names
/// cannot collide with each other and the pair stays obviously one unit on disk.
pub(crate) fn sidecar_path(envelope: &Path) -> PathBuf {
    let mut name = envelope.as_os_str().to_owned();
    name.push(SIDECAR_SUFFIX);
    PathBuf::from(name)
}

/// The reduction classes published and routed, as `(name, reduced extent)`.
///
/// Three programs, not three operand sets. The reduced extent lives in the input
/// shape, so it changes the semantic graph, the verified kernels, and the
/// artifact identity; an empty domain and a singleton cannot be reached by
/// choosing different numbers for a fixed shape.
///
/// The empty domain leads, because it is the boundary the other two cannot speak
/// for: a reduction over zero contributors reads its input buffer never, and its
/// result is a reduction's identity element rather than a sum. The singleton is
/// where a serial reduction reduces in every order and so cannot observe an
/// ordering defect at all, which is what makes the nontrivial class mean
/// something.
pub(crate) const REDUCTION_CLASSES: [(&str, u64); 3] =
    [("empty-domain", 0), ("singleton", 1), ("nontrivial", 3)];

/// The plan roles published for each reduction class.
///
/// `selected` is whatever the portfolio ranks first, which is the fused plan on
/// this profile; `materialized` is the retained alternative that dispatches two
/// stages through one intermediate. Publishing both is the point of the matrix:
/// the two are different programs on the device and must agree bit for bit.
pub(crate) const PLAN_ROLES: [&str; 2] = ["selected", "materialized"];

/// Class name of the published contraction member.
///
/// Its `2 x 2` result has more than one row *and* more than one column, which is
/// what makes the two operand access relations — `(t, o, d) -> (t, d)` never
/// mentioning `o`, and `(t, o, d) -> (o, d)` never mentioning `t` — separately
/// observable. [`L3_CORRECTNESS_CELLS`] arrive as further members rather than as
/// a move of this one for exactly that reason: every cell of the profile is
/// `M = 1` or has `M != N`, so none of them can separate those two relations.
pub(crate) const CONTRACTION_CLASS: &str = "contraction";

/// Interface key of the contraction's first operand, `[M, K]`.
pub(crate) const CONTRACTION_ACTIVATIONS_KEY: &str = "activations";
/// Interface key of the contraction's second operand, `[N, K]`.
pub(crate) const CONTRACTION_WEIGHTS_KEY: &str = "weights";
/// Interface key of the contraction's one output, `[M, N]`.
pub(crate) const CONTRACTION_OUTPUT_KEY: &str = "projected";

/// One cell of the L3 correctness profile, with the digest a device measured for
/// its `direct` realization.
///
/// **Every digest below is a measurement, not a constant this workspace
/// derived.** All six were recorded on an Apple M4 Max under macOS 27.0
/// `26A5388g`, Xcode 26.6 `17F113`, SDK 26.5 `25F70`, and the offline Metal
/// compiler `32023.883`, by `spikes/scheduling/metal_contraction_vertical`, and
/// they live in that spike's `workload.tsv` under
/// [`crate::retained_record::RECORD_DIRECTORY`]. They stay literals here and are
/// held against that file by
/// `tests::the_pinned_cells_are_the_retained_records_own_direct_rows`, so a
/// transcription defect is a red test on every host rather than an agreement
/// nobody checked; reading them from the record and comparing them to themselves
/// would remove the pin instead of checking it.
///
/// The digest domain is the probe's own — little-endian `f32` bytes in row-major
/// order, exactly the buffer the probe's host handed to `CC_SHA256` — so
/// [`result_digest`] reproduces it from readback bit patterns without an
/// intervening shape or dtype.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct L3CorrectnessCell {
    /// The retained record's own identifier for the cell.
    pub(crate) id: &'static str,
    /// The class name the cell is published and opened under.
    ///
    /// Carried rather than derived from [`Self::id`], because a class name is a
    /// path component in a published member's file name and deriving it would
    /// need a format at a site that has to be `const`.
    pub(crate) class: &'static str,
    /// Rows of the activations operand and of the result.
    pub(crate) m: u64,
    /// Rows of the weights operand and columns of the result.
    pub(crate) n: u64,
    /// The contracted extent, shared by both operands.
    pub(crate) k: u64,
    /// `m * n * k`, the multiply-accumulate fold the oracle is asked for.
    ///
    /// Stated rather than multiplied at the point of use, and checked against the
    /// product by `tests::the_pinned_cells_are_the_retained_records_own_direct_rows`.
    /// It is what decides whether publishing this cell needs a *stated* reference
    /// iteration-step allowance, and a number a reader can compare against the
    /// record by eye is worth more there than an expression.
    pub(crate) fold_steps: u64,
    /// SHA-256 of the `direct` realization's result bytes.
    pub(crate) result_sha256: &'static str,
}

/// The reference evaluator's default per-occurrence iteration-step allowance.
///
/// Restated rather than imported because `tiler-reference` keeps the constant
/// `pub(crate)`. Unlike the two other restatements of it in this workspace, this
/// one is checked: `ReferenceEvaluator::iteration_step_allowance` is a public
/// accessor and `crate::publication::proof::tests` compares the two, so a bound
/// that moved is a red test rather than a stale number deciding which cells the
/// gate runs.
pub(crate) const REFERENCE_DEFAULT_STEP_ALLOWANCE: u64 = 16 * 1024 * 1024;

/// The six correctness cells of the L3 contraction profile, in the record's own
/// order.
///
/// Named for the retained cells rather than for the profile, because a class
/// like `contraction-l3` would name whichever of the six happened to land first.
pub(crate) const L3_CORRECTNESS_CELLS: [L3CorrectnessCell; 6] = [
    L3CorrectnessCell {
        id: "w_decode_kv",
        class: "contraction-w-decode-kv",
        m: 1,
        n: 1024,
        k: 1024,
        fold_steps: 1_048_576,
        result_sha256: "79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f",
    },
    L3CorrectnessCell {
        id: "w_prefill_q",
        class: "contraction-w-prefill-q",
        m: 10,
        n: 2048,
        k: 1024,
        fold_steps: 20_971_520,
        result_sha256: "1c54f5cd7265ee288ec79bcd9254243b78a95d57c3c489e5ea90bcc4298073c0",
    },
    L3CorrectnessCell {
        id: "w_prefill_mlp_in",
        class: "contraction-w-prefill-mlp-in",
        m: 128,
        n: 3072,
        k: 1024,
        fold_steps: 402_653_184,
        result_sha256: "eb382840ac9e533f57e51a0ffed2d61608664ecc5869aaa9f93afa3c312696a0",
    },
    L3CorrectnessCell {
        id: "w_prefill_mlp_out",
        class: "contraction-w-prefill-mlp-out",
        m: 128,
        n: 1024,
        k: 3072,
        fold_steps: 402_653_184,
        result_sha256: "124571de47ebff2f152b120afc9944b3465bffe94d8ac283a077677f61feb5f5",
    },
    L3CorrectnessCell {
        id: "w_prefill_o",
        class: "contraction-w-prefill-o",
        m: 128,
        n: 1024,
        k: 2048,
        fold_steps: 268_435_456,
        result_sha256: "b99eff9042d9e4b25e3844ff0462e5e6303e57b146aa79400622885bffc5f2f6",
    },
    L3CorrectnessCell {
        id: "w_vocab_slice",
        class: "contraction-w-vocab-slice",
        m: 1,
        n: 8192,
        k: 1024,
        fold_steps: 8_388_608,
        result_sha256: "88b01ae776f42bdb2f2d1092ddfd039e20e652d28393a6e2ec19e5cc1d9803c8",
    },
];

impl L3CorrectnessCell {
    /// Whether the *default* reference evaluator folds this cell's oracle.
    ///
    /// **This is the line the gate is split on, and it is a property rather than
    /// a cost estimate.** Publishing a cell needs the reference's expected bytes,
    /// and a fold above the default allowance is only reachable by a caller
    /// stating a larger number. A cell on this side of the line therefore
    /// publishes under exactly the evaluator every other consumer gets, so
    /// nothing the ordinary gate runs authorizes extra host work; the cells on
    /// the other side are routed by an `#[ignore]`d run that states the
    /// allowance where a reader can see it.
    pub(crate) const fn folds_under_the_default_allowance(&self) -> bool {
        self.fold_steps <= REFERENCE_DEFAULT_STEP_ALLOWANCE
    }

    /// Bytes of this cell's largest proof-sidecar payload.
    ///
    /// A sidecar binds one payload per declared interface entry, so the largest
    /// of the three — `[m, k]` activations, `[n, k]` weights, `[m, n]` expected —
    /// is what the artifact layer's per-payload bound is applied to. Computed
    /// rather than tabulated, because a cell whose extents moved must move this
    /// with them.
    pub(crate) const fn largest_payload_bytes(&self) -> u64 {
        let activations = self.m * self.k;
        let weights = self.n * self.k;
        let expected = self.m * self.n;
        let largest = if activations > weights {
            activations
        } else {
            weights
        };
        let largest = if largest > expected {
            largest
        } else {
            expected
        };
        largest * crate::serial_sum::F32_BYTES
    }

    /// Whether the artifact layer's proof sidecar can carry this cell at all.
    ///
    /// **A hard boundary rather than a cost, and it is owned elsewhere.**
    /// `tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES` bounds one case payload
    /// at 16 MiB, and `w_vocab_slice`'s `[8192, 1024]` weights operand is exactly
    /// 33,554,432 bytes — twice it. No arrangement inside this crate reaches that
    /// cell: splitting the operand across cases would publish a different program,
    /// and the constant is `tiler-artifact`'s public surface, which
    /// `implementation/conformance` does not own.
    /// `tests::the_unpublishable_cell_is_named_against_the_bound_that_stops_it`
    /// holds the exclusion to that arithmetic, so a raised bound admits the cell
    /// by making a test say so rather than by anyone remembering.
    pub(crate) fn fits_one_proof_payload(&self) -> bool {
        u64::try_from(tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES)
            .is_ok_and(|limit| self.largest_payload_bytes() <= limit)
    }

    /// The extents this cell is published and routed at.
    pub(crate) const fn extents(&self) -> (u64, u64, u64) {
        (self.m, self.n, self.k)
    }
}

/// How many entries and shared allocations a member of each role must show.
///
/// This is the matrix's central observable, not a formality. `selected` is the
/// fused plan: one dispatch, no intermediate. `materialized` computes the same
/// function as two dispatches through one shared allocation. Asserting the
/// counts is what separates "both agreed" from "both ran the same program
/// twice", and the latter would agree trivially.
pub(crate) fn expected_shape(role: &str) -> (usize, usize) {
    if role == "selected" { (1, 0) } else { (2, 1) }
}

/// Returns the envelope path for one published member of the proof matrix.
///
/// One derivation, reached by the publication that writes the member and by the
/// route that opens it, so the whole set stays obviously one unit on disk and no
/// two members can collide.
pub(crate) fn proof_member(base: &Path, class: &str, role: &str) -> PathBuf {
    let mut name = base.as_os_str().to_owned();
    name.push(format!(".{class}.{role}"));
    PathBuf::from(name)
}

/// Why one envelope run did not complete.
#[derive(Debug)]
pub(crate) enum EnvelopeFailure {
    /// A file this run needs could not be read.
    Read(String, std::io::Error),
    /// The proof sidecar did not decode.
    Sidecar(ProofCodecError),
    /// The sidecar does not describe the envelope it was paired with.
    SidecarAssociation(ProofAssociationError),
    /// The sidecar carries no case with an input and an expected output.
    SidecarWithoutCases,
    /// A payload is not exactly the declared element count.
    SidecarShapeMismatch {
        /// Which payload disagreed.
        role: &'static str,
        /// Elements the artifact declares.
        declared: u64,
        /// Bytes the sidecar records.
        recorded: usize,
    },
    /// A case binds a different number of operands than the artifact declares.
    SidecarInterfaceArity {
        /// Payloads the case supplies.
        sidecar: usize,
        /// Inputs the artifact declares.
        artifact: usize,
    },
    /// A case's operand is placed under a different key than the artifact
    /// declares.
    SidecarInterfaceKey {
        /// The key the sidecar used.
        sidecar: String,
        /// The key the artifact declares.
        artifact: String,
    },
    /// The identity recorded beside the artifact is not statable as one.
    RecordedIdentity(String),
    /// The artifact's interface is not the program family this run expects.
    Interface(String),
    /// The compiler's target profile does not compose a host environment.
    HostProfile,
    /// The artifact was refused by the loader.
    Load(LoadRejection),
    /// The fail-closed probes have no accepted neighbour to perturb.
    ProbeBaseline(LoadRejection),
    /// A fail-closed probe could not be constructed from these bytes.
    UnprobableEnvelope {
        /// What made the envelope unprobable.
        detail: &'static str,
    },
    /// The loader did not fail closed on a perturbed input.
    NotFailedClosed {
        /// Which probe was run.
        probe: &'static str,
        /// What happened instead of the refusal its class requires.
        outcome: String,
    },
    /// A routed launch covers no threads and is not declared skippable.
    EmptyLaunch {
        /// Position of the entry.
        entry: usize,
        /// Whether the route declares the dispatch skippable.
        skipped: bool,
    },
    /// A routed slot addresses something this run binds no storage for.
    UnboundBinding {
        /// Position of the entry.
        entry: usize,
        /// The ABI slot.
        slot: usize,
        /// What that slot addresses.
        target: String,
    },
    /// A binding's accessible range does not fit a `u64` allocation length.
    BindingRangeOverflow {
        /// Position of the entry.
        entry: usize,
        /// The ABI slot.
        slot: usize,
        /// Where the range starts.
        offset: u64,
        /// How far it reaches.
        extent: u64,
    },
    /// The route bound storage for fewer program inputs than the artifact
    /// declares.
    ///
    /// The check that makes "two inputs actually reached the device" a measured
    /// fact rather than an inference from the interface. A route binding one slot
    /// for a two-operand program would dispatch against an unwritten buffer, and
    /// an unwritten `StorageModeShared` allocation is zeroed rather than poisoned
    /// — so the result would be a plausible tensor, not a crash.
    UnboundOperand {
        /// Slots this run placed operands into.
        bound: usize,
        /// Inputs the artifact declares.
        declared: usize,
    },
    /// The device refused the route, before any commit.
    DevicePreflight(Box<PreflightRefusal>),
    /// A member routed to a different number of dispatches than its role means.
    ///
    /// The fused and materialized members must not converge on one shape. If they
    /// did, their bit-for-bit agreement would be the agreement of one program
    /// with itself, which proves nothing about the optimization the matrix exists
    /// to check.
    UnexpectedRouteShape {
        /// Which member routed.
        member: String,
        /// Dispatches the role means.
        expected_entries: usize,
        /// Dispatches it routed.
        entries: usize,
        /// Shared allocations the role means.
        expected_shared: usize,
        /// Shared allocations it routed.
        shared: usize,
    },
    /// The device and the published reference computed different bits.
    Mismatch {
        /// Which path disagreed.
        path: &'static str,
        /// What the device returned.
        device: Vec<u32>,
        /// What the published record requires.
        reference: Vec<u32>,
    },
    /// A member's bytes do not carry the digest a device measured over the same
    /// operands.
    ///
    /// **Its own class rather than a [`Self::Mismatch`], because it is a
    /// different disagreement.** `Mismatch` says this device and the published
    /// reference computed different bits. This says the two agreed with each
    /// other and disagreed with a *measurement* — a correctness finding about the
    /// vertical rather than about one dispatch — and the three digests it carries
    /// are what narrow it.
    RetainedDigestMismatch {
        /// Which member was compared.
        member: &'static str,
        /// Which operand case.
        case: String,
        /// Digest of the bytes this device produced.
        executed: String,
        /// Digest of the producer's published expectation.
        embedded: String,
        /// The retained realization-probe measurement.
        retained: &'static str,
    },
    /// The packaged program's identity matches none of the alternatives this
    /// process compiles for the artifact's own declared shape.
    ///
    /// Two numbers rather than one, because they answer different questions:
    /// this compares one identity against a *count* of candidate alternatives,
    /// and rendering that count beside a byte length once read as "compiled one
    /// of 2 \[bytes\]" — a misdirection at exactly the moment somebody is
    /// diagnosing a drift.
    ForeignProgram {
        /// Byte length of the packaged program's canonical identity.
        packaged: usize,
        /// How many compiled alternatives were checked, none of which matched.
        alternatives: usize,
    },
    /// Something else this run reached refused.
    Stage(String),
}

impl std::fmt::Display for EnvelopeFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(path, cause) => write!(formatter, "{path} could not be read: {cause}"),
            Self::Sidecar(cause) => {
                write!(formatter, "the proof sidecar did not decode: {cause}")
            }
            Self::SidecarAssociation(cause) => write!(
                formatter,
                "the proof sidecar does not describe this envelope: {cause}",
            ),
            Self::SidecarWithoutCases => formatter.write_str(
                "the proof sidecar carries no case with an input and an expected output",
            ),
            Self::SidecarShapeMismatch {
                role,
                declared,
                recorded,
            } => write!(
                formatter,
                "the artifact declares {declared} {role} element(s), which is {} byte(s), and the \
                 sidecar records {recorded}",
                declared.saturating_mul(F32_BYTES),
            ),
            Self::SidecarInterfaceArity { sidecar, artifact } => write!(
                formatter,
                "a proof case binds {sidecar} operand payload(s) and the artifact declares \
                 {artifact} input(s)",
            ),
            Self::SidecarInterfaceKey { sidecar, artifact } => write!(
                formatter,
                "a proof case places an operand under {sidecar:?} where the artifact declares \
                 {artifact:?}",
            ),
            Self::RecordedIdentity(cause) => write!(
                formatter,
                "the recorded artifact identity is not statable: {cause}",
            ),
            Self::Interface(detail) => write!(
                formatter,
                "the artifact's interface is not this program's: {detail}",
            ),
            Self::HostProfile => formatter
                .write_str("the compiler's target profile does not compose a host environment"),
            Self::Load(rejection) => write!(formatter, "the artifact was refused: {rejection}"),
            Self::ProbeBaseline(rejection) => write!(
                formatter,
                "the fail-closed probes have no accepted neighbour to perturb: the unperturbed \
                 subject was itself refused: {rejection}",
            ),
            Self::UnprobableEnvelope { detail } => write!(
                formatter,
                "a fail-closed probe could not be constructed from these bytes: {detail}",
            ),
            Self::NotFailedClosed { probe, outcome } => {
                write!(
                    formatter,
                    "the loader did not fail closed on {probe}: {outcome}"
                )
            }
            Self::EmptyLaunch { entry, skipped } => write!(
                formatter,
                "entry {entry}'s routed launch covers no threads (skipped: {skipped}), so there is \
                 no result to compare",
            ),
            Self::UnboundBinding {
                entry,
                slot,
                target,
            } => write!(
                formatter,
                "entry {entry}'s ABI slot {slot} addresses {target}, which this run binds no \
                 storage for",
            ),
            Self::BindingRangeOverflow {
                entry,
                slot,
                offset,
                extent,
            } => write!(
                formatter,
                "entry {entry}'s ABI slot {slot} starts at byte {offset} and reaches {extent} \
                 byte(s), which does not fit in a u64 allocation length",
            ),
            Self::UnboundOperand { bound, declared } => write!(
                formatter,
                "the route binds storage for {bound} program input(s) and the artifact declares \
                 {declared}; an unbound operand buffer is read as zeroes rather than refused",
            ),
            Self::DevicePreflight(refusal) => write!(
                formatter,
                "this device refused the route before the commit: {refusal}",
            ),
            Self::UnexpectedRouteShape {
                member,
                expected_entries,
                entries,
                expected_shared,
                shared,
            } => write!(
                formatter,
                "{member} routed {entries} dispatch(es) over {shared} shared allocation(s), and \
                 its role means {expected_entries} over {expected_shared}",
            ),
            Self::Mismatch {
                path,
                device,
                reference,
            } => write!(
                formatter,
                "the {path} path returned {device:08x?}, reference requires {reference:08x?}",
            ),
            Self::RetainedDigestMismatch {
                member,
                case,
                executed,
                embedded,
                retained,
            } => write!(
                formatter,
                "{member} case {case:?}: the SHA-256 of the executed result bytes is {executed} \
                 and the retained realization-probe measurement is {retained}; the producer's \
                 published expectation hashes to {embedded}. This is a correctness finding about \
                 the contraction vertical, not a dispatch failure",
            ),
            Self::ForeignProgram {
                packaged,
                alternatives,
            } => write!(
                formatter,
                "the artifact packages a kernel program whose {packaged}-byte identity matches \
                 none of the {alternatives} alternative(s) this process compiled for the \
                 artifact's own declared shape; what was published and what this route derives \
                 from the declaration have drifted",
            ),
            Self::Stage(detail) => formatter.write_str(detail),
        }
    }
}

impl std::error::Error for EnvelopeFailure {}

/// Reads exactly `elements` big-endian `f32` bit patterns out of a sidecar
/// payload, or refuses the payload.
///
/// Most-significant byte first, matching the order the producer wrote, so the
/// operands never depend on host endianness. Bit patterns throughout: a signed
/// zero, a subnormal, and a non-canonical NaN must survive to the comparison
/// unchanged, which they would not if these were parsed as numbers.
///
/// The length is checked rather than truncated to a whole number of elements. A
/// payload that decodes short would reach the comparison as a shorter vector and
/// be reported as [`EnvelopeFailure::Mismatch`] — a claim about the *device's*
/// arithmetic, made about a defect in the record. Refusing here keeps a
/// malformed sidecar in the sidecar's own error class.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::SidecarShapeMismatch`] for any other length.
pub(crate) fn decode_f32_bits(
    role: &'static str,
    elements: u64,
    bytes: &[u8],
) -> Result<Vec<u32>, EnvelopeFailure> {
    let needed = elements
        .checked_mul(F32_BYTES)
        .and_then(|needed| usize::try_from(needed).ok())
        .ok_or(EnvelopeFailure::SidecarShapeMismatch {
            role,
            declared: elements,
            recorded: bytes.len(),
        })?;
    if bytes.len() != needed {
        return Err(EnvelopeFailure::SidecarShapeMismatch {
            role,
            declared: elements,
            recorded: bytes.len(),
        });
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| u32::from_be_bytes(*chunk))
        .collect())
}

/// One input an artifact declares, read rather than assumed.
///
/// The extents are kept alongside the element count because the two answer
/// different questions: a buffer is sized from the count, and a *family* is
/// recognized from the shape — `[M, K]` and `[N, K]` are the same count at
/// `M = N` and are still two different operands.
#[derive(Clone, Debug)]
pub(crate) struct DeclaredInput {
    /// The interface key the artifact declares for this input.
    pub(crate) key: String,
    /// Its declared extents, in the artifact's own order.
    pub(crate) extents: Vec<u64>,
    /// Its element count, the product of those extents.
    pub(crate) elements: u64,
}

/// The whole interface one artifact declares.
///
/// **Ordered as the artifact orders it, and that order is load-bearing.** The
/// sidecar's case payloads, the routed `BindingTarget::ProgramInput` keys, and
/// this vector are three views of one interface; the ordinal a binding resolves
/// to here is the ordinal its operands are read from, so a host that sorted or
/// deduplicated this list would bind the right number of buffers with the wrong
/// bytes in them — a wrong answer rather than a refusal.
#[derive(Clone, Debug)]
pub(crate) struct DeclaredInterface {
    /// Every declared input, in the artifact's own interface order.
    pub(crate) inputs: Vec<DeclaredInput>,
    /// The interface key of the one declared output.
    pub(crate) output_key: String,
    /// How many elements that output publishes.
    pub(crate) output_elements: u64,
    /// The ABI facts bound from those declared shapes.
    pub(crate) abi: AbiFacts,
}

/// Reads the interface an artifact declares, and binds every declared shape.
///
/// **The declared shape is read rather than asserted equal to a constant this
/// build holds, and that is the design rather than a gap.** What a consumer may
/// take from an artifact is what the artifact says; asserting a shape here would
/// replace the artifact's declaration with this build's expectation, and two
/// halves would then agree because they were told to rather than because one
/// packaged what the other runs.
///
/// **No input count is expected here**, deliberately. This function reads what
/// the artifact declares and refuses only what it cannot represent; which
/// cardinality a given program family requires is
/// [`require_serial_sum_interface`]'s and [`require_contraction_interface`]'s to
/// state.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::Interface`] for a logical type this run does not
/// bind, a shape that does not bind into the ABI facts, or an output count other
/// than one.
pub(crate) fn bind_declared_interface(
    decoded: &DecodedProgram,
) -> Result<DeclaredInterface, EnvelopeFailure> {
    let f32_type = F32::resolved_type().canonical_encoding();
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    let mut inputs = Vec::with_capacity(decoded.inputs().len());
    for (position, input) in decoded.inputs().enumerate() {
        if input.resolved_type_encoding() != f32_type.as_bytes() {
            return Err(EnvelopeFailure::Interface(format!(
                "the artifact's input {position} {:?} has logical type {:02x?} and this run binds \
                 canonical F32 only",
                input.key().as_str(),
                input.resolved_type_encoding(),
            )));
        }
        let extents: Vec<u64> = input
            .shape()
            .extents()
            .iter()
            .map(|extent| extent.get())
            .collect();
        binder
            .bind_input_shape(input.key(), input.shape())
            .map_err(|cause| {
                EnvelopeFailure::Interface(format!(
                    "the declared shape of input {position} {:?} does not bind: {cause}",
                    input.key().as_str(),
                ))
            })?;
        inputs.push(DeclaredInput {
            key: input.key().as_str().to_owned(),
            elements: extents.iter().product(),
            extents,
        });
    }

    let outputs: Vec<_> = decoded.outputs().collect();
    let [output] = outputs.as_slice() else {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact declares {} outputs and this run reads back exactly 1",
            outputs.len(),
        )));
    };
    if output.resolved_type_encoding() != f32_type.as_bytes() {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact's output {:?} has logical type {:02x?} and this run reads canonical F32 \
             only",
            output.key().as_str(),
            output.resolved_type_encoding(),
        )));
    }

    Ok(DeclaredInterface {
        inputs,
        output_key: output.key().as_str().to_owned(),
        output_elements: output
            .shape()
            .extents()
            .iter()
            .map(|extent| extent.get())
            .product(),
        abi: binder.build(),
    })
}

/// Requires the declared interface to be the serial sum's, and returns its
/// extents.
///
/// Split from [`bind_declared_interface`] so the *reading* of an interface and
/// the *expectation* of one program family are two steps. Reading is what a
/// consumer may take from an artifact; expecting is this build's own claim, and
/// keeping them apart is what lets a second family be added without either one
/// growing a special case for the other.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::Interface`] for any interface that is not a
/// rank-2, one-input reduction publishing one element per row.
pub(crate) fn require_serial_sum_interface(
    interface: &DeclaredInterface,
) -> Result<(u64, u64), EnvelopeFailure> {
    let [input] = interface.inputs.as_slice() else {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact declares {} input(s) and the serial sum declares 1",
            interface.inputs.len(),
        )));
    };
    let [rows, columns] = input.extents.as_slice() else {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact's input has rank {} and the serial sum reduces a rank-2 input",
            input.extents.len(),
        )));
    };
    if input.key != INPUT_KEY || interface.output_key != OUTPUT_KEY {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact's interface is {:?} -> {:?} and the serial sum's is {INPUT_KEY:?} -> \
             {OUTPUT_KEY:?}",
            input.key, interface.output_key,
        )));
    }
    if interface.output_elements != *rows {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact publishes {} F32 element(s) and reducing a {rows}x{columns} input's \
             inner axis publishes {rows}",
            interface.output_elements,
        )));
    }
    Ok((*rows, *columns))
}

/// Requires the declared interface to be the contraction's, and returns
/// `(M, N, K)`.
///
/// **The shared contracted extent is checked rather than taken from one
/// operand.** `td,od->to` requires `activations[M, K]` and `weights[N, K]` to
/// agree on `K`, and an artifact whose two operands disagreed would describe a
/// program the structure's own extent-agreement rule refuses — so reading `K` off
/// the first operand and never looking at the second would turn a malformed
/// interface into a silently wrong buffer length.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::Interface`] for a wrong operand count, wrong keys,
/// a rank other than two, disagreeing contracted extents, or an output count
/// that is not `M * N`.
pub(crate) fn require_contraction_interface(
    interface: &DeclaredInterface,
) -> Result<(u64, u64, u64), EnvelopeFailure> {
    let [activations, weights] = interface.inputs.as_slice() else {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact declares {} input(s) and the contraction declares 2",
            interface.inputs.len(),
        )));
    };
    if activations.key != CONTRACTION_ACTIVATIONS_KEY
        || weights.key != CONTRACTION_WEIGHTS_KEY
        || interface.output_key != CONTRACTION_OUTPUT_KEY
    {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact's interface is ({:?}, {:?}) -> {:?} and the contraction's is \
             ({CONTRACTION_ACTIVATIONS_KEY:?}, {CONTRACTION_WEIGHTS_KEY:?}) -> \
             {CONTRACTION_OUTPUT_KEY:?}",
            activations.key, weights.key, interface.output_key,
        )));
    }
    let ([m, left_k], [n, right_k]) = (activations.extents.as_slice(), weights.extents.as_slice())
    else {
        return Err(EnvelopeFailure::Interface(format!(
            "the contraction's operands have ranks {} and {}, and `td,od->to` reads two rank-2 \
             operands",
            activations.extents.len(),
            weights.extents.len(),
        )));
    };
    if left_k != right_k {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact's operands contract over {left_k} and {right_k}, and `td,od->to` shares \
             one contracted extent",
        )));
    }
    let published = m.checked_mul(*n).ok_or_else(|| {
        EnvelopeFailure::Interface(format!("a {m}x{n} result has no element count"))
    })?;
    if interface.output_elements != published {
        return Err(EnvelopeFailure::Interface(format!(
            "the artifact publishes {} F32 element(s) and a {m}x{n} contraction publishes \
             {published}",
            interface.output_elements,
        )));
    }
    Ok((*m, *n, *left_k))
}

/// Reads one case's operand payloads, one per input the artifact declares.
///
/// **Every payload, and each at its own declared element count.** Reading only
/// the leading payload is the whole set against a one-input artifact and half of
/// it against a two-input one, silently. The sidecar layer already guarantees one
/// payload per declared input — it refuses a case that supplies any other number
/// — so a count disagreement here is this reader's own defect and is reported as
/// one.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::SidecarInterfaceArity`],
/// [`EnvelopeFailure::SidecarInterfaceKey`], or a payload-length refusal.
pub(crate) fn case_operands(
    interface: &DeclaredInterface,
    case: ProofCaseRef<'_>,
) -> Result<Vec<Vec<u32>>, EnvelopeFailure> {
    let payloads: Vec<_> = case.inputs().collect();
    if payloads.len() != interface.inputs.len() {
        return Err(EnvelopeFailure::SidecarInterfaceArity {
            sidecar: payloads.len(),
            artifact: interface.inputs.len(),
        });
    }
    payloads
        .iter()
        .zip(&interface.inputs)
        .map(|(payload, declared)| {
            // Bound to the key as well as to the position: the sidecar places its
            // payloads into the artifact's interface order, so a disagreement
            // here means the two orders have drifted apart and every operand
            // after it would be written into the wrong buffer.
            if payload.key().as_str() != declared.key {
                return Err(EnvelopeFailure::SidecarInterfaceKey {
                    sidecar: payload.key().as_str().to_owned(),
                    artifact: declared.key.clone(),
                });
            }
            decode_f32_bits("input", declared.elements, payload.bytes())
        })
        .collect()
}

/// Reads one case's expected output payload at the declared element count.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::SidecarWithoutCases`] when the case carries none,
/// or a payload-length refusal.
pub(crate) fn case_expected(
    interface: &DeclaredInterface,
    case: ProofCaseRef<'_>,
) -> Result<Vec<u32>, EnvelopeFailure> {
    let payload = case
        .expected()
        .next()
        .ok_or(EnvelopeFailure::SidecarWithoutCases)?;
    decode_f32_bits("expected", interface.output_elements, payload.bytes())
}

/// Reads the envelope bytes and the identity recorded beside them.
///
/// The sidecar is the only thing that makes the loader's identity check mean
/// anything: an identity re-read from the envelope would be a tautology, so the
/// expected one has to come from whatever *named* the artifact, and here that is
/// [`crate::publication`], which derives it from the `VerifiedArtifactProgram` it
/// assembled rather than from the encoding. That still catches a stale envelope,
/// a mixed-up path, and a torn write between the two files; it resists nothing
/// that rewrites both, and nothing unsigned could.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::Read`] for either file, and the sidecar's own
/// decode or association refusal.
pub(crate) fn read_artifact(
    path: &Path,
) -> Result<(Vec<u8>, DecodedProofSidecar), EnvelopeFailure> {
    let sidecar_path = sidecar_path(path);
    let bytes = std::fs::read(path)
        .map_err(|cause| EnvelopeFailure::Read(path.display().to_string(), cause))?;
    let sidecar_bytes = std::fs::read(&sidecar_path)
        .map_err(|cause| EnvelopeFailure::Read(sidecar_path.display().to_string(), cause))?;
    let sidecar = decode_proof_sidecar(&sidecar_bytes).map_err(EnvelopeFailure::Sidecar)?;

    // The record names an exact envelope by digest and by artifact identity, so a
    // sidecar paired with the wrong artifact is caught here rather than surviving
    // to be compared against bits it never described. A torn write between the
    // two files fails the same way, loudly.
    sidecar
        .bind_to_envelope(&bytes)
        .map_err(EnvelopeFailure::SidecarAssociation)?;
    eprintln!(
        "  artifact: {} ({} bytes), sidecar {} ({} bytes, {} case(s))",
        path.display(),
        bytes.len(),
        sidecar_path.display(),
        sidecar_bytes.len(),
        sidecar.cases().len(),
    );
    Ok((bytes, sidecar))
}

/// The environment the **diagnostic** envelope route runs under.
///
/// # This is producer-declared equality, NOT host-earned eligibility
///
/// The profile below is the one `tiler-build` *declares* for this Metal target;
/// nothing about this host earned the right to offer it.
/// `crate::applicability::refuse_to_offer_the_declared_profile` is where that
/// second question is asked, and it refuses.
///
/// # The dtype rows are read from the declaration, and the gap that leaves
///
/// Every field comes from `declaration`, the dtype rows included, so a widened,
/// narrowed, or retracted ledger measurement moves this environment with it
/// rather than leaving it stating a verdict the profile stopped holding.
///
/// **That removes a second copy of the rows and not the authority gap.** They
/// remain *producer-declared*: this crate holds a real `MTLDevice` and asks it
/// nothing about either dtype, so what is stated here is what the ledger measured
/// on its own host and not what this machine demonstrated.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::HostProfile`] when the declared profile does not
/// compose a host environment.
pub(crate) fn declared_route_environment(
    declaration: &BoundMetalCompileDeclaration,
) -> Result<ExecutionEnvironment, EnvelopeFailure> {
    let profile = declaration
        .target_profile_ref()
        .map_err(|_| EnvelopeFailure::HostProfile)?;
    Ok(ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: TargetProfileKey::new(profile.key.as_str())
                .map_err(|_| EnvelopeFailure::HostProfile)?,
            descriptor: profile.descriptor,
        },
        backend: BackendKey::new(BACKEND_KEY).map_err(|_| EnvelopeFailure::HostProfile)?,
        representation: RepresentationKey::new(REPRESENTATION_KEY)
            .map_err(|_| EnvelopeFailure::HostProfile)?,
        // The same authority as every field above, under the caveat this
        // function's heading states. A dtype the declaration says nothing about
        // produces no row at all, which is what keeps silence fail-closed: the
        // loader resolves an absent key as `Unknown` and refuses it rather than
        // reading the absence as permission.
        dtype_dispatch: declaration
            .dtype_dispatchability_rows()
            .into_iter()
            .map(|(arithmetic, verdict)| (arithmetic, host_dtype_dispatch(verdict)))
            .collect(),
    })
}

/// Restates one declared dispatchability verdict in the host's own vocabulary.
///
/// An exhaustive match rather than a conversion helper on either vocabulary's own
/// crate: `tiler-build` states what a *profile* declares and `tiler-runtime`
/// states what a *host* offers, neither depends on the other, and this is the one
/// place this consumer turns the first into the second. Wildcard-free, so a
/// verdict added to the compile-profile vocabulary stops this crate compiling
/// instead of being guessed at.
pub(crate) const fn host_dtype_dispatch(verdict: DTypeDispatchability) -> DTypeDispatch {
    match verdict {
        DTypeDispatchability::Dispatchable => DTypeDispatch::Dispatchable,
        DTypeDispatchability::Unsupported => DTypeDispatch::Unsupported,
    }
}

/// The L3 profile's index structure, `td,od->to`.
///
/// Spelled with arbitrary frontend index labels rather than with `0, 1, 2`, so
/// the published artifact reaches its canonical encoding through the
/// renaming-invariant rule ADR 0087 requires rather than by the labels happening
/// to be the canonical ones.
///
/// **What that no longer establishes here, stated rather than left implied.**
/// This spelling used to be one of two, reached independently by the publishing
/// executable and by this consumer, so a renaming-invariance defect showed up as
/// two identities that would not match. One function serves both halves now, so
/// what remains is that the *published* member is built from non-canonical labels
/// — which keeps the rule exercised end to end — and not that two spellings
/// agree. `prototypes/serial-sum-compile` carries the second spelling.
pub(crate) fn contraction_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .expect("the profile's index structure passes every structural admission rule")
}

/// Builds `activations[m, k] x weights[n, k] -> projected[m, n]`.
///
/// Reconstructed here for the same reason the serial sum is: the envelope carries
/// no semantic program, so naming the computation the artifact packages requires
/// deriving it independently and comparing canonical identities.
pub(crate) fn contraction_program(m: u64, n: u64, k: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let activations = builder
        .input::<F32>(
            InputKey::new(CONTRACTION_ACTIVATIONS_KEY).expect("the activations key is valid"),
            Shape::from_dims([m, k]),
        )
        .expect("the activations operand binds");
    let weights = builder
        .input::<F32>(
            InputKey::new(CONTRACTION_WEIGHTS_KEY).expect("the weights key is valid"),
            Shape::from_dims([n, k]),
        )
        .expect("the weights operand binds");
    let projected =
        F32TensorContraction::apply(&mut builder, &contraction_structure(), activations, weights)
            .expect("the contraction applies");
    builder
        .output(
            OutputKey::new(CONTRACTION_OUTPUT_KEY).expect("the output key is valid"),
            projected,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Compiles this build's alternatives for the shape *the artifact* declares.
///
/// **The one place a declared shape becomes a program, and the reason it is a
/// function rather than four lines inside a routing loop.** The historic defect
/// was those four lines compiling a consumer's *own* row count against a producer
/// that had moved to another, so every packaged program was foreign and the whole
/// matrix could prove nothing. Nothing in the repository could see it, because
/// the matrix ran only against real published members on hardware.
///
/// Routing it through one named function is what lets a device-free case run the
/// same code.
///
/// # Errors
///
/// Returns the interface refusal or the compile refusal of whichever stage
/// declined.
pub(crate) fn compile_for_declared_shape(
    declaration: &BoundMetalCompileDeclaration,
    decoded: &DecodedProgram,
) -> Result<(u64, u64, Compilation), EnvelopeFailure> {
    let interface = bind_declared_interface(decoded)?;
    let (rows, columns) = require_serial_sum_interface(&interface)?;
    let compilation = compile_under(
        declaration,
        &serial_sum_program(rows, columns),
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    )
    .map_err(|cause| EnvelopeFailure::Stage(cause.to_string()))?;
    Ok((rows, columns, compilation))
}

/// Requires a packaged kernel program to be one this build derived for that
/// declared shape.
///
/// The packaged program is matched against *some* alternative rather than against
/// the selected one: a producer legitimately packages a plan the portfolio did
/// not rank first, and demanding `selected` would refuse a materialized member
/// for being exactly what it is meant to be. The set is still this build's own
/// governed compilation of the shape the artifact declares, so this is a narrower
/// claim than "some program" by a wide margin.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::ForeignProgram`] when no alternative's canonical
/// identity is the packaged one.
pub(crate) fn require_derived_program(
    compilation: &Compilation,
    packaged: &[u8],
) -> Result<(), EnvelopeFailure> {
    if compilation.alternatives().any(|alternative| {
        alternative
            .abi()
            .kernel_program()
            .canonical_identity()
            .as_bytes()
            == packaged
    }) {
        return Ok(());
    }
    Err(EnvelopeFailure::ForeignProgram {
        packaged: packaged.len(),
        alternatives: compilation.alternatives().count(),
    })
}

/// Which storage a run will supply for one routed ABI slot.
///
/// Resolved before the commit and carried as an owned decision, so the encoder
/// never re-asks a question whose answer could have refused the route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Placement {
    /// The buffer holding one program input the artifact names, by its ordinal
    /// in the artifact's declared interface.
    ///
    /// **The ordinal is what makes a two-operand route expressible.** A placement
    /// that could not say *which* operand a slot takes would fill both buffers
    /// from the same host slice and return a plausible tensor computed from the
    /// wrong bytes.
    Input(usize),
    /// The buffer receiving the program output the artifact names.
    Output,
    /// Entry-internal storage: named by nothing, sized by its own
    /// accessible-byte expression, and allocated rather than bound.
    Internal,
}

/// One routed ABI slot, resolved to storage this host can actually supply.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlacedSlot {
    /// The argument-table index this slot binds at.
    pub(crate) transport: u32,
    /// The byte offset the binding starts from.
    pub(crate) offset: u64,
    /// The byte length the allocation must reach through.
    pub(crate) needed: u64,
    /// What this run will bind there.
    pub(crate) placement: Placement,
}

/// Decides whether this host can carry out a route, while abandoning it is still
/// permitted.
///
/// **Every refusal here is one a host owes itself before the commit.**
/// `Preflight` publishes the launch geometry and the routed bindings precisely so
/// a caller can judge them and decline; a host that instead committed and *then*
/// discovered it binds no storage for some slot would have destroyed its own
/// fallback authority for a reason that was decidable while it still held it.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::EmptyLaunch`] for a zero-thread dispatch the route
/// does not declare skippable, [`EnvelopeFailure::UnboundBinding`] for a target
/// this run places no storage for, and
/// [`EnvelopeFailure::BindingRangeOverflow`] for a range that does not fit.
pub(crate) fn plan_route(
    preflight: &Preflight<'_>,
    interface: &DeclaredInterface,
) -> Result<Vec<Vec<PlacedSlot>>, EnvelopeFailure> {
    let mut plan = Vec::with_capacity(preflight.entries().len());
    for (position, routed) in preflight.entries().iter().enumerate() {
        let launch = routed.launch();
        // An entry covering no threads is legitimate rather than exceptional: a
        // reduction over an empty domain maps zero elements before reducing them
        // to its identity element, so its first stage has nothing to run and its
        // second still produces every output. The artifact *states* which of the
        // two an empty launch is, so the answer is read rather than assumed, and a
        // route that demands a zero-thread dispatch be encoded is refused —
        // `dispatchThreads` has no meaning at zero and inventing one thread would
        // run a body the plan did not ask for.
        if launch.grid_threads() == 0 && !launch.zero_work_skips_dispatch() {
            return Err(EnvelopeFailure::EmptyLaunch {
                entry: position,
                skipped: false,
            });
        }

        let mut slots = Vec::with_capacity(routed.bindings().len());
        for binding in routed.bindings() {
            let placement = match binding.binding().target() {
                // Resolved against the artifact's *own* declared input order
                // rather than against a key constant this build holds. That is
                // what makes one function place a one-operand reduction and a
                // two-operand contraction without knowing which it is looking at.
                BindingTarget::ProgramInput(key) => interface
                    .inputs
                    .iter()
                    .position(|declared| declared.key == key.as_str())
                    .map_or_else(
                        || {
                            Err(EnvelopeFailure::UnboundBinding {
                                entry: position,
                                slot: binding.slot(),
                                target: format!(
                                    "ProgramInput({:?}), which this artifact does not declare",
                                    key.as_str(),
                                ),
                            })
                        },
                        |ordinal| Ok(Placement::Input(ordinal)),
                    )?,
                BindingTarget::ProgramOutput(keys)
                    if keys.len() == 1 && keys[0].as_str() == interface.output_key =>
                {
                    Placement::Output
                }
                BindingTarget::Internal => Placement::Internal,
                // Named rather than left to a wildcard: every `ProgramInput` is
                // resolved above, so the only target that can still fall through
                // is an output whose key or arity is not this artifact's. A
                // catch-all would additionally swallow a *new* `BindingTarget`
                // variant as an ordinary refusal, where the repository's posture
                // is that a variant added to a vocabulary must be a build error at
                // every site that decides on it.
                other @ BindingTarget::ProgramOutput(_) => {
                    return Err(EnvelopeFailure::UnboundBinding {
                        entry: position,
                        slot: binding.slot(),
                        target: format!("{other:?}"),
                    });
                }
            };
            let offset = binding.accessible_offset();
            let needed = offset.checked_add(binding.accessible_bytes()).ok_or(
                EnvelopeFailure::BindingRangeOverflow {
                    entry: position,
                    slot: binding.slot(),
                    offset,
                    extent: binding.accessible_bytes(),
                },
            )?;
            slots.push(PlacedSlot {
                transport: binding.transport_slot(),
                offset,
                needed,
                placement,
            });
        }
        plan.push(slots);
    }
    Ok(plan)
}

/// Governed key of the Metal requirement naming a minimum Apple GPU family.
///
/// Owned by `tiler.metal`, which is the backend key this host states, so the
/// loader refuses a row owned by anything else before this adapter is asked.
pub(crate) const METAL_MINIMUM_GPU_FAMILY: &str =
    "tiler.metal.route-requirement.minimum-gpu-family";

/// Governed version of [`METAL_MINIMUM_GPU_FAMILY`]'s meaning.
///
/// Matched exactly. A version this adapter does not know is `Unrecognized` rather
/// than approximated, because one key at two versions can mean two things and
/// guessing which is how a route runs on a device it was refused on.
pub(crate) const METAL_MINIMUM_GPU_FAMILY_VERSION: u32 = 1;

/// Decides one live-device route requirement from an observed family.
///
/// Pure, and split from the device exactly as `tiler_metal::applicability` splits
/// its policy: an adapter observes, this decides.
///
/// # What Metal cannot answer here, stated rather than approximated
///
/// [`RouteResourceDimension::SubgroupThreads`] is a live-device property in
/// general — Vulkan publishes `subgroupSize` on the physical device — and Metal
/// publishes no device-scoped equivalent: `MTLDevice.h` in the macOS 26.5 SDK
/// declares no execution-width property, and `threadExecutionWidth` lives on
/// `MTLComputePipelineState`, which is a *prepared-kernel* fact. So this adapter
/// answers `Unrecognized`, which refuses the route. A route that genuinely needs
/// that width on Metal must state it as a `PreparedEntryTargetRequirement`
/// against the prepared pipeline, which is the authority that has it; answering it
/// from a family table here would report a documentation constant as a device
/// observation.
pub(crate) fn decide_live_device_requirement(
    observed: ProbedGpuFamily,
    request: LiveDeviceRequest<'_>,
) -> LiveDeviceObservation {
    // Exhaustive on both the kind and the dimension: a row this adapter has never
    // seen must stop this build rather than reach an arm that guesses.
    match request.requirement() {
        RouteRequirement::Resource(resource) => match resource.dimension() {
            RouteResourceDimension::SubgroupThreads => LiveDeviceObservation::Unrecognized,
        },
        RouteRequirement::BackendFeature(feature) => {
            if feature.key().as_str() != METAL_MINIMUM_GPU_FAMILY
                || feature.version() != METAL_MINIMUM_GPU_FAMILY_VERSION
            {
                return LiveDeviceObservation::Unrecognized;
            }
            let Some(required) = gpu_family_from_payload(feature.payload()) else {
                return LiveDeviceObservation::Unrecognized;
            };
            // Cumulative families: the highest supported family implies every
            // lower one, so the ordering decides support without a second device
            // call. A device naming none of them satisfies no family requirement.
            let supported = match observed {
                ProbedGpuFamily::Answered(MetalGpuFamilySupport::Highest(highest)) => {
                    highest >= required
                }
                ProbedGpuFamily::Answered(MetalGpuFamilySupport::NoneNamed) => false,
                // This adapter owns the row and still has no observation to decide
                // it from, which is what `Unrecognized` is for: it refuses the
                // route. `Feature(false)` would be this adapter reporting a device
                // that answered no to a question its binding could not put.
                ProbedGpuFamily::Unnameable(_) => return LiveDeviceObservation::Unrecognized,
            };
            LiveDeviceObservation::Feature(supported)
        }
    }
}

/// Reads a canonical family payload through the governed vocabulary's own
/// spelling.
///
/// Scanned against `MetalGpuFamily::ALL` rather than matched against a second
/// table of names written here: one spelling authority, so a family added to that
/// vocabulary cannot be silently unreadable at this boundary.
pub(crate) fn gpu_family_from_payload(payload: &[u8]) -> Option<MetalGpuFamily> {
    MetalGpuFamily::ALL
        .into_iter()
        .find(|family| family.as_str().as_bytes() == payload)
}

/// The exact inputs a fail-closed probe perturbs one element of.
///
/// Grouped rather than passed as four arguments so a probe's signature shows that
/// it changes *one* of them and leaves the rest alone. That is what makes a
/// refusal evidence about the perturbation rather than about the whole kind: the
/// same subject routes under [`probe_accepted_baseline`], so a probe that gets a
/// refusal has isolated its cause.
#[derive(Clone, Copy)]
pub(crate) struct ProbeSubject<'a> {
    /// The exact encoded envelope bytes under test.
    pub(crate) bytes: &'a [u8],
    /// The identity whatever named this artifact recorded, stated as such.
    pub(crate) expected: &'a RecordedArtifactProgramIdentity,
    /// What the host running these probes independently states it offers.
    pub(crate) environment: &'a ExecutionEnvironment,
    /// The ABI facts bound from the artifact's own declared interface.
    pub(crate) abi: &'a AbiFacts,
}

/// Reports a probe whose refusal did not arrive under the class it must.
fn refused(probe: &'static str, outcome: String) -> EnvelopeFailure {
    EnvelopeFailure::NotFailedClosed { probe, outcome }
}

/// Proves the loader **accepts** the unperturbed subject, before anything is
/// perturbed.
///
/// This is the neighbour every probe below is paired against, and without it each
/// of them proves close to nothing. A refusal is the easy outcome to obtain: a
/// subject whose bytes never decoded, whose recorded identity was wrong, or whose
/// host profile never matched would refuse *every* perturbation under a
/// plausible-looking class, and the probes would report a fail-closed loader while
/// measuring a broken harness.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::ProbeBaseline`] when the unperturbed subject is
/// itself refused.
pub(crate) fn probe_accepted_baseline(
    subject: &ProbeSubject<'_>,
) -> Result<String, EnvelopeFailure> {
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(EnvelopeFailure::ProbeBaseline)?;
    let preflight = decoded
        .prepare(subject.environment, subject.expected, subject.abi)
        .and_then(|qualification| {
            // This subject declares no live-device requirement, so the stage is
            // passed through. The resolver is still supplied rather than skipped,
            // because the stage is not skippable — which is what keeps a route
            // that *does* declare one from reaching a commit unchecked.
            qualification.resolve_live_device_requirements(|_| LiveDeviceObservation::Unrecognized)
        })
        .and_then(|preparation| preparation.resolve_target_properties(|_| u64::MAX))
        .map_err(EnvelopeFailure::ProbeBaseline)?;
    let entries = preflight.entries();
    let threads: u64 = entries
        .iter()
        .map(|entry| entry.launch().grid_threads())
        .sum();
    let bindings: usize = entries.iter().map(|entry| entry.bindings().len()).sum();
    Ok(format!(
        "the unperturbed subject routes: {} entr(y/ies), {threads} thread(s) over {bindings} \
         binding(s), {} shared allocation(s)",
        entries.len(),
        preflight.shared_allocations().len(),
    ))
}

/// A flipped byte inside a framed section's content is an **integrity** failure.
///
/// The class is *derived* rather than observed. The encoder writes the framing
/// header, then the manifest, then each section as its ordinal, its length, and
/// its exact content, so the last section's content ends the envelope — asserted
/// here rather than assumed. The manifest carries that section's content digest,
/// so a changed content byte can only be caught by a digest comparison: a section
/// digest, the payload identity derived from the metadata section, or the artifact
/// identity re-derived from decoded content. All three classify as
/// [`ArtifactCodecFailure::IntegrityFailure`], and none of them is a routing
/// question.
///
/// Pinning the exact class is the whole point. A damaged file reported as
/// `NoApplicableVariant` reads as "this artifact does not apply to your host",
/// which sends a reader to rebuild a plan when the repair is to re-fetch the
/// bytes; one reported as `Malformed` sends them to look for a different file.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::NotFailedClosed`] when the refusal arrives under
/// another class, or none at all.
pub(crate) fn probe_damaged_section_content(
    subject: &ProbeSubject<'_>,
) -> Result<String, EnvelopeFailure> {
    let decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(EnvelopeFailure::ProbeBaseline)?;
    let content = decoded
        .sections()
        .last()
        .ok_or(EnvelopeFailure::UnprobableEnvelope {
            detail: "the envelope frames no section to damage",
        })?
        .bytes()
        .to_vec();
    if content.is_empty() || !subject.bytes.ends_with(&content) {
        return Err(EnvelopeFailure::UnprobableEnvelope {
            detail: "the last framed section's content does not end the envelope",
        });
    }
    let at = subject.bytes.len() - content.len();

    let mut damaged = subject.bytes.to_vec();
    damaged[at] ^= 0x01;
    match DecodedProgram::decode(&damaged, SOLE_DELIVERY) {
        Err(rejection @ LoadRejection::Artifact(ArtifactCodecFailure::IntegrityFailure { .. })) => {
            Ok(format!(
                "a flipped byte at section offset {at}: {rejection}"
            ))
        }
        Err(other) => Err(refused("a damaged section", other.to_string())),
        Ok(_) => Err(refused(
            "a damaged section",
            "the envelope decoded as valid".to_owned(),
        )),
    }
}

/// A flipped byte at an arbitrary interior offset never survives into routing.
///
/// Retained beside [`probe_damaged_section_content`] because it perturbs the
/// envelope the way damage actually arrives — at an offset nobody chose — and
/// deliberately asserts less. Which boundary refuses is a function of where the
/// byte lands: inside the manifest or a section's content it is an integrity
/// failure, inside a framed length it is malformed, inside a section ordinal it is
/// invalid. What must hold for *every* offset is that the artifact layer refuses,
/// so that is what is asserted; pinning one of those classes here would pin an
/// accident of this envelope's size rather than a property of the loader.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::NotFailedClosed`] when the artifact layer does not
/// refuse.
pub(crate) fn probe_damaged_interior_byte(
    subject: &ProbeSubject<'_>,
) -> Result<String, EnvelopeFailure> {
    let mut damaged = subject.bytes.to_vec();
    let midpoint = damaged.len() / 2;
    damaged[midpoint] ^= 0x01;
    match DecodedProgram::decode(&damaged, SOLE_DELIVERY) {
        Err(rejection @ LoadRejection::Artifact(_)) => {
            Ok(format!("a flipped byte at offset {midpoint}: {rejection}"))
        }
        Err(other) => Err(refused("a flipped interior byte", other.to_string())),
        Ok(_) => Err(refused(
            "a flipped interior byte",
            "the envelope decoded as valid".to_owned(),
        )),
    }
}

/// A truncated envelope is **malformed**, and that class is derivable.
///
/// The framing header states the envelope's own total length, which is a derived
/// field of the exact encoding rather than a producer claim. No proper prefix
/// satisfies it, so a prefix long enough to carry the header is refused as a
/// total-length disagreement and a shorter one is refused as truncation. Both
/// classify as [`ArtifactCodecFailure::Malformed`], for either length, so nothing
/// about this class depends on where the cut falls.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::NotFailedClosed`] when the refusal arrives under
/// another class, or none at all.
pub(crate) fn probe_truncated_envelope(
    subject: &ProbeSubject<'_>,
) -> Result<String, EnvelopeFailure> {
    let midpoint = subject.bytes.len() / 2;
    match DecodedProgram::decode(&subject.bytes[..midpoint], SOLE_DELIVERY) {
        Err(rejection @ LoadRejection::Artifact(ArtifactCodecFailure::Malformed { .. })) => {
            Ok(format!("truncated to {midpoint} byte(s): {rejection}"))
        }
        Err(other) => Err(refused("a truncated envelope", other.to_string())),
        Ok(_) => Err(refused(
            "a truncated envelope",
            "the envelope decoded as valid".to_owned(),
        )),
    }
}

/// An artifact that is not the expected one is a **program mismatch**.
///
/// Not a variant that failed to apply, and not damage. These bytes decode and are
/// internally consistent; what is wrong is that they are some other valid
/// artifact, which is a stale cache entry or a mixed-up path rather than a plan to
/// rebuild.
///
/// The perturbation is in the *trailing* byte deliberately. A recorded identity is
/// domain-checked when it is stated, so flipping a leading byte would be refused
/// at the assertion boundary and never reach the loader — a different refusal, and
/// not the one this probe is about.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::NotFailedClosed`] when the refusal arrives under
/// another class, or none at all.
pub(crate) fn probe_foreign_expected_identity(
    subject: &ProbeSubject<'_>,
) -> Result<String, EnvelopeFailure> {
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(EnvelopeFailure::ProbeBaseline)?;
    let mut bytes = subject.expected.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    let foreign = RecordedArtifactProgramIdentity::from_bytes(&bytes)
        .map_err(|cause| EnvelopeFailure::RecordedIdentity(cause.to_string()))?;
    match decoded.preflight(subject.environment, &foreign, subject.abi) {
        Err(rejection @ LoadRejection::ProgramMismatch { .. }) => Ok(format!(
            "an expected identity that is not this artifact's: {rejection}"
        )),
        Err(other) => Err(refused("a foreign expected identity", other.to_string())),
        Ok(_) => Err(refused(
            "a foreign expected identity",
            "the route was accepted".to_owned(),
        )),
    }
}

/// Returns the sole exclusion of a one-variant artifact no eligible variant
/// survived.
///
/// The three probes below all perturb what the *host* states, and a host-relative
/// exclusion is a filter applied before any guard is evaluated rather than a
/// terminal mismatch, so that an artifact packaging plans for two backend families
/// cannot have its first plan refuse on behalf of a host the second one fits. What
/// each probe pins is therefore the exclusion the rejection carries, and the class
/// it pins in addition — that *every* packaged variant was filtered — is what says
/// the artifact offers this host nothing at all.
fn sole_exclusion<T>(
    name: &'static str,
    outcome: Result<T, LoadRejection>,
) -> Result<(VariantIneligibility, String), EnvelopeFailure> {
    match outcome {
        Err(
            ref rejection @ LoadRejection::NoEligibleVariant {
                packaged,
                ref filtered,
            },
        ) if filtered.len() == packaged => match filtered.as_slice() {
            [only] => Ok((only.reason.clone(), rejection.to_string())),
            _ => Err(refused(
                name,
                format!("this artifact packages one variant, and {filtered:?} names another"),
            )),
        },
        Err(other) => Err(refused(name, other.to_string())),
        Ok(_) => Err(refused(name, "the route was accepted".to_owned())),
    }
}

/// A host offering another profile descriptor **filters** the only variant, on
/// the profile that variant was assessed against.
///
/// Both halves of the exclusion are pinned. Which declaration excluded it
/// separates a plan assessed for another profile from an object compiled for one,
/// and those are different repairs; the classification separates the same target
/// family under a descriptor this host does not offer from an artifact built for
/// another family entirely. Asserting only that something refused would erase both
/// distinctions at the moment a caller needs them.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::NotFailedClosed`] when the exclusion is not the
/// descriptor mismatch.
pub(crate) fn probe_other_profile_descriptor(
    subject: &ProbeSubject<'_>,
) -> Result<String, EnvelopeFailure> {
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(EnvelopeFailure::ProbeBaseline)?;
    let mut descriptor = subject
        .environment
        .target_profile
        .descriptor
        .as_bytes()
        .to_vec();
    if let Some(last) = descriptor.last_mut() {
        *last ^= 0x01;
    }
    let other_host = ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: subject.environment.target_profile.key.clone(),
            descriptor: TargetProfileDescriptorDigest::from_bytes(&descriptor)
                .map_err(|_| EnvelopeFailure::HostProfile)?,
        },
        backend: subject.environment.backend.clone(),
        representation: subject.environment.representation.clone(),
        dtype_dispatch: subject.environment.dtype_dispatch.clone(),
    };
    let name = "another profile descriptor";
    let (reason, rendered) = sole_exclusion(
        name,
        decoded.preflight(&other_host, subject.expected, subject.abi),
    )?;
    match reason {
        VariantIneligibility::AssessedProfile {
            classification: tiler_runtime::load::TargetCompatibility::DescriptorMismatch { .. },
        } => Ok(format!(
            "a host offering another profile descriptor: {rendered}"
        )),
        other => Err(refused(name, other.to_string())),
    }
}

/// A host offering another profile *key* filters the only variant too, and the
/// classification names the family rather than the descriptor.
///
/// The sibling of [`probe_other_profile_descriptor`], and separate from it
/// because the two are different repairs: a key mismatch means this artifact was
/// built for another target family and the consumer should look for a different
/// artifact, while a descriptor mismatch under the same key means this family
/// under a profile revision the host does not offer and the consumer should
/// rebuild. A probe covering only the descriptor would pass while a loader that
/// compared descriptors and ignored keys admitted every foreign family.
///
/// The perturbed key is a *valid* profile key that no artifact here declares, so
/// the refusal is the classification and not a key-validation failure.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::NotFailedClosed`] when the exclusion is not the key
/// mismatch.
pub(crate) fn probe_other_profile_key(
    subject: &ProbeSubject<'_>,
) -> Result<String, EnvelopeFailure> {
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(EnvelopeFailure::ProbeBaseline)?;
    let other_host = ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: TargetProfileKey::new("tiler.metal.some-other-target-family.v1")
                .map_err(|_| EnvelopeFailure::HostProfile)?,
            descriptor: subject.environment.target_profile.descriptor.clone(),
        },
        backend: subject.environment.backend.clone(),
        representation: subject.environment.representation.clone(),
        dtype_dispatch: subject.environment.dtype_dispatch.clone(),
    };
    let name = "another profile key";
    let (reason, rendered) = sole_exclusion(
        name,
        decoded.preflight(&other_host, subject.expected, subject.abi),
    )?;
    match reason {
        VariantIneligibility::AssessedProfile {
            classification: tiler_runtime::load::TargetCompatibility::ProfileKeyMismatch { .. },
        } => Ok(format!("a host offering another profile key: {rendered}")),
        other => Err(refused(name, other.to_string())),
    }
}

/// A host stating another backend family filters the variant on the
/// **representation** it cannot execute.
///
/// Excluded on that ground rather than on the target profile it happens to share,
/// which is why this probe changes only the backend key: the host still offers the
/// exact profile the variant was assessed against, so the exclusion cannot come
/// from the compatibility classification. The entry position is pinned as well,
/// because a multi-entry route realized by two payloads must say which of them
/// this host is not.
///
/// # Errors
///
/// Returns [`EnvelopeFailure::NotFailedClosed`] when the exclusion is not the
/// unsupported representation.
pub(crate) fn probe_other_backend_family(
    subject: &ProbeSubject<'_>,
) -> Result<String, EnvelopeFailure> {
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(EnvelopeFailure::ProbeBaseline)?;
    let other_backend = ExecutionEnvironment {
        target_profile: subject.environment.target_profile.clone(),
        backend: BackendKey::new("tiler.some-other-backend")
            .map_err(|_| EnvelopeFailure::HostProfile)?,
        representation: subject.environment.representation.clone(),
        dtype_dispatch: subject.environment.dtype_dispatch.clone(),
    };
    let name = "another backend family";
    let (reason, rendered) = sole_exclusion(
        name,
        decoded.prepare(&other_backend, subject.expected, subject.abi),
    )?;
    match reason {
        VariantIneligibility::UnsupportedRepresentation { entry: 0, .. } => {
            Ok(format!("a host stating another backend family: {rendered}"))
        }
        other => Err(refused(name, other.to_string())),
    }
}

/// Proves the loader fails closed on inputs that are not this artifact.
///
/// Run against the **real** envelope a producer published, and run *before* any
/// positive route is claimed. Each probe perturbs exactly one thing and pins the
/// class of the refusal, because the failure mode this guards against is not "it
/// was accepted" — it is a refusal arriving under the wrong class. That is the
/// "corrupt artifacts must not become route misses" obligation, and it is only
/// observable by asserting the variant.
///
/// # Errors
///
/// Returns the first probe's refusal, naming the probe.
pub(crate) fn probe_fail_closed(subject: &ProbeSubject<'_>) -> Result<(), EnvelopeFailure> {
    for probe in [
        probe_accepted_baseline as fn(&ProbeSubject<'_>) -> Result<String, EnvelopeFailure>,
        probe_damaged_section_content,
        probe_damaged_interior_byte,
        probe_truncated_envelope,
        probe_foreign_expected_identity,
        probe_other_profile_key,
        probe_other_profile_descriptor,
        probe_other_backend_family,
    ] {
        eprintln!("    {}", probe(subject)?);
    }
    Ok(())
}

/// How one member's executed bytes compared against a retained measurement.
///
/// **Three facts reported together, because on a mismatch each one narrows the
/// cause and no two of them are the same claim.** `executed` is the digest of the
/// bytes this device produced and is the deliverable. `embedded` is the digest of
/// the expected bytes the *producer* published beside the artifact, and is a
/// validity condition on the fixture: it says the published record describes the
/// probe's workload rather than some other operand set. Reporting only the first
/// would leave a mismatch unable to say whether the device computed the wrong
/// answer or the record asked the wrong question.
#[derive(Clone, Debug)]
pub(crate) struct RetainedComparison {
    /// Digest of the bytes this device produced.
    pub(crate) executed: String,
    /// Digest of the producer's published expectation.
    pub(crate) embedded: String,
    /// The retained realization-probe measurement.
    pub(crate) retained: &'static str,
}

impl RetainedComparison {
    /// Whether the executed bytes carry the retained digest.
    pub(crate) fn executed_matches(&self) -> bool {
        self.executed == self.retained
    }

    /// Whether the producer's published expectation carries it too.
    pub(crate) fn embedded_matches(&self) -> bool {
        self.embedded == self.retained
    }
}

/// One published contraction member, and what its executed bytes are compared
/// against.
///
/// **The members are one route and two different claims**, which is why one path
/// drives all of them rather than two paths sharing a helper. The `2x2x3`
/// member's result has more than one row *and* more than one column, so it is the
/// one that can separate the two operand access relations, and its operand classes
/// are adversarial numerical cases with no measured device result anywhere to
/// compare against. Every L3 cell is `M = 1` or has `M != N`, so none of them can
/// separate those relations at all — and each carries the one thing the
/// adversarial member cannot: a `result_sha256` a device measured over those
/// exact operands.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ContractionMember {
    /// The class name this member is published and opened under.
    pub(crate) class: &'static str,
    /// The program family and extents [`crate::publication`] publishes it at.
    ///
    /// Carried on the member rather than resolved from the class name at
    /// publication time, so the one table below states what each member *is* and
    /// the publication has no second mapping that could disagree with it. The
    /// operand source is part of the family: the adversarial member's five cases
    /// are numerical classes chosen here, and the cell's single case is the
    /// realization probe's own workload stream.
    pub(crate) family: crate::publication::ProofFamily,
    /// The retained `direct` result digest, for a member the L3 realization probe
    /// measured.
    ///
    /// `None` is a statement rather than an omission: no measurement exists for
    /// the adversarial member's operands, so there is nothing to compare its
    /// executed bytes against beyond the published reference, and a comparison
    /// against a digest computed here would be this process checking itself.
    pub(crate) retained_result_sha256: Option<&'static str>,
}

/// Restates one L3 correctness cell as a routable member.
///
/// **Derived rather than written out, so [`L3_CORRECTNESS_CELLS`] is the single
/// authority for a cell's class, extents, and retained digest.** Seven members
/// written by hand would be a second table, and the failure it invites is a class
/// pointing at another cell's digest — which would route, agree with its own
/// published reference, and disagree with the retained value in a way that reads
/// as a device defect.
const fn l3_member(index: usize) -> ContractionMember {
    let cell = L3_CORRECTNESS_CELLS[index];
    ContractionMember {
        class: cell.class,
        family: crate::publication::ProofFamily::L3CorrectnessCell {
            m: cell.m,
            n: cell.n,
            k: cell.k,
        },
        retained_result_sha256: Some(cell.result_sha256),
    }
}

/// Every contraction member this module routes, in the order it routes them.
///
/// The adversarial `2x2x3` member leads, then the correctness cells in the
/// retained record's own order — **five of the six.** `w_vocab_slice` is absent
/// because no sidecar can carry its operand, which is
/// [`L3CorrectnessCell::fits_one_proof_payload`]; the exclusion is a hand-written
/// index here only because a `const` cannot filter, and
/// `tests::the_routed_members_are_exactly_the_publishable_cells` derives the same
/// set from that predicate and compares, so this list cannot quietly drop or
/// re-admit a cell.
///
/// The extents are checked against the operand tables and the retained
/// measurement written for them by
/// `crate::publication::proof::tests::the_published_contraction_extents_are_the_ones_this_module_is_written_for`,
/// so moving one fails in the ordinary gate rather than on the first host that
/// publishes.
pub(crate) const CONTRACTION_MEMBERS: [ContractionMember; 6] = [
    ContractionMember {
        class: CONTRACTION_CLASS,
        family: crate::publication::ProofFamily::Contraction { m: 2, n: 2, k: 3 },
        retained_result_sha256: None,
    },
    l3_member(0),
    l3_member(1),
    l3_member(2),
    l3_member(3),
    l3_member(4),
];

/// The probe's digest domain: little-endian `f32` bytes in row-major order.
///
/// The readback already yields bit patterns in the buffer's own element order, so
/// this is the identity re-encoding of the bytes the device wrote, not a
/// reinterpretation of them. Written as `to_le_bytes` rather than a raw byte copy
/// so the byte order is stated where a reader can check it against the probe's
/// host, which digests the result buffer's storage directly.
pub(crate) fn result_digest(bits: &[u32]) -> String {
    let bytes: Vec<u8> = bits.iter().flat_map(|value| value.to_le_bytes()).collect();
    sha256_hex(&bytes)
}

/// FIPS 180-4 SHA-256 over a byte string, as lowercase hexadecimal.
///
/// **Written out here rather than reached for, and the reason is domain rather
/// than preference.** `tiler-artifact` owns the governed artifact digest, but that
/// API digests under a mandatory domain separator and cannot express the raw
/// pre-image the realization probe hashed.
/// `crates/tiler-compiler/src/governed/contraction_conformance.rs` reached the
/// same conclusion for the same reason and this is the same implementation.
///
/// It is checked against the two published FIPS 180-4 vectors before any
/// comparison rests on it, by
/// `tests::the_digest_helper_reproduces_the_published_vectors`: a digest function
/// that silently computed something else would make every retained-value
/// comparison disagree, and a reader would have no way to tell that from a device
/// defect.
#[allow(
    clippy::many_single_char_names,
    reason = "the FIPS 180-4 round variables are transcribed at the names the standard gives them, and renaming them would make the transcription unverifiable against the source it comes from"
)]
pub(crate) fn sha256_hex(message: &[u8]) -> String {
    use std::fmt::Write as _;

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

/// What one routed member established.
#[derive(Clone, Debug)]
pub(crate) struct RoutedMember {
    /// The member's `class.role` name on disk.
    pub(crate) name: String,
    /// How many operand cases agreed bit for bit with the published reference.
    pub(crate) proved: usize,
    /// Dispatches the route carried.
    pub(crate) entries: usize,
    /// Shared allocations the route paired.
    pub(crate) shared: usize,
    /// The retained comparison, for a member that carries a measurement.
    pub(crate) retained: Option<RetainedComparison>,
    /// Why a member carrying a measurement was nevertheless not compared
    /// against it.
    ///
    /// **`None` and `Some` are two different absences and collapsing them is the
    /// silent skip this crate exists to refuse.** A member with no retained
    /// digest has nothing to compare; a member with one that this hardware row
    /// cannot speak for has something to compare and a stated reason not to. Only
    /// the second may leave [`Self::retained`] empty while
    /// `ContractionMember::retained_result_sha256` is `Some`, and the routed test
    /// requires the reason rather than accepting the gap.
    pub(crate) retained_declined: Option<String>,
}

/// The dtype-dispatch rows an environment states, for comparison.
///
/// Extracted so a test can compare two derivations of the same rows without
/// reaching into `ExecutionEnvironment`'s field directly at every site.
pub(crate) fn dtype_rows(
    environment: &ExecutionEnvironment,
) -> &BTreeMap<tiler_artifact::program::ArithmeticType, DTypeDispatch> {
    &environment.dtype_dispatch
}

#[cfg(target_os = "macos")]
mod apple;

#[cfg(not(target_os = "macos"))]
mod apple {
    use super::{ContractionMember, RoutedMember};
    use crate::measurement::{Measured, absent_apple_row};

    /// Reports the serial-sum matrix as unavailable.
    ///
    /// One outcome covers both halves here, and that is the shape rather than a
    /// simplification: publishing the members needs the same offline Apple
    /// toolchain routing them needs, so a host without it has nothing to say
    /// about either separately.
    pub(super) fn run_matrix() -> Measured<Vec<RoutedMember>> {
        Measured::Unavailable(absent_apple_row())
    }

    /// Reports one contraction member as unavailable.
    pub(super) fn run_contraction(_member: &ContractionMember) -> Measured<RoutedMember> {
        Measured::Unavailable(absent_apple_row())
    }
}

/// Publishes and routes every serial-sum member, or states why this host cannot.
pub(crate) fn measured_matrix() -> crate::measurement::Measured<Vec<RoutedMember>> {
    apple::run_matrix()
}

/// Publishes and routes one contraction member, or states why this host cannot.
pub(crate) fn measured_contraction(
    member: &ContractionMember,
) -> crate::measurement::Measured<RoutedMember> {
    apple::run_contraction(member)
}

#[cfg(test)]
mod tests;
