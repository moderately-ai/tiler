//! What a guarded selection does with facts and bytes it was handed.
//!
//! Every case here supplies the route facts directly, because the facts are the
//! input this module is a function of. The *correspondence* half — that an
//! expansion emits the facts of the artifact it actually produced — is the
//! frontend's obligation and is asserted in `tiler-macros`, against an artifact
//! it builds; asserting it here would need a producer this crate must not
//! depend on.
//!
//! What no case here does is *complete* a dispatch, and the reason is stated
//! rather than left as a gap: an executable payload is a backend's, and this
//! crate depends on no backend and builds no artifact. Every case therefore
//! drives the route to a refusal — which is the whole population of pre-commit
//! outcomes — and the two post-commit outcomes are asserted against constructed
//! values, exactly as `tiler-runtime`'s own
//! `only_a_post_commit_dispatch_failure_forecloses_a_fallback` does. The
//! end-to-end evidence that an out-of-tree consumer reaches these paths at all
//! is `tests/facade/pass/inline_region_dispatches.rs`.

use super::{
    RouteFacts, RouteOutcome, bind_route_and_build, dispatch_embedded_route,
    producer_declared_equality,
};
use crate::artifact::program::ArithmeticType;
use crate::expansion::{OperandExtent, OperandFacts, RegionFacts, ResultAxis, ResultFacts};
use crate::runtime::adapter::{AdapterRouteFailure, LiveExecutionContext, RuntimeAdapter};
use crate::runtime::load::{
    DTypeDispatch, DTypeDispatchResolution, ExecutionEnvironment, LiveDeviceObservation,
    LiveDeviceRequest, Preflight, RoutedDispatch, RoutedEntry, TargetPropertyRequest,
};
use crate::value::{
    AdapterCapability, BindError, DispatchAdapter, OperandAxis, RegionRequest, ResultRequest,
    StorageScalar, Tensor, TensorAdapter, ValueMetadata,
};

/// The artifact-identity domain separator a recorded identity must open with.
///
/// Restated rather than imported, because `tiler-artifact` publishes no
/// accessor for it and this crate must not build an artifact to obtain one. The
/// restatement is safe because it is self-detecting: every case below that
/// expects to get *past* the identity restatement asserts an outcome the
/// restatement cannot produce, so a domain bump turns each of them into a
/// `MalformedRouteFacts` failure naming the identity rather than into a silent
/// pass.
const IDENTITY_DOMAIN: &[u8] = b"tiler.artifact-program.v16\0";

/// A consumer-shaped value, so a region has something to bind and build.
///
/// Carries real bytes rather than a shape alone, because a storage seam that
/// was only ever handed empty runs would not be exercised at all: every length
/// check below would pass for the same reason.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
    bytes: Vec<u8>,
}

impl Buffer {
    /// Builds one buffer whose byte run is the length its extents describe.
    fn dense(scalar: StorageScalar, extents: Vec<u64>) -> Self {
        let bytes = vec![0_u8; dense_len(scalar, &extents)];
        Self {
            scalar,
            extents,
            bytes,
        }
    }
}

/// Bytes a dense row-major value of this shape occupies, for the fixtures.
///
/// The width is read from [`StorageScalar::byte_width`], the vocabulary's single
/// width authority, rather than from a table local to these fixtures. A local
/// table would keep compiling as the carrier vocabulary widened and would state
/// a width the carrier itself does not have.
fn dense_len(scalar: StorageScalar, extents: &[u64]) -> usize {
    let width = usize::try_from(scalar.byte_width()).expect("a carrier byte width fits a usize");
    let elements: usize = extents
        .iter()
        .map(|extent| usize::try_from(*extent).expect("a fixture extent fits a usize"))
        .product();
    elements * width
}

/// A consumer-shaped adapter error.
#[derive(Debug)]
struct Refused;

impl core::fmt::Display for Refused {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("refused")
    }
}

impl std::error::Error for Refused {}

struct Toy;

impl TensorAdapter for Toy {
    type Value = Buffer;
    type Context = ();
    type Error = Refused;

    fn supports(capability: AdapterCapability) -> bool {
        match capability {
            AdapterCapability::DenseRowMajorStorage | AdapterCapability::ResultConstruction => true,
        }
    }

    fn metadata(value: &Buffer) -> Result<ValueMetadata, Refused> {
        Ok(ValueMetadata::new(
            value.scalar,
            value.extents.iter().copied(),
        ))
    }

    fn build((): &(), request: &ResultRequest<'_>) -> Result<Buffer, Refused> {
        Ok(Buffer::dense(
            request.storage_scalar(),
            request.extents().to_vec(),
        ))
    }
}

/// The device authority a `tiler`-only test consumer supplies.
///
/// It binds nothing, which is the honest answer for a crate that links no
/// backend, and it holds the region's storage so a test can observe what the
/// seam handed over. Every stage after binding is unreachable and says so: a
/// stub that returned a plausible value instead would make an ordering defect in
/// [`super::dispatch_embedded_route`] look like a passing test.
struct NoDevice<'region> {
    request: RegionRequest<'region>,
}

impl RuntimeAdapter for NoDevice<'_> {
    type Refusal = String;
    type Failure = String;
    type Completion = ();

    /// Refuses, and reports what the seam handed it while doing so.
    ///
    /// The refusal is built from the request rather than being a fixed string,
    /// which is what makes the storage handover observable through the real
    /// entry point. A constant here would let `dispatch_embedded_route` build
    /// the request wrongly — or not at all — with every case below still green.
    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, String> {
        Err(format!(
            "no backend is linked; handed {} operand(s) [{}] and result `{}` under {}",
            self.request.operands().len(),
            self.request
                .operands()
                .iter()
                .map(|operand| format!("{}={:02x?}", operand.key(), operand.bytes()))
                .collect::<Vec<_>>()
                .join(", "),
            self.request.result_key(),
            producer_declared_equality(
                self.request
                    .declared_environment()
                    .target_profile
                    .key
                    .as_str()
            ),
        ))
    }

    fn validate_payload(
        &mut self,
        _: &LiveExecutionContext,
        _: &RoutedEntry<'_>,
    ) -> Result<(), String> {
        unreachable!("no context was bound, so no payload is reachable")
    }

    fn observe_live_device(
        &mut self,
        _: &LiveExecutionContext,
        _: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        unreachable!("no context was bound, so no device row is reachable")
    }

    fn prepare_entries(
        &mut self,
        _: &LiveExecutionContext,
        _: &[RoutedEntry<'_>],
    ) -> Result<(), String> {
        unreachable!("no context was bound, so no entry is reachable")
    }

    fn observe_prepared_entry(
        &mut self,
        _: &LiveExecutionContext,
        _: TargetPropertyRequest<'_>,
    ) -> u64 {
        unreachable!("no context was bound, so no prepared entry is reachable")
    }

    fn plan_dispatch(&mut self, _: &LiveExecutionContext, _: &Preflight<'_>) -> Result<(), String> {
        unreachable!("no context was bound, so no preflight is reachable")
    }

    fn allocate_dispatch(
        &mut self,
        _: &LiveExecutionContext,
        _: &RoutedDispatch<'_>,
    ) -> Result<(), String> {
        unreachable!("no context was bound, so no route ever committed")
    }

    fn dispatch(&mut self, _: &LiveExecutionContext, _: &RoutedDispatch<'_>) -> Result<(), String> {
        unreachable!("no context was bound, so no route ever committed")
    }
}

impl DispatchAdapter for Toy {
    type Refusal = String;
    type Failure = String;
    type Dispatch<'region> = NoDevice<'region>;

    fn storage(value: &Buffer) -> Result<&[u8], Refused> {
        Ok(&value.bytes)
    }

    fn storage_mut(value: &mut Buffer) -> Result<&mut [u8], Refused> {
        Ok(&mut value.bytes)
    }

    fn dispatcher<'region>(
        (): &(),
        request: RegionRequest<'region>,
    ) -> Result<NoDevice<'region>, Refused> {
        Ok(NoDevice { request })
    }
}

fn operand(extent: u64) -> Tensor<Toy> {
    filled_operand(extent, 0)
}

/// One operand whose every byte is `fill`.
///
/// The fill is what makes a crossed or duplicated handover visible: two operands
/// of one shape are indistinguishable by length, and distinguishable by content.
fn filled_operand(extent: u64, fill: u8) -> Tensor<Toy> {
    let mut buffer = Buffer::dense(StorageScalar::F32, vec![extent]);
    buffer.bytes.fill(fill);
    Tensor::new(buffer, ())
}

/// One operand whose reported extents describe more bytes than it holds.
fn truncated_operand(extent: u64, bytes: usize) -> Tensor<Toy> {
    let mut buffer = Buffer::dense(StorageScalar::F32, vec![extent]);
    buffer.bytes.truncate(bytes);
    Tensor::new(buffer, ())
}

/// The region `in a: f32[4], b: f32[4]; out a * b`, as an expansion emits it.
const REGION: RegionFacts = RegionFacts {
    operands: &[
        OperandFacts {
            key: "a",
            storage_scalar: StorageScalar::F32,
            extents: &[OperandExtent::Literal(4)],
        },
        OperandFacts {
            key: "b",
            storage_scalar: StorageScalar::F32,
            extents: &[OperandExtent::Literal(4)],
        },
    ],
    symbols: &[],
    capabilities: &[
        AdapterCapability::DenseRowMajorStorage,
        AdapterCapability::ResultConstruction,
    ],
    result: ResultFacts {
        key: "out",
        storage_scalar: StorageScalar::F32,
        axes: &[ResultAxis::Literal(4)],
    },
};

/// The dtype-dispatchability rows a delivering expansion emits, in the shape it
/// emits them.
///
/// Two rows rather than one, and one of each verdict rather than two of the
/// same: a conversion that read the first row for every entry, or that collapsed
/// the two verdicts, would still pass a single-row fixture. The values are the
/// *shape* an expansion emits and not a claim about any profile — what the bound
/// macOS declaration actually declares is asserted in `tiler-build`, which owns
/// those rows.
const EMITTED_ROWS: &[(ArithmeticType, DTypeDispatch)] = &[
    (ArithmeticType::Bf16, DTypeDispatch::Unsupported),
    (ArithmeticType::F32, DTypeDispatch::Dispatchable),
];

/// Route facts whose every field is well formed but whose bytes are not an
/// artifact.
///
/// The identity is domain-correct so that a refusal here is the *decode*'s and
/// not the identity restatement's — without that, this case would pass for the
/// wrong reason and the `MalformedRouteFacts` cases below would be untested.
fn well_formed_facts(artifact: &'static [u8], payload: Option<usize>) -> RouteFacts {
    RouteFacts {
        artifact,
        payload,
        artifact_identity: IDENTITY_DOMAIN,
        target_profile_key: "tiler.metal.macos-apple9.msl4-0.f32.v1",
        target_profile_descriptor: b"a descriptor identity",
        backend: "tiler.metal",
        representation: "metallib",
        dtype_dispatch: EMITTED_ROWS,
    }
}

/// Drives one set of route facts through the seam with values `REGION` accepts.
///
/// The region's own obligations are discharged first by construction — two
/// `f32[4]` operands and a `f32[4]` result — so every case below observes the
/// *route*'s answer rather than a binding refusal that would have preceded it.
fn outcome(facts: &RouteFacts) -> RouteOutcome<String, String> {
    let (a, b) = (operand(4), operand(4));
    let mut result = Buffer::dense(StorageScalar::F32, vec![4]);
    dispatch_embedded_route::<Toy>(&REGION, facts, &[&a, &b], &mut result)
        .expect("the region's own values honour its declared interface")
}

/// A target matching no built family routes nothing at all.
///
/// It is checked before the bytes are looked at, which is what keeps a
/// non-Apple consumer of a `deliver macos;` region from paying a decode for an
/// artifact it will never run.
#[test]
fn a_target_with_no_payload_routes_nothing() {
    let outcome = outcome(&well_formed_facts(b"not an artifact", None));
    assert!(
        matches!(outcome, RouteOutcome::NoEmbeddedPayload),
        "unexpected outcome: {outcome:?}",
    );
    assert!(outcome.is_fallback());
}

/// Bytes that are not an artifact are the loader's refusal, not a panic and not
/// a silent pass.
#[test]
fn bytes_that_are_not_an_artifact_are_refused_before_any_device_question() {
    let outcome = outcome(&well_formed_facts(b"not an artifact", Some(0)));
    assert!(
        matches!(outcome, RouteOutcome::Refused(_)),
        "unexpected outcome: {outcome:?}",
    );
    assert!(outcome.is_fallback());
}

/// An empty envelope is refused too, and by the same class.
///
/// Paired with the case above because "not an artifact" and "nothing at all"
/// reach the decoder through different lengths, and a guard that only rejected
/// the first would still admit a consumer whose literal was emptied.
#[test]
fn an_empty_envelope_is_refused() {
    let outcome = outcome(&well_formed_facts(b"", Some(0)));
    assert!(
        matches!(outcome, RouteOutcome::Refused(_)),
        "unexpected outcome: {outcome:?}",
    );
}

/// Every malformable emitted fact is refused as an expansion defect, and the
/// refusal names which one.
///
/// Parametrized over the whole population rather than one member: each field
/// reaches a different governed constructor, and a check written against one
/// would leave the others able to panic or to be silently accepted.
#[test]
fn every_unrestatable_emitted_fact_is_an_expansion_defect() {
    // One arithmetic type carrying two verdicts. A map cannot hold both, so
    // without this case the second row would silently win and the environment
    // would state a verdict the expansion never declared twice over.
    const REPEATED: &[(ArithmeticType, DTypeDispatch)] = &[
        (ArithmeticType::F32, DTypeDispatch::Dispatchable),
        (ArithmeticType::F32, DTypeDispatch::Unsupported),
    ];
    let cases: [(RouteFacts, &str); 5] = [
        (
            RouteFacts {
                target_profile_key: "",
                ..well_formed_facts(b"not an artifact", Some(0))
            },
            "target profile key",
        ),
        (
            RouteFacts {
                dtype_dispatch: REPEATED,
                ..well_formed_facts(b"not an artifact", Some(0))
            },
            "arithmetic type",
        ),
        (
            RouteFacts {
                target_profile_descriptor: b"",
                ..well_formed_facts(b"not an artifact", Some(0))
            },
            "target profile descriptor",
        ),
        (
            RouteFacts {
                backend: "",
                ..well_formed_facts(b"not an artifact", Some(0))
            },
            "backend family",
        ),
        (
            RouteFacts {
                representation: "",
                ..well_formed_facts(b"not an artifact", Some(0))
            },
            "representation",
        ),
    ];
    assert_eq!(
        cases.len(),
        5,
        "the population this test covers is every emitted fact a route restates, counted",
    );
    for (facts, subject) in cases {
        let outcome = outcome(&facts);
        let RouteOutcome::MalformedRouteFacts { detail } = outcome else {
            panic!("a malformed {subject} must be an expansion defect, got {outcome:?}");
        };
        assert!(
            detail.contains(subject),
            "the refusal for a malformed {subject} must name it: {detail}",
        );
    }
}

/// The published environment's dtype rows are the emitted ones and nothing else.
///
/// The whole of what emitting the rows bought: the map a loader classifies
/// against is a transcription of [`RouteFacts::dtype_dispatch`], so a producer
/// whose profile refuses a dtype publishes that refusal here rather than having
/// it replaced by whatever this crate would otherwise have assumed. Both
/// verdicts are read back, because a conversion that mapped every row to
/// `Dispatchable` would be the permissive default this boundary must not have.
#[test]
fn the_published_environment_states_exactly_the_emitted_dtype_rows() {
    let environment = super::execution_environment(&well_formed_facts(b"", Some(0)))
        .expect("the fixture facts restate a governed environment");
    assert_eq!(
        environment.classify_dtype(ArithmeticType::F32),
        DTypeDispatchResolution::Dispatchable,
    );
    assert_eq!(
        environment.classify_dtype(ArithmeticType::Bf16),
        DTypeDispatchResolution::Unsupported,
        "an emitted refusal must survive as a stated negative, not be dropped into silence",
    );
    // Every type the vocabulary defines that the fixture does not emit, rather
    // than a chosen example: a conversion that seeded the map from the whole
    // vocabulary would pass a two-case check.
    for arithmetic in ArithmeticType::ALL {
        if EMITTED_ROWS.iter().any(|(row, _)| *row == arithmetic) {
            continue;
        }
        assert_eq!(
            environment.classify_dtype(arithmetic),
            DTypeDispatchResolution::Unknown,
            "{} is not an emitted row and must not acquire a verdict here",
            arithmetic.canonical_type_key(),
        );
    }
}

/// An expansion that emitted no dtype row publishes a host that dispatches
/// nothing.
///
/// The fail-closed direction, asserted rather than left to follow from the map
/// being empty: `RouteFacts::dtype_dispatch` is a slice an expansion fills, and
/// the honest answer for a producer that declared nothing is a host that routes
/// nothing — never a permissive default supplied here because the slice was
/// short.
#[test]
fn an_expansion_that_emits_no_dtype_row_publishes_a_host_that_dispatches_nothing() {
    let environment = super::execution_environment(&RouteFacts {
        dtype_dispatch: &[],
        ..well_formed_facts(b"", Some(0))
    })
    .expect("an empty rows literal is well formed");
    for arithmetic in ArithmeticType::ALL {
        assert!(
            !environment.classify_dtype(arithmetic).is_dispatchable(),
            "{} must not be dispatchable on a host that was told nothing",
            arithmetic.canonical_type_key(),
        );
    }
}

/// An identity outside the artifact identity domain is an expansion defect too,
/// and is caught before the bytes are decoded.
#[test]
fn a_foreign_recorded_identity_is_an_expansion_defect() {
    let outcome = outcome(&RouteFacts {
        artifact_identity: b"not this build's identity domain",
        ..well_formed_facts(b"not an artifact", Some(0))
    });
    let RouteOutcome::MalformedRouteFacts { detail } = outcome else {
        panic!("unexpected outcome: {outcome:?}");
    };
    assert!(detail.contains("artifact identity"), "{detail}");
}

/// The region's own obligations are checked before the artifact is, so a
/// consumer that supplied the wrong values reads about its values.
///
/// Without this ordering a mismatched operand under a damaged artifact would
/// report the artifact, which is not the thing the consumer can fix. The
/// perturbation is the operand's declared literal *extent*, which is the whole
/// of the region's shape claim here — `REGION` declares `f32[4]` twice and names
/// no symbol, so an extent this artifact was never compiled for is precisely
/// what a routed dispatch would launch against.
#[test]
fn the_region_contract_is_checked_before_the_artifact() {
    let (a, b) = (operand(4), operand(7));
    let refusal = bind_route_and_build(
        &REGION,
        &well_formed_facts(b"not an artifact", Some(0)),
        &[&a, &b],
    )
    .expect_err("the second operand does not report the declared extent");
    assert!(
        matches!(
            refusal,
            BindError::LiteralExtentMismatch {
                axis: OperandAxis {
                    input: "b",
                    axis: 0,
                },
                declared: 4,
                actual: 7,
            },
        ),
        "unexpected refusal: {refusal}",
    );
}

/// A refused artifact leaves the region on its semantic fallback with a value,
/// rather than turning into an error the consumer must handle.
///
/// This is the half that makes "fallback authority" mean something: the guard
/// said no, and the region still produced the result it declared.
#[test]
fn a_refused_artifact_still_produces_the_declared_result() {
    let (a, b) = (operand(4), operand(4));
    let built = bind_route_and_build(
        &REGION,
        &well_formed_facts(b"not an artifact", Some(0)),
        &[&a, &b],
    )
    .expect("a refused artifact is a fallback, not a region failure");
    assert_eq!(built, Buffer::dense(StorageScalar::F32, vec![4]));
}

/// A target with no payload also produces the declared result.
///
/// The paired positive of the case above: the two reach the fallback through
/// different outcomes, and a `bind_route_and_build` that only handled one would
/// still pass the other.
#[test]
fn a_target_with_no_payload_still_produces_the_declared_result() {
    let (a, b) = (operand(4), operand(4));
    let built = bind_route_and_build(&REGION, &well_formed_facts(b"", None), &[&a, &b])
        .expect("a target matching no family takes the fallback");
    assert_eq!(built.extents, vec![4]);
}

/// A request looks its operands up by interface key and by nothing else.
///
/// **This is a test of the request, not of the route that builds one.** The
/// value is constructed here rather than obtained from
/// [`dispatch_embedded_route`], because reaching the adapter needs an artifact
/// that decodes and this crate builds none — so a `dispatch_embedded_route` that
/// paired every operand with the wrong key would still pass this. That pairing
/// is `tests/facade/pass/inline_region_dispatches.rs`'s to check, against the
/// artifact the macro compiled, and it was watched failing for exactly this
/// perturbation.
///
/// The assertion is on the *contents* rather than on the lengths, because two
/// operands of one shape have equal lengths and a lookup that returned the same
/// run twice would still pass a length check. Distinct fill bytes are what make
/// a crossed pairing visible at all.
#[test]
fn a_request_looks_its_operands_up_by_interface_key() {
    let a = filled_operand(4, 0xA1);
    let b = filled_operand(4, 0xB2);
    let mut result = Buffer::dense(StorageScalar::F32, vec![4]);

    let request = RegionRequest::new(
        vec![
            crate::value::RegionOperand::new("a", &a.value().bytes),
            crate::value::RegionOperand::new("b", &b.value().bytes),
        ],
        "out",
        &mut result.bytes,
        super::execution_environment(&well_formed_facts(b"", Some(0)))
            .expect("the fixture facts restate a governed environment"),
    );

    assert_eq!(request.operand("a"), Some([0xA1_u8; 16].as_slice()));
    assert_eq!(request.operand("b"), Some([0xB2_u8; 16].as_slice()));
    assert_eq!(request.operand("c"), None, "the region declares no `c`");
    assert_eq!(request.result_key(), "out");
    assert_eq!(
        request.declared_environment().backend.as_str(),
        "tiler.metal",
        "the environment handed over is the producer's declaration",
    );
}

/// A value whose byte run is shorter than its own extents describe is refused
/// before any of those bytes reach a kernel.
///
/// The perturbation is the *storage*, not the shape: the adapter still reports
/// `f32[4]`, so every check that preceded this one still passes and only the
/// length disagrees. Without this, a dispatch would derive a four-element launch
/// against sixteen bytes that are not there.
#[test]
fn storage_shorter_than_the_extents_it_reports_is_refused() {
    let a = truncated_operand(4, 12);
    let b = operand(4);
    let mut result = Buffer::dense(StorageScalar::F32, vec![4]);

    let refusal = dispatch_embedded_route::<Toy>(
        &REGION,
        &well_formed_facts(b"not an artifact", Some(0)),
        &[&a, &b],
        &mut result,
    )
    .expect_err("a short byte run is not a route outcome");

    assert!(
        matches!(
            refusal,
            BindError::StorageLengthMismatch {
                input: "a",
                declared: 16,
                actual: 12,
            },
        ),
        "unexpected refusal: {refusal}",
    );
}

/// The region's *result* storage is length-checked too, and by the same rule.
///
/// Paired with its operand neighbour because the two reach the check through
/// different borrows — shared for an operand, exclusive for the result — and a
/// check written for one would not run for the other.
#[test]
fn a_result_shorter_than_its_declared_shape_is_refused() {
    let (a, b) = (operand(4), operand(4));
    let mut result = Buffer::dense(StorageScalar::F32, vec![4]);
    result.bytes.truncate(4);

    let refusal = dispatch_embedded_route::<Toy>(
        &REGION,
        &well_formed_facts(b"not an artifact", Some(0)),
        &[&a, &b],
        &mut result,
    )
    .expect_err("a short result run is not a route outcome");

    assert!(
        matches!(
            refusal,
            BindError::StorageLengthMismatch {
                input: "out",
                declared: 16,
                actual: 4,
            },
        ),
        "unexpected refusal: {refusal}",
    );
}

/// Whether a region fell back is no longer one answer for every outcome.
///
/// Asserted over the written-out population rather than over whichever outcomes
/// the cases above happen to produce, because the property that matters is the
/// *split*: three loader-side outcomes and every pre-commit adapter stage fall
/// back, and the two outcomes reached at or after the commit do not. A
/// `is_fallback` that regressed to a constant passes no half of this.
#[test]
fn the_fallback_answer_is_no_longer_a_constant() {
    let fall_back: [RouteOutcome<&str, &str>; 8] = [
        RouteOutcome::NoEmbeddedPayload,
        RouteOutcome::MalformedRouteFacts { detail: "any" },
        RouteOutcome::Refused(
            crate::runtime::load::DecodedProgram::decode(b"short", 0)
                .expect_err("five bytes are not an artifact"),
        ),
        RouteOutcome::Adapter(AdapterRouteFailure::Context("no device")),
        RouteOutcome::Adapter(AdapterRouteFailure::Payload {
            entry: 0,
            refusal: "truncated image",
        }),
        RouteOutcome::Adapter(AdapterRouteFailure::Preparation("no pipeline")),
        RouteOutcome::Adapter(AdapterRouteFailure::Plan("storage too small")),
        RouteOutcome::Adapter(AdapterRouteFailure::Load(
            crate::runtime::load::DecodedProgram::decode(b"", 0)
                .expect_err("no bytes are not an artifact"),
        )),
    ];
    assert_eq!(
        fall_back.len(),
        8,
        "the population this test covers is every outcome reached before the commit, counted",
    );
    for outcome in &fall_back {
        assert!(
            outcome.is_fallback(),
            "{outcome:?} is reached before the commit and leaves the region on its fallback",
        );
    }

    let committed: [RouteOutcome<&str, &str>; 2] = [
        RouteOutcome::Dispatched,
        RouteOutcome::Adapter(AdapterRouteFailure::Dispatch("submission did not complete")),
    ];
    for outcome in &committed {
        assert!(
            !outcome.is_fallback(),
            "{outcome:?} is reached at or after the commit, so no fallback follows it",
        );
    }
}

/// A committed dispatch that did not complete is a refusal, never the fallback's
/// value wearing the dispatch's clothes.
///
/// The outcome is constructed rather than routed to, because reaching a real
/// post-commit failure needs an executable payload this crate cannot build. What
/// is checked is the conversion `bind_route_and_build` performs, which is the
/// part that could return an incorrect tensor.
#[test]
fn a_post_commit_failure_is_reported_rather_than_returned_as_a_result() {
    let failure: AdapterRouteFailure<&str, &str> =
        AdapterRouteFailure::Dispatch("the command buffer ended in Error");
    assert!(
        !failure.fallback_permitted(),
        "this fixture is only meaningful for an outcome that forecloses a fallback",
    );
    let rendered = failure.to_string();
    let refusal: BindError<Refused> = BindError::DispatchFailed {
        detail: rendered.clone(),
    };
    assert!(
        refusal.to_string().contains(&rendered),
        "the refusal must carry the adapter's own account: {refusal}",
    );
    assert!(
        refusal.to_string().contains("ADR 0051"),
        "the refusal must say why no fallback follows: {refusal}",
    );
}

/// The labelled diagnostic renders for a profile key and keeps both halves.
///
/// Both halves, because the sentence is only true with the negation in it: a
/// paraphrase that kept "producer-declared equality" and dropped "NOT
/// host-earned eligibility" would read as the opposite claim.
#[test]
fn the_labelled_diagnostic_names_the_profile_and_keeps_its_negation() {
    let rendered = producer_declared_equality("tiler.metal.macos-apple9.msl4-0.f32.v1");
    assert!(
        rendered.contains("tiler.metal.macos-apple9.msl4-0.f32.v1"),
        "the label must name the profile the route was settled against: {rendered}",
    );
    assert!(
        rendered.contains("producer-declared equality"),
        "{rendered}",
    );
    assert!(
        rendered.contains("NOT host-earned eligibility"),
        "{rendered}",
    );
    assert!(
        !rendered.contains("{}"),
        "the placeholder must be substituted rather than printed: {rendered}",
    );
}
