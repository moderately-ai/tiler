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
//! Two of the builder's obligations tie the ABI to the *program* rather than to
//! the manifest: a binding's accessible byte range must equal the exact byte
//! window its stage access addresses, and an entry's bindings must correspond
//! one-to-one with its kernel's buffer parameters. Neither the byte windows nor
//! the kernel signature travel in this profile, so a decoder cannot recompute
//! them. They are not therefore unguarded: both are folded into the artifact's
//! canonical identity through the binding's expression content key and the
//! entry's stage key, and the identity is re-derived and compared below. A
//! forged envelope can restate them only by becoming a different artifact.
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
use super::super::model::deferred_key;
use super::error::{ArtifactCodecError, OrderedSubject};
use super::model::{
    ArtifactEnvelope, EntryRow, VariantRow, expression_keys, node_operands, position,
};

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
    for pair in envelope.sections().windows(2) {
        match pair[0].bytes.cmp(&pair[1].bytes) {
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
    let referenced: BTreeSet<u32> = envelope
        .variants()
        .iter()
        .map(|variant| variant.program_section)
        .collect();
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
                    .map(|binding| binding.accessible_bytes),
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
    check_ordered(
        &variant
            .deferred
            .iter()
            .map(|predicate| deferred_key(keys, predicate))
            .collect::<Vec<_>>(),
        OrderedSubject::DeferredPredicate,
    )?;
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
