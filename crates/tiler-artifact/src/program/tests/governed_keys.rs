//! Governed key alphabets, and the bound each opaque identity takes.

use super::super::{
    ArtifactBuildError, ArtifactKeyKind, ArtifactProgramBuilder, BackendEntryKey, BackendKey,
    CapabilityFamilyKey, CompilationEnvironment, FeasibilityRuleSetKey, PayloadDigest,
    RepresentationKey, RouteFeatureKey, TargetProfileDescriptorDigest, TargetProfileKey,
};
use super::offered_physical;
use super::{
    SCALE_BITS, declare_realization, formulas, fused_program, lowering_provider, payload,
    selection, semantic_program, variant,
};
use tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES;

// -------------------------------------------------------------------------
// A governed key is spelled in one alphabet
// -------------------------------------------------------------------------

/// Every governed key refuses a byte outside the governed-key alphabet.
///
/// One case per key type rather than one case, because the six share a single
/// validator through the `governed_key!` macro and the per-type `kind` is the
/// only thing their refusals differ by. A key wired to the wrong subject would
/// report a rejection about something the producer did not write, and the
/// shared validator is exactly what stops that from being caught elsewhere.
///
/// The refused bytes are the classes the alphabet exists to exclude: case,
/// which leaves two keys a reader sees as one comparing unequal; a space, a
/// NUL, and a non-ASCII byte, which cannot be reproduced from the rejection
/// that prints them; and a separator from another naming scheme, which would
/// let one subject be spelled two ways.
#[test]
fn a_governed_key_refuses_a_byte_outside_the_governed_alphabet() {
    assert_eq!(
        BackendKey::new("tiler.Metal"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::Backend,
            index: 6,
            value: b'M',
        }),
    );
    assert_eq!(
        RepresentationKey::new("metal lib"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::Representation,
            index: 5,
            value: b' ',
        }),
    );
    assert_eq!(
        TargetProfileKey::new("tiler.target\0"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::TargetProfile,
            index: 12,
            value: 0,
        }),
    );
    assert_eq!(
        FeasibilityRuleSetKey::new("tiler/feasibility"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::FeasibilityRuleSet,
            index: 5,
            value: b'/',
        }),
    );
    assert_eq!(
        CapabilityFamilyKey::new("fusé"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::CapabilityFamily,
            index: 3,
            value: 0xc3,
        }),
    );
    assert_eq!(
        RouteFeatureKey::new("tiler.test.strict-f32!"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::RouteFeature,
            index: 21,
            value: b'!',
        }),
    );
}

/// The decoding constructor enforces the same grammar as the building one.
///
/// `from_owned` is what the decoder calls on every governed key it reads out of
/// foreign bytes, and the macro gives it its own body rather than routing it
/// through `new`. A grammar only `new` enforced would be a producer courtesy
/// instead of the boundary check this layer exists to perform.
#[test]
fn the_decoding_constructor_enforces_the_governed_alphabet() {
    assert_eq!(
        TargetProfileKey::from_owned("tiler.Target.v1".to_owned()),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::TargetProfile,
            index: 6,
            value: b'T',
        }),
    );
    BackendKey::from_owned("tiler.metal".to_owned())
        .expect("a canonically spelled key is admitted");
}

/// The empty and bound refusals fire too, and the bound stays this layer's own.
///
/// The maximum-length case is the deliberate half of the reconciliation: this
/// bound is what the artifact layer will *hold*, not what any one producer will
/// *mint*, so its own maximum is admitted rather than narrowed to the smaller
/// minting bound `tiler_compiler::target::MAX_TARGET_PROFILE_KEY_BYTES` sets.
#[test]
fn a_governed_key_refuses_an_empty_and_an_oversized_spelling() {
    assert_eq!(
        BackendKey::new(""),
        Err(ArtifactBuildError::EmptyKey {
            kind: ArtifactKeyKind::Backend,
        }),
    );
    assert_eq!(
        TargetProfileKey::new("a".repeat(super::super::MAX_GOVERNED_KEY_BYTES + 1)),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::TargetProfile,
            bytes: super::super::MAX_GOVERNED_KEY_BYTES + 1,
            limit: super::super::MAX_GOVERNED_KEY_BYTES,
        }),
    );
    TargetProfileKey::new("a".repeat(super::super::MAX_GOVERNED_KEY_BYTES))
        .expect("the admission bound admits its own maximum");
}

// -------------------------------------------------------------------------
// Received opaque identities are bounded by whoever mints them
// -------------------------------------------------------------------------

/// Each opaque identity is bounded by the authority that derives its subject.
///
/// **The over-bound vector is fabricated, and its length is derived rather than
/// measured.** It is one byte past [`super::super::MAX_OPAQUE_IDENTITY_BYTES`] — the
/// smallest length the shared bound refuses — so it states only that a
/// `BackendEntryKey` is admitted past that bound, which is this case's whole
/// subject. No kernel is involved and none can be: this crate carries no
/// `tiler-compiler` edge, for the reason stated above `[dependencies]` in its
/// manifest, so it can never compile a real reduction to measure one.
///
/// **The measured claim lives in `tiler-conformance`**, whose
/// `serial_sum::tests::the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor`
/// compiles a serial `f32` sum at one and at two contributors and asserts the
/// crossing from both sides. A length written here would restate a figure from a
/// tree that has since moved, which is what the previous `vec![0x5a; 1_121]`
/// named "measured" did: 1,121 was the two-contributor identity on 2026-07-25
/// and it measured 1,309 on 2026-08-08, while this case stayed green throughout.
///
/// The fixed-width payload digest keeps the smaller bound, while the structured
/// target-profile descriptor takes the compiler's larger minting bound.
#[test]
fn an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it() {
    BackendEntryKey::from_bytes(vec![0x5a; super::super::MAX_OPAQUE_IDENTITY_BYTES + 1])
        .expect("a backend entry key is admitted past the shared opaque-identity bound");

    assert_eq!(
        BackendEntryKey::from_bytes(vec![0x5a; MAX_KERNEL_IDENTITY_BYTES + 1]),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::BackendEntry,
            bytes: MAX_KERNEL_IDENTITY_BYTES + 1,
            limit: MAX_KERNEL_IDENTITY_BYTES,
        }),
        "beyond what the shared IR can mint, the refusal is still loud",
    );

    assert_eq!(
        PayloadDigest::from_bytes(vec![0x5a; super::super::MAX_OPAQUE_IDENTITY_BYTES + 1]),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::PayloadDigest,
            bytes: super::super::MAX_OPAQUE_IDENTITY_BYTES + 1,
            limit: super::super::MAX_OPAQUE_IDENTITY_BYTES,
        }),
    );
    assert_eq!(
        TargetProfileDescriptorDigest::from_bytes(vec![
            0x5a;
            super::super::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES
                + 1
        ]),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::TargetProfileDescriptor,
            bytes: super::super::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES + 1,
            limit: super::super::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
        }),
    );
}

/// The bound admits every entry key the packaged program itself carries.
///
/// An artifact carries one entry's kernel identity twice — as the entry key,
/// and inside the stage subject `stage_key` derives — so the two bounds have to
/// admit the same values or an artifact could be built and not encoded. This
/// asserts the first half against the second at a length the old bound refused.
///
/// That length is derived from [`super::super::MAX_OPAQUE_IDENTITY_BYTES`] rather than
/// written out, for the reason
/// [`an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it`] states:
/// the smallest refused length is the one this case wants, and a literal here
/// would be a figure about a tree rather than about the bound.
#[test]
fn an_artifact_encodes_an_entry_key_longer_than_the_digest_bound() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], offered_physical()).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let long_key = vec![0x5a; super::super::MAX_OPAQUE_IDENTITY_BYTES + 1];
    draft
        .push_variant(&program, variant(&formulas, descriptor, &long_key))
        .unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().unwrap();

    let bytes = artifact.encode().expect("the envelope encodes");
    let decoded = super::super::decode_artifact(&bytes).expect("the envelope decodes");
    assert_eq!(
        decoded
            .variants()
            .next()
            .expect("one variant")
            .entries()
            .next()
            .expect("one entry")
            .backend_entry_key()
            .as_bytes(),
        long_key.as_slice(),
    );
}
