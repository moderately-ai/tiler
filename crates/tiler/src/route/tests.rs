//! What a guarded selection does with facts and bytes it was handed.
//!
//! Every case here supplies the route facts directly, because the facts are the
//! input this module is a function of. The *correspondence* half — that an
//! expansion emits the facts of the artifact it actually produced — is the
//! frontend's obligation and is asserted in `tiler-macros`, against an artifact
//! it builds; asserting it here would need a producer this crate must not
//! depend on.
//!
//! What no case does is commit. `grep -rn "\.commit()" crates/tiler/src`
//! returns nothing, which is the exact check behind
//! [`RouteOutcome::is_fallback`] reading as a constant.

use super::{RouteFacts, RouteOutcome, bind_route_and_build, select_embedded_route};
use crate::expansion::{OperandExtent, OperandFacts, RegionFacts, ResultAxis, ResultFacts};
use crate::value::{
    AdapterCapability, BindError, OperandAxis, ResultRequest, StorageScalar, Tensor, TensorAdapter,
    ValueMetadata,
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
const IDENTITY_DOMAIN: &[u8] = b"tiler.artifact-program.v12\0";

/// A consumer-shaped value, so a region has something to bind and build.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
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
        Ok(Buffer {
            scalar: request.storage_scalar(),
            extents: request.extents().to_vec(),
        })
    }
}

fn operand(extent: u64) -> Tensor<Toy> {
    Tensor::new(
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![extent],
        },
        (),
    )
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
    }
}

/// A target matching no built family routes nothing at all.
///
/// It is checked before the bytes are looked at, which is what keeps a
/// non-Apple consumer of a `deliver macos;` region from paying a decode for an
/// artifact it will never run.
#[test]
fn a_target_with_no_payload_routes_nothing() {
    let outcome = select_embedded_route(&well_formed_facts(b"not an artifact", None));
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
    let outcome = select_embedded_route(&well_formed_facts(b"not an artifact", Some(0)));
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
    let outcome = select_embedded_route(&well_formed_facts(b"", Some(0)));
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
    let cases: [(RouteFacts, &str); 4] = [
        (
            RouteFacts {
                target_profile_key: "",
                ..well_formed_facts(b"not an artifact", Some(0))
            },
            "target profile key",
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
        4,
        "the population this test covers is every governed key a route restates, counted",
    );
    for (facts, subject) in cases {
        let outcome = select_embedded_route(&facts);
        let RouteOutcome::MalformedRouteFacts { detail } = outcome else {
            panic!("a malformed {subject} must be an expansion defect, got {outcome:?}");
        };
        assert!(
            detail.contains(subject),
            "the refusal for a malformed {subject} must name it: {detail}",
        );
    }
}

/// An identity outside the artifact identity domain is an expansion defect too,
/// and is caught before the bytes are decoded.
#[test]
fn a_foreign_recorded_identity_is_an_expansion_defect() {
    let outcome = select_embedded_route(&RouteFacts {
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
    assert_eq!(
        built,
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![4],
        },
    );
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
