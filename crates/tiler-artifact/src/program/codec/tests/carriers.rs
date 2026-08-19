//! Storage carrier and access-type pairs through the encoding and identity.

use super::super::super::model::{
    BindingKind, BindingTargetData, address_space_tag, buffer_access_tag, element_type_from_tag,
    element_type_tag, push_component_role, push_storage_encoding, storage_scalar_from_tag,
    storage_scalar_tag,
};
use super::super::super::tests::{
    bf16_pointwise_artifact, default_artifact, f32_pointwise_artifact,
};
use super::super::decode::decode;
use super::super::encode::encode;
use super::super::error::{ArtifactCodecError, TagSubject};
use super::super::model::ArtifactEnvelope;
use super::support::{encoded, envelope_of, manifest_occurrences, reseal};
use tiler_ir::kernel::{AddressSpace, BufferAccess, KernelType};
use tiler_ir::program::{StorageEncoding, StorageScalar};

// -------------------------------------------------------------------------
// BF16 through the encoding and the identity
// -------------------------------------------------------------------------

/// Moves the fixture's program input onto the `bf16` carrier, consistently.
///
/// The binding and the interface component it addresses move together, because
/// `check_target_component` refuses a binding that contradicts its component and
/// would reject a one-sided edit before any question below could be reached.
///
/// **The envelope is forged rather than built, and the producer wall that once
/// forced it is gone.** The BF16 index-realization laws are registered and
/// `Bf16NumericalContractKey` is admitted into `NumericalContractIdentity`, so a
/// `bf16` occurrence now obtains a `CoveredOccurrence` and a pure-BF16 artifact
/// is reachable through the ordinary builder — [`bf16_pointwise_artifact`],
/// whose round trip and identity are asserted in
/// [`a_producer_built_bf16_artifact_round_trips_and_re_derives_its_identity`].
///
/// What still needs a hand-built envelope is what a producer cannot emit. The
/// cases below perturb *one field at a time* on an otherwise well-formed
/// artifact — an unassigned carrier tag, an access type walked back to `F32` —
/// and each is a state the builder refuses to construct, which is the point of
/// the check. Forging from the `f32` fixture also keeps every other byte fixed,
/// so a refusal is attributable to the perturbed field rather than to the many
/// things that legitimately differ between two separately derived programs. The
/// forgery is encoded by the ordinary encoder and validated by the ordinary
/// decoder, so nothing below runs against test-only code.
fn bf16_input_envelope() -> ArtifactEnvelope {
    let mut envelope = envelope_of(&default_artifact());
    for component in &mut envelope.inputs[0].components {
        component.storage_scalar = StorageScalar::Bf16;
        component.access_type = KernelType::Bf16;
    }
    let binding = program_input_binding(&mut envelope);
    binding.storage_scalar = StorageScalar::Bf16;
    binding.access_type = KernelType::Bf16;
    envelope
}

/// Returns the one binding that addresses the fixture's named program input.
///
/// Found by target rather than by slot, because binding order is the kernel
/// signature's and an entry reordering would otherwise move these cases onto the
/// output binding without any of them failing.
fn program_input_binding(
    envelope: &mut ArtifactEnvelope,
) -> &mut super::super::super::model::BindingData {
    let mut bindings = envelope.variants[0].entries[0]
        .bindings
        .iter_mut()
        .filter(|binding| matches!(binding.target, BindingTargetData::ProgramInput(_)));
    let binding = bindings.next().expect("the fixture entry binds its input");
    assert!(
        bindings.next().is_none(),
        "the fixture entry binds its program input exactly once",
    );
    binding
}

/// Moves the fixture's program input onto the exact unsigned 32-bit pair.
fn u32_input_envelope() -> ArtifactEnvelope {
    let mut envelope = envelope_of(&default_artifact());
    for component in &mut envelope.inputs[0].components {
        component.storage_scalar = StorageScalar::U32;
        component.access_type = KernelType::U32;
    }
    let binding = program_input_binding(&mut envelope);
    binding.storage_scalar = StorageScalar::U32;
    binding.access_type = KernelType::U32;
    envelope
}

/// The exact U32/U32 pair survives the neutral artifact boundary.
#[test]
fn a_u32_carrier_round_trips_only_through_its_u32_access_type() {
    let envelope = u32_input_envelope();
    let bytes = encode(&envelope).expect("the exact U32 pair encodes");
    let decoded = decode(&bytes).expect("the exact U32 pair decodes");
    assert_eq!(decoded, envelope);
    let component = &decoded.inputs[0].components[0];
    assert_eq!(component.storage_scalar, StorageScalar::U32);
    assert_eq!(component.access_type, KernelType::U32);
    let mut decoded = decoded;
    let binding = program_input_binding(&mut decoded);
    assert_eq!(binding.storage_scalar, StorageScalar::U32);
    assert_eq!(binding.access_type, KernelType::U32);
}

/// An equal-width signed access is not an alias for unsigned storage.
#[test]
fn u32_storage_read_through_i32_is_refused_without_reinterpretation() {
    let mut envelope = u32_input_envelope();
    program_input_binding(&mut envelope).access_type = KernelType::I32;
    let bytes = encode(&envelope).expect("the forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::BindingAccessTypeMismatch),
    );
}

/// An equal-width floating access is not an alias for unsigned storage.
#[test]
fn u32_storage_read_through_f32_is_refused_without_reinterpretation() {
    let mut envelope = u32_input_envelope();
    program_input_binding(&mut envelope).access_type = KernelType::F32;
    let bytes = encode(&envelope).expect("the forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::BindingAccessTypeMismatch),
    );
}

/// A binding cannot substitute an equal-width F32 carrier for a U32 component.
#[test]
fn a_u32_component_bound_from_f32_storage_is_refused_by_component() {
    let mut envelope = u32_input_envelope();
    let binding = program_input_binding(&mut envelope);
    binding.storage_scalar = StorageScalar::F32;
    binding.access_type = KernelType::F32;
    let bytes = encode(&envelope).expect("the forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::BindingComponentMismatch)
    );
}

/// Bit-packed storage remains the U8-only carrier profile.
#[test]
fn bit_packed_u32_storage_is_refused() {
    let mut envelope = u32_input_envelope();
    program_input_binding(&mut envelope).encoding = StorageEncoding::PACKED_U4_LSB_ZERO_TAIL;
    let bytes = encode(&envelope).expect("the forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::BindingAccessTypeMismatch),
    );
}

/// Byte positions at which the carrier-only fixture pair's *envelopes* differ.
///
/// **Measured, and it is exactly the arithmetic with no digest byte
/// coinciding:** 32 for the manifest digest in the framing header, 32 for the
/// identity digest the manifest ends with, and one each for the interface
/// component's carrier and access tags and the binding row's pair. It was
/// **40** at manifest schema `14.0`, where the trailing identity *preimage*
/// restated both tag pairs in four bytes rather than being covered by a digest;
/// the step traded those four for thirty-two. A digest byte can coincide by
/// chance, so this is measured rather than asserted arithmetic, and it is
/// pinned because `docs/artifact-abi.md` states it as a measurement.
///
/// **It has been 68, then 67, then 68, then 66, and now 68 again; the
/// arithmetic never changed.**
/// The two tag pairs and the two digests are the same four and sixty-four
/// positions throughout; only how many digest bytes happen to coincide has
/// moved. `tiler.semantic-graph.v3` gave both digests content in which one byte
/// coincided, taking it to 67; `tiler.artifact-program.v16` — the derived
/// index-arithmetic requirement entering every entry row, and so entering both
/// digests — gave them content in which none does, taking it back to 68. The
/// fieldless input-role step then produced two coincidences and took it to 66;
/// the structured selected-capability subject changed both digests again and
/// returned the measured count to 68.
///
/// That the count returned to an earlier value is coincidence and not a revert:
/// nothing about the `v3` step was undone. This is exactly the chance the doc
/// comment above warns about, which is why the count is measured and never
/// derived, and why a reader must not "simplify" it to the arithmetic.
const DIFFERING_CARRIER_POSITIONS: usize = 68;

/// Byte positions at which the carrier-only fixture pair's *identities* differ.
///
/// A separate subject from [`DIFFERING_CARRIER_POSITIONS`], and deliberately
/// neither folded into it nor derived from it. That count is dominated by two
/// thirty-two-byte digests, so it moves whenever a digest byte happens to
/// coincide and its own comment above forbids reading it as arithmetic. A
/// canonical artifact identity is a *preimage* and carries no digest of itself,
/// so a carrier reaches it as the tag bytes and nothing else: the interface
/// component's carrier and access tags, and the binding row's pair. Two tag
/// pairs, four positions, with no chance term.
///
/// So this one moves only for a reason — a carrier reaching a third place in
/// the identity encoding, or ceasing to reach one of these two — which is the
/// property that would be lost if the two counts were ever one assertion.
///
/// **The byte offsets are deliberately not pinned, and must not be.** An offset
/// is a position in a layout that is free to move, and these four do not even
/// move together: `push_interface` writes the component pair near the head of
/// the encoding while the binding pair is written per variant, with the sorted
/// provider and payload keys and the whole expression arena in between. Anything
/// inserted into that span slides the binding pair and leaves the component pair
/// where it was, so a pinned offset fails for reasons that say nothing about the
/// carrier. The count is a statement about what a carrier change *means*, and
/// that is the part worth holding.
const DIFFERING_IDENTITY_POSITIONS: usize = 4;

/// A `bf16` artifact survives the encoding, and its carrier is part of what it is.
///
/// Four properties, and the third is the one a cache is wrong about if it does
/// not hold. The round trip says the carrier is not lost; the equal encoded
/// lengths say the carrier travels as a *tag* rather than as a width the framing
/// depends on, which is why widening the vocabulary moved neither
/// `ARTIFACT_DOMAIN` nor `MANIFEST_SCHEMA`; the identity inequality says two
/// artifacts differing only in their carrier are two artifacts; and the two
/// pinned counts say *how much* of each byte run a carrier reaches.
///
/// Each count is preceded by the length equality that makes it well defined:
/// counting differing positions between runs of different lengths would compare
/// a prefix and call it the whole, so the precondition is part of each property
/// rather than scaffolding for it.
#[test]
fn a_bf16_artifact_round_trips_and_its_carrier_enters_identity() {
    let at_f32 = envelope_of(&default_artifact());
    let at_bf16 = bf16_input_envelope();
    assert_ne!(at_f32, at_bf16, "the two models genuinely differ");

    let bytes = encode(&at_bf16).expect("the bf16 envelope encodes");
    let decoded = decode(&bytes).expect("its own bytes decode");
    assert_eq!(
        decoded, at_bf16,
        "a decoded bf16 envelope must equal the model that produced it",
    );

    let elements: u64 = at_bf16.inputs[0]
        .shape
        .extents()
        .iter()
        .map(|extent| extent.get())
        .product();
    assert_eq!(elements, 6, "the fixture input is the [2, 3] tensor");
    let at_f32_bytes = encode(&at_f32).expect("the f32 envelope encodes");
    assert_eq!(
        bytes.len(),
        at_f32_bytes.len(),
        "a carrier is one tag byte, so no framing width moves with it",
    );
    assert_ne!(bytes, at_f32_bytes, "the tag byte itself did move");

    // Pinned rather than described, because `docs/artifact-abi.md` carried this
    // count as prose and manifest schema `15.0` silently falsified it: the
    // trailing identity *preimage* used to restate both tag pairs, and the
    // manifest now declares its identity by digest instead. The count is
    // therefore the two tag pairs plus the two thirty-two-byte digests that
    // cover them — the manifest digest in the framing header and the identity
    // digest the manifest ends with — less whatever digest bytes coincide.
    let differing = bytes
        .iter()
        .zip(&at_f32_bytes)
        .filter(|(left, right)| left != right)
        .count();
    assert_eq!(
        differing, DIFFERING_CARRIER_POSITIONS,
        "the carrier-only byte difference moved; update the count here and the \
         measurement in docs/artifact-abi.md together",
    );

    let identity_at_bf16 = at_bf16
        .canonical_identity()
        .expect("the bf16 envelope has an identity");
    assert_eq!(
        decoded
            .canonical_identity()
            .expect("the identity re-derives from decoded content"),
        identity_at_bf16,
    );
    let identity_at_f32 = at_f32
        .canonical_identity()
        .expect("the f32 envelope has an identity");
    assert_ne!(
        identity_at_bf16, identity_at_f32,
        "two artifacts differing only in their carrier must not share an identity: a cache that \
         confused them would hand a consumer a kernel addressing twice the bytes it was given",
    );

    // The identity's own count, kept a separate subject from the envelope's
    // above. `docs/artifact-abi.md` carried this one as prose too and retired it
    // unasserted, so it is pinned here instead of restated there.
    //
    // The length equality is the precondition, not a warm-up: a positional
    // comparison between runs of different lengths counts a prefix and reports
    // it as the whole, so the count below means nothing without it. It is also a
    // property in its own right — a carrier that changed the identity's *length*
    // would be entering it as a width rather than as a tag.
    let bf16_identity_bytes = identity_at_bf16.as_bytes();
    let f32_identity_bytes = identity_at_f32.as_bytes();
    assert_eq!(
        bf16_identity_bytes.len(),
        f32_identity_bytes.len(),
        "a carrier enters the identity as a tag, so the two identity byte runs must be equal in \
         length before their differing positions can be counted at all",
    );
    let differing_identity = bf16_identity_bytes
        .iter()
        .zip(f32_identity_bytes)
        .filter(|(left, right)| left != right)
        .count();
    assert_eq!(
        differing_identity, DIFFERING_IDENTITY_POSITIONS,
        "the carrier reaches the artifact identity at the interface component's tag pair and the \
         binding row's, and nowhere else; a different count means it reaches a new place or has \
         stopped reaching one of these",
    );
}

/// A producer-built BF16 artifact survives its own codec with its identity intact.
///
/// The half of the BF16 encoding evidence that could not be produced when the
/// encoding rung landed. The artifact under test is not an `f32` envelope with
/// two tag bytes rewritten: it is derived from a pure-BF16 semantic graph whose
/// four occurrences each obtained refinement evidence under the `bf16` contract,
/// so what round-trips here is a `bf16` program rather than a `bf16`-shaped
/// reading of an `f32` one.
///
/// Four properties. The decoded envelope equals the model that produced it. Its
/// identity re-derives from the decoded content and equals the identity the
/// builder stamped — the property a cache depends on, since a hit is validated
/// by re-deriving rather than by trusting the carried bytes. Re-encoding the
/// decoded envelope reproduces the bytes exactly, which is the canonical-form
/// obligation this suite holds every artifact to. And the carrier is read back
/// off the decoded interface rather than inferred from structural equality.
#[test]
fn a_producer_built_bf16_artifact_round_trips_and_re_derives_its_identity() {
    let artifact = bf16_pointwise_artifact();
    let envelope = envelope_of(&artifact);
    let bytes = encode(&envelope).expect("the bf16 artifact encodes");
    let decoded = decode(&bytes).expect("its own bytes decode");
    assert_eq!(
        decoded, envelope,
        "a decoded bf16 envelope must equal the model that produced it",
    );
    assert_eq!(
        decoded
            .canonical_identity()
            .expect("the identity re-derives from decoded content"),
        *artifact.canonical_identity(),
        "a re-derived identity that disagreed with the stamped one would make every \
         cache hit a different artifact than the one that was stored",
    );
    assert_eq!(
        encode(&decoded).expect("the decoded envelope re-encodes"),
        bytes,
        "the encoding is canonical, so a round trip is byte-preserving",
    );

    let components = &decoded.inputs[0].components;
    assert_eq!(
        components
            .iter()
            .map(|component| (component.storage_scalar, component.access_type))
            .collect::<Vec<_>>(),
        [(StorageScalar::Bf16, KernelType::Bf16)],
        "the carrier a consumer sizes its buffer from must survive the decode",
    );

    // The contrast with the forged pair above, and the reason the four-byte
    // identity difference that rung recorded does not carry over to this one.
    // There the two envelopes were one artifact with two tag bytes rewritten, so
    // the encodings were necessarily the same length — which is what evidenced a
    // carrier travelling as a tag rather than as a width the framing depends on.
    // Here the two are separately derived from separately verified graphs whose
    // operation keys, refinement evidence, and buffer sizes all differ, so the
    // encodings are not the same length and no positional byte difference
    // between them is defined. The lengths themselves are deliberately not
    // pinned: an identity step would move them, and the numbers would carry no
    // information this inequality does not.
    let twin = encoded(&f32_pointwise_artifact());
    assert_ne!(
        bytes.len(),
        twin.len(),
        "a producer-path width change is structural, not a tag swap",
    );
}

/// An unassigned carrier or access tag is refused before its width is used.
///
/// This is the pre-BF16 reader's situation, reproduced as exactly as it can be.
/// That build cannot be run here, so the case is stated the only honest way it
/// can be: a tag byte carried by a `bf16` artifact is replaced with one *this*
/// build has not assigned, which takes the identical path `0x06` took through a
/// reader written before `0x06` existed. Both replacements are proven
/// unassigned first, so the day either is given a meaning this case fails rather
/// than quietly stopping being about an unknown tag.
///
/// The refusal is at the tag reader, so the width the tag names is never used to
/// frame or address a byte — which is the whole point of refusing, since a
/// two-byte carrier read as four addresses twice the buffer the interface
/// provides and every digest and identity check passes on the way there.
#[test]
fn an_unassigned_carrier_or_access_tag_is_refused_before_its_width_is_used() {
    const UNASSIGNED_CARRIER: u8 = 0x05;
    const UNASSIGNED_ACCESS: u8 = 0x08;

    let bytes = encode(&bf16_input_envelope()).expect("the bf16 envelope encodes");
    decode(&bytes).expect("the unperturbed bf16 envelope decodes");

    // The binding row's fixed-width head, derived through the same writers the
    // encoder uses rather than spelled as literals, so a tag that moves cannot
    // leave this locating an unrelated run of bytes.
    let mut head = vec![
        BindingKind::Buffer.tag(),
        storage_scalar_tag(StorageScalar::Bf16),
        element_type_tag(KernelType::Bf16),
    ];
    push_component_role(&mut head, None);
    push_storage_encoding(&mut head, StorageEncoding::Unpacked);
    head.push(address_space_tag(AddressSpace::Device));
    head.push(buffer_access_tag(BufferAccess::Read));
    // One occurrence, and the count is asserted rather than assumed. It was two
    // until manifest schema `15.0`: the manifest ended with the artifact
    // identity *preimage*, whose encoder restates a binding's carrier and access
    // tags in the same order the entry row writes them. The manifest now
    // declares its identity by digest, so the row is the only spelling left, and
    // it is still the one to perturb — a reader frames the entry long before it
    // reaches the identity to compare, so the refusal under test is the tag
    // reader's rather than `ArtifactIdentityMismatch`.
    let found = manifest_occurrences(&bytes, &head);
    assert_eq!(
        found.len(),
        1,
        "the binding row, with no identity preimage left to restate it",
    );
    let at = found[0];

    assert!(
        storage_scalar_from_tag(UNASSIGNED_CARRIER).is_none(),
        "the carrier vocabulary must not have grown into the tag this case perturbs to",
    );
    assert!(
        element_type_from_tag(UNASSIGNED_ACCESS).is_none(),
        "the access vocabulary must not have grown into the tag this case perturbs to",
    );

    for (offset, subject, tag) in [
        (at + 1, TagSubject::StorageScalar, UNASSIGNED_CARRIER),
        (at + 2, TagSubject::ElementType, UNASSIGNED_ACCESS),
    ] {
        let mut forged = bytes.clone();
        forged[offset] = tag;
        reseal(&mut forged);
        assert_eq!(
            decode(&forged),
            Err(ArtifactCodecError::UnknownTag { subject, tag }),
            "an unrecognized carrier or access tag must be refused by name with its own byte",
        );
    }
}

/// A `bf16` artifact read through the four-byte access type is refused.
///
/// The sibling case above states this pairing on an otherwise `f32` artifact,
/// where the carrier is the only `bf16` field present. Here the artifact really
/// is `bf16` — interface component, binding, and identity — and only the access
/// type is walked back, which is the shape a partially updated producer would
/// emit. `check_binding_access` runs ahead of the component check, so the
/// refusal names the width disagreement rather than the component it also
/// contradicts.
#[test]
fn a_bf16_artifact_read_through_the_f32_access_type_is_refused() {
    let mut envelope = bf16_input_envelope();
    program_input_binding(&mut envelope).access_type = KernelType::F32;
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::BindingAccessTypeMismatch),
        "a two-byte carrier addressed as four must be refused, not read",
    );
}
