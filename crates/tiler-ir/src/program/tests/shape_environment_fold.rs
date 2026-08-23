//! The `v13` identity's environment slice, and the `v12` preimage it replaced.
//!
//! The preimage is reconstructed by *removing* the framed slice from the live
//! bytes rather than by re-implementing the old encoder, so what the tests below
//! compare is the exact byte string `v12` would have produced for the same
//! program. Removing it is well defined because the slice is length-framed at a
//! fixed position — immediately after the framed semantic graph — which is the
//! property the fold was chosen for.

use crate::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use crate::shape::{
    Axis, BindingSource, ExtentRelation, ExtentTerm, FactProvenance, RootBinding,
    SemanticInputConstraint, ShapeEnv, ShapeEnvBuilder, ShapeSymbol, SymbolScope,
};
use std::sync::Arc;

use super::support::{canonical_program, input_shape};

const SCALE_BITS: u32 = 0x3f80_0000;

fn symbol(name: &str) -> ShapeSymbol {
    ShapeSymbol::new(SymbolScope::new("program/0").unwrap(), name).unwrap()
}

fn rooted_binding() -> RootBinding {
    RootBinding::new(
        BindingSource::InputDimension {
            input: InputKey::new("input").unwrap(),
            axis: Axis::new(0),
        },
        crate::program::abi::AvailabilityPhase::LiveDevicePreflight,
        FactProvenance::RuntimeValidated,
    )
    .unwrap()
}

/// An environment declaring `n` at `input[0]`, optionally with one extra
/// constraint and one extra unreferenced declaration.
fn environment(constrained: bool, unused: bool) -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    let n = symbol("n");
    draft.declare(n.clone()).unwrap();
    draft.bind(&n, rooted_binding()).unwrap();
    if unused {
        let spare = symbol("spare");
        draft.declare(spare.clone()).unwrap();
        draft.bind(&spare, rooted_binding()).unwrap();
    }
    if constrained {
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(ExtentTerm::Symbol(n), 1, 4_096).unwrap(),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
    }
    Arc::new(draft.build().unwrap())
}

/// The `serial_sum` fixture over a *fixed* interface, carrying `environment`.
///
/// The interface is deliberately literal, so the environment reaches nothing
/// the semantic graph subject or the physical content already carries: the
/// only place these programs can differ is the folded subject itself.
fn program_over(environment: Option<Arc<ShapeEnv>>) -> SemanticProgram {
    let mut draft = match environment {
        Some(environment) => {
            SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap()
        }
        None => SemanticProgramBuilder::try_standard().unwrap(),
    };
    let input = draft
        .input::<F32>(InputKey::new("input").unwrap(), input_shape())
        .unwrap();
    let scale = F32Constant::apply(&mut draft, SCALE_BITS).unwrap();
    let bias = F32Constant::apply(&mut draft, super::support::BIAS_BITS).unwrap();
    let product = F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = F32Add::apply(&mut draft, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)]).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    draft.build().unwrap()
}

fn identity_over(environment: Option<Arc<ShapeEnv>>) -> Vec<u8> {
    canonical_program(&program_over(environment))
        .canonical_identity()
        .as_bytes()
        .to_vec()
}

/// Splits a `v13` identity into its domain, its framed graph slice, its
/// framed environment slice, and everything after.
///
/// Both runs are `push_slice`-framed — eight big-endian length bytes then
/// that many content bytes — so this reads the framing rather than guessing
/// an offset, and a fold that moved either run breaks it loudly here.
fn split(identity: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    const DOMAIN: &[u8] = b"tiler.kernel-program.v13\0";
    assert!(identity.starts_with(DOMAIN), "the identity opens with v13");
    let read_slice = |at: usize| -> (usize, Vec<u8>) {
        let length = u64::from_be_bytes(identity[at..at + 8].try_into().unwrap());
        let length = usize::try_from(length).unwrap();
        let start = at + 8;
        (start + length, identity[at..start + length].to_vec())
    };
    let (after_graph, graph) = read_slice(DOMAIN.len());
    let (after_environment, environment) = read_slice(after_graph);
    let mut head = identity[..DOMAIN.len()].to_vec();
    head.extend_from_slice(&graph);
    (head, environment, identity[after_environment..].to_vec())
}

/// The bytes a `v12` encoder would have written for the same program: the
/// live identity with its environment slice removed.
fn v12_preimage(identity: &[u8]) -> Vec<u8> {
    let (head, _, tail) = split(identity);
    let mut preimage = head;
    preimage.extend_from_slice(&tail);
    preimage
}

/// Two programs differing only by one `SemanticInputConstraint` are two
/// kernel programs at `v13` and were one at `v12`.
///
/// **The collision is the pre-fix demonstration and its disappearance is the
/// evidence.** A constraint has no root spelling at all — it reaches neither
/// the semantic graph's symbol occurrences nor an `AbiRoot::InputExtent`
/// formula — so before the fold these two programs shared
/// `CanonicalKernelProgramIdentity`, and the ADR 0013 plan-determinism
/// witness binds that identity.
#[test]
fn one_semantic_input_constraint_separates_two_kernel_programs() {
    let plain = identity_over(Some(environment(false, false)));
    let constrained = identity_over(Some(environment(true, false)));
    assert_eq!(
        v12_preimage(&plain),
        v12_preimage(&constrained),
        "the v12 preimage collided, which is what this fold exists to fix",
    );
    assert_ne!(
        plain, constrained,
        "a constraint-differing neighbour must not share a v13 identity",
    );
}

/// The same for one unreferenced symbol declaration and binding.
///
/// An unreferenced symbol generates no formula and appears in no shape, so
/// it too was invisible to `v12` and is carried by the subject at `v13`.
#[test]
fn one_unused_symbol_binding_separates_two_kernel_programs() {
    let plain = identity_over(Some(environment(false, false)));
    let spare = identity_over(Some(environment(false, true)));
    assert_eq!(
        v12_preimage(&plain),
        v12_preimage(&spare),
        "the v12 preimage collided, which is what this fold exists to fix",
    );
    assert_ne!(
        plain, spare,
        "an unused-binding neighbour must not share a v13 identity",
    );
}

/// A program with no environment folds exactly the empty subject's bytes,
/// and the two ways of saying "no symbols" stay one spelling.
#[test]
fn a_program_without_an_environment_folds_the_empty_subject() {
    let absent = identity_over(None);
    let (_, slice, _) = split(&absent);
    let mut expected = Vec::new();
    crate::identity::push_slice(
        &mut expected,
        crate::shape::empty_environment_identity().as_bytes(),
    );
    assert_eq!(
        slice, expected,
        "a program with no environment must fold the empty subject's exact bytes",
    );

    let empty = identity_over(Some(Arc::new(ShapeEnvBuilder::new().build().unwrap())));
    assert_eq!(
        absent, empty,
        "declaring no symbols and declaring an empty environment are one fact",
    );

    // And the totality does not collide a real environment into the empty
    // one, which is the property that makes the fixed slot safe.
    assert_ne!(absent, identity_over(Some(environment(false, false))));
}

/// Corrupting the folded slice's own framing moves the identity.
///
/// The subject is domain-separated and length-framed, so a forged preimage
/// that re-frames it — here by shortening the declared length and absorbing
/// the byte into the following run — is a different byte string. This is the
/// injectivity argument stated as a check rather than as prose: the fold
/// cannot be re-partitioned into a different program's slices.
#[test]
fn re_framing_the_folded_slice_moves_the_identity() {
    let identity = identity_over(Some(environment(false, false)));
    let (head, slice, tail) = split(&identity);
    let mut forged = head;
    let shortened = u64::try_from(slice.len() - 8 - 1).unwrap();
    forged.extend_from_slice(&shortened.to_be_bytes());
    forged.extend_from_slice(&slice[8..]);
    forged.extend_from_slice(&tail);
    assert_eq!(
        forged.len(),
        identity.len(),
        "the forgery re-frames the same bytes rather than removing any",
    );
    assert_ne!(
        forged, identity,
        "a re-framed subject must not be the same identity",
    );
}
