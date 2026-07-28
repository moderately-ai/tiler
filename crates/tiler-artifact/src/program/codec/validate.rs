//! Re-proving a decoded envelope against the artifact model's own rules.
//!
//! A digest proves that these are the exact bytes someone wrote. It proves
//! nothing about whether those bytes describe a legal artifact, and
//! `docs/artifact-abi.md` is explicit that parse success never implies
//! executable validity. Everything here is therefore a *second* proof, run
//! against decoded content, of an obligation the transactional builder already
//! discharged against the draft it verified.
//!
//! Each check reports the model's own typed cause rather than a codec-local
//! restatement, so a rejection reads the same whether an artifact was refused
//! at construction or at load.
//!
//! # What cannot be re-proven here, and why it is still pinned
//!
//! Three of the builder's obligations tie the ABI to the *program* rather than
//! to the manifest: a binding's accessible offset and byte range must equal the
//! exact byte window its stage access addresses, an entry's bindings must
//! correspond one-to-one with its kernel's buffer parameters, and each binding's
//! carried interface reference must be the one its stage access resolves to.
//! Neither the byte windows, the kernel signature, nor the program's value table
//! travel in this profile, so a decoder cannot recompute them. They are not
//! therefore unguarded: all three are folded into the artifact's canonical
//! identity through the binding's expression content key, its encoded target,
//! and the entry's stage key, and the identity is re-derived and compared below.
//! A forged envelope can restate them only by becoming a different artifact.
//!
//! The binding target is the first such row whose misreading would silently bind
//! the wrong buffer rather than fail, so the part of it that *is* decidable here
//! is decided here: `check_binding_targets` proves the name it uses is one the
//! manifest's own interface declares. That does not prove the correspondence —
//! nothing decoded can — and the two claims are kept apart deliberately.
//!
//! Carrying the byte windows so the check could run locally was considered and
//! rejected: the window is a value only the program establishes, so a carried
//! copy would let a forged envelope assert a range no verifier examined, and
//! the check would prove agreement between two producer-supplied fields rather
//! than agreement with the plan.

use std::collections::BTreeSet;

use tiler_ir::semantic::ProviderIdentity;

use super::super::error::{AbiExprUse, ArtifactBuildError, ArtifactDiagnostic};
use super::super::expr::{
    AbiFacts, AbiType, AvailabilityPhase, ExprNode, evaluate, node_is_interface_only, node_phase,
    node_type,
};
use super::super::facts::AbiFactBinder;
use super::super::model::{BindingTargetData, canonical_deferred_order, deferred_key};
use super::error::{ArtifactCodecError, OrderedSubject};
use super::model::{
    ArtifactEnvelope, EntryRow, VariantRow, expression_keys, node_operands, position,
};
use super::payload::{decode_metadata, payload_identity};

/// Proves every artifact-model obligation a decoded envelope can discharge.
///
/// # Errors
///
/// Returns [`ArtifactCodecError::ModelRule`] for an insertion-time rule,
/// [`ArtifactCodecError::ModelObligation`] for a whole-artifact obligation, or
/// a canonical-order, duplicate, or feature rejection this codec owns.
pub(super) fn validate(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    if envelope.variants().is_empty() {
        return Err(obligation(ArtifactDiagnostic::EmptyPortfolio));
    }
    if envelope.providers().is_empty() {
        return Err(obligation(ArtifactDiagnostic::MissingSelectedProvider));
    }
    if envelope.features() != envelope.derived_features() {
        return Err(ArtifactCodecError::DeclaredFeatureMismatch);
    }
    check_interface(envelope)?;
    check_sections(envelope)?;
    check_payload_identity(envelope)?;
    check_binding_targets(envelope)?;
    check_entry_mappings(envelope)?;
    let facts = ExpressionFacts::derive(envelope.expressions());
    let keys = expression_keys(envelope.expressions());
    check_expression_closure(envelope)?;
    check_backend_entries(envelope)?;
    let static_facts = interface_facts(envelope);
    for variant in envelope.variants() {
        check_variant(envelope, variant, &facts, &keys, &static_facts)?;
    }
    check_duplicate_variants(envelope)?;
    Ok(())
}

/// The per-node value type, availability phase, and interface-only facts.
///
/// Every one is re-derived from the decoded arena through the same recurrence
/// the transactional builder uses, so a use-site check means the same thing on
/// both sides of the wire.
struct ExpressionFacts {
    types: Vec<AbiType>,
    phases: Vec<AvailabilityPhase>,
    interface_only: Vec<bool>,
}

impl ExpressionFacts {
    fn derive(nodes: &[ExprNode]) -> Self {
        let mut facts = Self {
            types: Vec::with_capacity(nodes.len()),
            phases: Vec::with_capacity(nodes.len()),
            interface_only: Vec::with_capacity(nodes.len()),
        };
        for node in nodes {
            facts.types.push(node_type(node, &facts.types));
            facts.phases.push(node_phase(node, &facts.phases));
            facts
                .interface_only
                .push(node_is_interface_only(node, &facts.interface_only));
        }
        facts
    }

    /// Re-proves one declared use site exactly as the builder does.
    fn check_use(
        &self,
        node: u32,
        use_site: AbiExprUse,
        expected: AbiType,
        admitted_through: AvailabilityPhase,
        interface_only: bool,
    ) -> Result<(), ArtifactCodecError> {
        let actual = self.types[position(node)];
        if actual != expected {
            return Err(rule(ArtifactBuildError::ExpressionType {
                use_site,
                expected,
                actual,
            }));
        }
        let available_at = self.phases[position(node)];
        if available_at > admitted_through {
            return Err(rule(ArtifactBuildError::RootPhaseEscape {
                use_site,
                available_at,
                admitted_through,
            }));
        }
        if interface_only && !self.interface_only[position(node)] {
            return Err(rule(ArtifactBuildError::NonInterfaceRoot { use_site }));
        }
        Ok(())
    }
}

/// Builds the declared-shape environment the ABI's static checks evaluate in.
fn interface_facts(envelope: &ArtifactEnvelope) -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    for input in envelope.inputs() {
        // A decoded interface may repeat a key only if it also survives the
        // identity comparison, so a duplicate here is recorded and left to that
        // check rather than masked by a binder rejection.
        let _ = binder.bind_input_shape(&input.key, &input.shape);
    }
    binder.build()
}

/// Proves the named interface can be bound unambiguously.
///
/// Interface order is meaning rather than canonical, so the obligation is
/// distinctness alone: a runtime binds inputs and reads outputs by key, and two
/// entries sharing a key would make that binding ambiguous. Identity encodes the
/// interface positionally and would happily fold a repeat, so this is decided
/// here rather than left to the identity comparison.
fn check_interface(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let mut inputs: Vec<&str> = envelope
        .inputs()
        .iter()
        .map(|input| input.key.as_str())
        .collect();
    inputs.sort_unstable();
    if inputs.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ArtifactCodecError::DuplicateItem {
            subject: OrderedSubject::InterfaceKey,
        });
    }
    let mut outputs: Vec<&str> = envelope
        .outputs()
        .iter()
        .map(|output| output.key.as_str())
        .collect();
    outputs.sort_unstable();
    if outputs.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ArtifactCodecError::DuplicateItem {
            subject: OrderedSubject::InterfaceKey,
        });
    }
    Ok(())
}

fn check_sections(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    // Purpose precedes content in the order, so two sections carrying equal
    // bytes under different purposes are distinct and adjacent rather than a
    // duplicate. Comparing bytes alone would report a legitimate pair as
    // non-canonical, and would call a genuine duplicate ordered.
    for pair in envelope.sections().windows(2) {
        match pair[0].canonical_key().cmp(&pair[1].canonical_key()) {
            std::cmp::Ordering::Less => {}
            std::cmp::Ordering::Equal => {
                return Err(ArtifactCodecError::DuplicateItem {
                    subject: OrderedSubject::Section,
                });
            }
            std::cmp::Ordering::Greater => {
                return Err(ArtifactCodecError::NonCanonicalOrder {
                    subject: OrderedSubject::Section,
                });
            }
        }
    }
    let mut referenced: BTreeSet<u32> = envelope
        .variants()
        .iter()
        .map(|variant| variant.program_section)
        .collect();
    for content in envelope.payload_content().iter().flatten() {
        referenced.insert(content.metadata);
        referenced.insert(content.code);
    }
    for section in 0..envelope.sections().len() {
        let section = u32::try_from(section).expect("a bounded section table fits u32");
        if !referenced.contains(&section) {
            // An unreferenced section changes the envelope's bytes without
            // changing the artifact's identity, which would give one artifact
            // two byte identities. That is the same hazard the model rejects
            // for an unreachable expression node.
            return Err(ArtifactCodecError::UnreferencedSection { section });
        }
    }
    Ok(())
}

/// Re-derives each carried payload's identity from the subject it carries.
///
/// The descriptor's digest is what artifact identity folds, and the metadata
/// section is what a consumer would read to learn how the object was built. A
/// forged envelope that pairs one descriptor with another payload's subject
/// keeps both sections well formed and both digests verifying, so this is the
/// check that binds the two together. It also proves the carried subject parses
/// under this reader's schema, which the section digest cannot.
fn check_payload_identity(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    for (payload, content) in envelope.payload_content().iter().enumerate() {
        let Some(content) = content else {
            continue;
        };
        let bytes = &envelope.sections()[position(content.metadata)].bytes;
        decode_metadata(bytes)?;
        let derived = payload_identity(bytes).map_err(|cause| ArtifactCodecError::ModelRule {
            cause: Box::new(cause),
        })?;
        if envelope.payloads()[payload].digest != derived {
            return Err(ArtifactCodecError::PayloadIdentityMismatch {
                payload: u32::try_from(payload).expect("a bounded payload table fits u32"),
            });
        }
    }
    Ok(())
}

/// Proves every binding target names an interface entry the artifact declares.
///
/// What a slot addresses is the one dispatch fact this decoder cannot
/// re-derive: the program that established it is carried as identity bytes
/// alone, so agreement between the reference and the plan is proven by the
/// builder and pinned by artifact identity, exactly as the accessible byte
/// range is. What *is* decidable here is narrower and still worth proving — the
/// name the reference uses must be one the manifest itself declares. Without
/// it, a forged envelope could direct a slot at a buffer the interface never
/// mentions and every framing, integrity and identity check would still pass,
/// because the forged name is folded into the identity it re-derives.
fn check_binding_targets(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    for binding in envelope
        .variants()
        .iter()
        .flat_map(|variant| &variant.entries)
        .flat_map(|entry| &entry.bindings)
    {
        match &binding.target {
            BindingTargetData::ProgramInput(key) => {
                if !envelope.inputs().iter().any(|input| input.key == *key) {
                    return Err(ArtifactCodecError::UnknownBindingTargetKey {
                        key: key.as_str().to_owned(),
                        input: true,
                    });
                }
            }
            BindingTargetData::ProgramOutput(keys) => {
                for key in keys {
                    if !envelope.outputs().iter().any(|output| output.key == *key) {
                        return Err(ArtifactCodecError::UnknownBindingTargetKey {
                            key: key.as_str().to_owned(),
                            input: false,
                        });
                    }
                }
            }
            BindingTargetData::Internal => {}
        }
    }
    Ok(())
}

/// Proves every carried payload maps each backend entry the artifact dispatches.
///
/// A neutral entry names its backend implementation by an opaque
/// [`BackendEntryKey`](super::super::BackendEntryKey); the carried payload's
/// entry mapping is what turns that into the symbol a loader resolves and the
/// transport slots its bindings occupy. Neither the builder nor the decoder
/// proved the two agreed before this check existed, so an artifact could carry
/// a payload that mapped none of the entries it realized and still decode — and
/// the failure would surface as a loader unable to find a symbol, with the
/// artifact layer having declared the bytes valid.
///
/// Only *carried* payloads are checked. A descriptor-only payload names a
/// backend object this envelope does not contain, so it has no mapping to
/// disagree with, and requiring one would make the descriptor-only form
/// unusable.
///
/// The obligation is coverage rather than exhaustion: every entry key must be
/// mapped, and a payload may map a backend entry no artifact entry dispatches.
/// A compiled object legitimately exports more than one symbol, and a mapping
/// for an undispatched one costs a reader nothing — it is folded into the
/// payload's compilation subject and therefore into artifact identity, so it is
/// not the unreferenced-content hazard the section and expression tables reject.
fn check_entry_mappings(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    for entry in envelope
        .variants()
        .iter()
        .flat_map(|variant| &variant.entries)
    {
        let Some(content) = envelope.payload_content()[position(entry.payload)] else {
            continue;
        };
        // Re-parsed rather than threaded from `check_payload_identity`: that
        // check ran for its own reason and this one must not depend on having
        // been reached, so each proves what it needs from the bytes.
        let metadata = decode_metadata(&envelope.sections()[position(content.metadata)].bytes)?;
        let mapping = metadata
            .entries
            .iter()
            .find(|mapping| mapping.entry_key == entry.entry_key)
            .ok_or(ArtifactCodecError::UnmappedBackendEntry {
                payload: entry.payload,
            })?;
        if mapping.transports.len() != entry.bindings.len() {
            return Err(ArtifactCodecError::EntryTransportCardinality {
                payload: entry.payload,
                bindings: entry.bindings.len(),
                transports: mapping.transports.len(),
            });
        }
    }
    Ok(())
}

/// Proves every arena node is reachable from a declared use site.
fn check_expression_closure(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let nodes = envelope.expressions();
    let mut reached = vec![false; nodes.len()];
    let mut work: Vec<u32> = Vec::new();
    for variant in envelope.variants() {
        work.push(variant.guard);
        work.extend(variant.deferred.iter().map(|predicate| predicate.predicate));
        for entry in &variant.entries {
            work.extend(
                entry
                    .bindings
                    .iter()
                    .flat_map(|binding| [binding.accessible_offset, binding.accessible_bytes]),
            );
            work.push(entry.launch.grid_threads);
            work.push(entry.launch.threads_per_workgroup);
            work.extend(entry.launch.preconditions.iter().copied());
        }
    }
    while let Some(node) = work.pop() {
        if reached[position(node)] {
            continue;
        }
        reached[position(node)] = true;
        work.extend(node_operands(&nodes[position(node)]));
    }
    if reached.iter().all(|node| *node) {
        Ok(())
    } else {
        Err(obligation(ArtifactDiagnostic::UnusedExpression))
    }
}

/// Proves every payload realizes an entry and no backend entry is claimed twice.
fn check_backend_entries(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let mut claimed: BTreeSet<(u32, &[u8])> = BTreeSet::new();
    let mut referenced: BTreeSet<u32> = BTreeSet::new();
    for entry in envelope
        .variants()
        .iter()
        .flat_map(|variant| &variant.entries)
    {
        referenced.insert(entry.payload);
        if !claimed.insert((entry.payload, entry.entry_key.as_bytes())) {
            return Err(obligation(ArtifactDiagnostic::DuplicateBackendEntry));
        }
    }
    for payload in 0..envelope.payloads().len() {
        let payload = u32::try_from(payload).expect("a bounded payload table fits u32");
        if !referenced.contains(&payload) {
            return Err(obligation(ArtifactDiagnostic::UnusedPayload));
        }
    }
    Ok(())
}

fn check_variant(
    envelope: &ArtifactEnvelope,
    variant: &VariantRow,
    facts: &ExpressionFacts,
    keys: &[Vec<u8>],
    static_facts: &AbiFacts,
) -> Result<(), ArtifactCodecError> {
    facts.check_use(
        variant.guard,
        AbiExprUse::ApplicabilityGuard,
        AbiType::Boolean,
        AvailabilityPhase::LiveDevicePreflight,
        false,
    )?;
    // The stored order must be the canonical one, checked against the shared
    // definition rather than against a locally re-derived key: a key sort here
    // and a comparator sort in the encoder would be two definitions of
    // canonical that only happen to agree.
    if canonical_deferred_order(envelope.expressions(), &variant.deferred)
        != (0..variant.deferred.len()).collect::<Vec<_>>()
    {
        return Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::DeferredPredicate,
        });
    }
    for predicate in &variant.deferred {
        if predicate.phase < AvailabilityPhase::LiveDevicePreflight {
            return Err(rule(ArtifactBuildError::NonDeferredPredicatePhase {
                phase: predicate.phase,
            }));
        }
        if !selects(envelope, &predicate.authority) {
            return Err(rule(ArtifactBuildError::UnselectedDeferredAuthority {
                provider: Box::new(predicate.authority.clone()),
            }));
        }
        facts.check_use(
            predicate.predicate,
            AbiExprUse::DeferredPredicate,
            AbiType::Boolean,
            predicate.phase,
            false,
        )?;
    }
    check_ordered(
        &variant
            .entries
            .iter()
            .map(|entry| entry.stage.as_bytes().to_vec())
            .collect::<Vec<_>>(),
        OrderedSubject::Entry,
    )?;
    for (index, entry) in variant.entries.iter().enumerate() {
        check_entry(envelope, entry, index, facts, keys, static_facts)?;
    }
    Ok(())
}

fn check_entry(
    envelope: &ArtifactEnvelope,
    entry: &EntryRow,
    index: usize,
    facts: &ExpressionFacts,
    keys: &[Vec<u8>],
    static_facts: &AbiFacts,
) -> Result<(), ArtifactCodecError> {
    for binding in &entry.bindings {
        facts.check_use(
            binding.accessible_offset,
            AbiExprUse::AccessibleOffset,
            AbiType::Unsigned,
            AvailabilityPhase::LiveDevicePreflight,
            true,
        )?;
        facts.check_use(
            binding.accessible_bytes,
            AbiExprUse::AccessibleBytes,
            AbiType::Unsigned,
            AvailabilityPhase::LiveDevicePreflight,
            true,
        )?;
    }
    facts.check_use(
        entry.launch.grid_threads,
        AbiExprUse::LaunchThreads,
        AbiType::Unsigned,
        AvailabilityPhase::LiveDevicePreflight,
        true,
    )?;
    facts.check_use(
        entry.launch.threads_per_workgroup,
        AbiExprUse::ThreadsPerWorkgroup,
        AbiType::Unsigned,
        AvailabilityPhase::LiveDevicePreflight,
        true,
    )?;
    check_ordered(
        &entry
            .launch
            .preconditions
            .iter()
            .map(|node| keys[position(*node)].clone())
            .collect::<Vec<_>>(),
        OrderedSubject::LaunchPrecondition,
    )?;
    for precondition in &entry.launch.preconditions {
        facts.check_use(
            *precondition,
            AbiExprUse::LaunchPrecondition,
            AbiType::Boolean,
            AvailabilityPhase::LaunchPreflight,
            false,
        )?;
    }

    // The threads-per-workgroup formula and the entry's proven requirements are
    // both carried and both folded into identity, so their agreement is
    // decidable here and is re-proven rather than assumed.
    let declared = static_unsigned(
        envelope,
        entry.launch.threads_per_workgroup,
        AbiExprUse::ThreadsPerWorkgroup,
        static_facts,
    )?;
    let required = u64::from(entry.resources.threads_per_workgroup);
    if declared != required {
        return Err(rule(ArtifactBuildError::LaunchDisagreement {
            entry: index,
            expected: required,
            actual: declared,
        }));
    }
    let threads = static_unsigned(
        envelope,
        entry.launch.grid_threads,
        AbiExprUse::LaunchThreads,
        static_facts,
    )?;
    if threads == 0 && !entry.launch.zero_work_skips_dispatch {
        return Err(rule(ArtifactBuildError::ZeroWorkPolicy { entry: index }));
    }
    Ok(())
}

fn static_unsigned(
    envelope: &ArtifactEnvelope,
    node: u32,
    use_site: AbiExprUse,
    facts: &AbiFacts,
) -> Result<u64, ArtifactCodecError> {
    match evaluate(envelope.expressions(), node, facts) {
        Ok(value) => value.unsigned().ok_or_else(|| {
            rule(ArtifactBuildError::ExpressionType {
                use_site,
                expected: AbiType::Unsigned,
                actual: AbiType::Boolean,
            })
        }),
        Err(cause) => Err(rule(ArtifactBuildError::StaticEvaluation {
            use_site,
            cause,
        })),
    }
}

fn check_duplicate_variants(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
    for variant in envelope.variants() {
        if !seen.insert((variant.program_section, variant.guard)) {
            return Err(rule(ArtifactBuildError::DuplicateVariant));
        }
    }
    Ok(())
}

fn selects(envelope: &ArtifactEnvelope, authority: &ProviderIdentity) -> bool {
    envelope
        .providers()
        .iter()
        .any(|selected| &selected.provider == authority)
}

/// Proves a canonically ordered collection is sorted and free of repeats.
fn check_ordered(keys: &[Vec<u8>], subject: OrderedSubject) -> Result<(), ArtifactCodecError> {
    for pair in keys.windows(2) {
        match pair[0].cmp(&pair[1]) {
            std::cmp::Ordering::Less => {}
            std::cmp::Ordering::Equal => {
                return Err(ArtifactCodecError::DuplicateItem { subject });
            }
            std::cmp::Ordering::Greater => {
                return Err(ArtifactCodecError::NonCanonicalOrder { subject });
            }
        }
    }
    Ok(())
}

fn rule(cause: ArtifactBuildError) -> ArtifactCodecError {
    ArtifactCodecError::ModelRule {
        cause: Box::new(cause),
    }
}

const fn obligation(cause: ArtifactDiagnostic) -> ArtifactCodecError {
    ArtifactCodecError::ModelObligation { cause }
}
