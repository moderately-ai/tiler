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
//! identity through its one canonical arena and the binding's canonical-position
//! expression references, its encoded target, and the entry's stage key. The
//! identity is re-derived and compared below; a forged envelope can restate them
//! only by becoming a different artifact.
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

use super::super::error::{AbiExprUse, ArtifactBuildError, ArtifactDiagnostic};
use super::super::expr::{
    AbiFacts, AbiType, AvailabilityPhase, ExprNode, evaluate, node_is_interface_only, node_phase,
    node_type,
};
use super::super::facts::AbiFactBinder;
use super::super::model::{
    BindingTargetData, canonical_deferred_order, deferred_predicate_matches_requirement,
};
use super::super::requirement::canonical_requirement_order;
use super::error::{ArtifactCodecError, OrderedSubject};
use super::model::{ArtifactEnvelope, EntryRow, VariantRow, node_operands, position};
use super::payload::{decode_metadata, payload_identity};
use tiler_ir::kernel::KernelType;
use tiler_ir::program::abi::compare_expr_nodes;
use tiler_ir::program::{StorageEncoding, StorageScalar};
use tiler_ir::shape::{BindingSource, ShapeSymbol, SourcedExtent};

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
        return Err(obligation(
            ArtifactDiagnostic::MissingSelectedLoweringProvider,
        ));
    }
    if envelope.features() != envelope.derived_features() {
        return Err(ArtifactCodecError::DeclaredFeatureMismatch);
    }
    check_interface(envelope)?;
    check_sections(envelope)?;
    check_payload_identity(envelope)?;
    check_binding_targets(envelope)?;
    check_extent_operands(envelope)?;
    // Before the mapping check, which resolves a payload per delivery position
    // and would otherwise report a scrambled realization run as a missing
    // symbol.
    check_delivery_positions(envelope)?;
    // After the position check, whose count every scope run is locked to.
    check_plan_determinism(envelope)?;
    check_entry_mappings(envelope)?;
    check_extent_operand_static_axes(envelope)?;
    // After the interface's own structural obligations and before any expression
    // check, because an axis whose symbol the environment does not declare has
    // no binding a later evaluation could resolve.
    check_interface_symbol_coherence(envelope)?;
    // After the coherence check, which owns the undeclared-symbol refusal. An
    // axis naming a symbol the environment does not declare is an incoherent
    // *interface* — invalid whether or not an operand row names that axis — so
    // reporting it as an operand fault would name the narrower of the two
    // causes. What is left for the association to decide is the root.
    check_extent_operand_association(envelope)?;
    let facts = ExpressionFacts::derive(envelope.expressions());
    check_expression_closure(envelope)?;
    check_backend_entries(envelope)?;
    let static_facts = interface_facts(envelope);
    for variant in envelope.variants() {
        check_variant(envelope, variant, &facts, &static_facts)?;
    }
    check_duplicate_variants(envelope)?;
    // Last, and deliberately: the record decoded on its own terms before
    // `validate` ran, so what remains is its agreement with the artifact around
    // it — the same obligation `ArtifactProgramBuilder::build` discharges on the
    // envelope it projects. It reads the entry table, so it runs after that
    // table's own structural obligations have been proven. Running it first
    // reported a forged *extra* entry as an unbound one, which named a real
    // disagreement but not the one that made the envelope invalid.
    envelope.check_realization().map_err(obligation)
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
        let _ = binder.bind_declared_extents(&input.key, &input.extents);
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
    for entry in envelope
        .inputs()
        .iter()
        .map(|entry| (&entry.logical_type, &entry.components))
        .chain(
            envelope
                .outputs()
                .iter()
                .map(|entry| (&entry.logical_type, &entry.components)),
        )
    {
        let (logical_type, components) = entry;
        let mut roles = std::collections::BTreeSet::new();
        if logical_type.is_empty()
            || components.is_empty()
            || components.iter().any(|component| {
                !roles.insert(component.role)
                    || component.role.is_some() != component.resolved_type.is_some()
            })
        {
            return Err(ArtifactCodecError::MalformedInterfaceComponents);
        }
    }
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
        check_binding_access(binding)?;
        match &binding.target {
            BindingTargetData::ProgramInput(key) => {
                let Some(input) = envelope.inputs().iter().find(|input| input.key == *key) else {
                    return Err(ArtifactCodecError::UnknownBindingTargetKey {
                        key: key.as_str().to_owned(),
                        input: true,
                    });
                };
                check_target_component(&input.components, binding)?;
            }
            BindingTargetData::ProgramOutput(keys) => {
                for key in keys {
                    let Some(output) = envelope.outputs().iter().find(|output| output.key == *key)
                    else {
                        return Err(ArtifactCodecError::UnknownBindingTargetKey {
                            key: key.as_str().to_owned(),
                            input: false,
                        });
                    };
                    check_target_component(&output.components, binding)?;
                }
            }
            BindingTargetData::Internal => {}
        }
    }
    Ok(())
}

/// Proves every live-extent operand names a declared input axis as an unsigned quantity.
///
/// The live *value* is not in the envelope. What is decidable here is the
/// declaration: the key is one the interface names, the axis is in rank, the
/// type is the unsigned quantity Metal binds, and the list is in canonical
/// `(key, axis)` order without duplicates. Whether the named axis is one the
/// published interface leaves open at all is
/// [`check_extent_operand_static_axes`], which runs after the transport checks
/// so each structural refusal stays reachable.
fn check_extent_operands(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    for entry in envelope
        .variants()
        .iter()
        .flat_map(|variant| &variant.entries)
    {
        for pair in entry.input_extents.windows(2) {
            match (
                pair[0].key.as_str().cmp(pair[1].key.as_str()),
                pair[0].axis.get().cmp(&pair[1].axis.get()),
            ) {
                (std::cmp::Ordering::Less, _)
                | (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => {}
                (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => {
                    return Err(ArtifactCodecError::DuplicateItem {
                        subject: OrderedSubject::ExtentOperand,
                    });
                }
                _ => {
                    return Err(ArtifactCodecError::NonCanonicalOrder {
                        subject: OrderedSubject::ExtentOperand,
                    });
                }
            }
        }
        for operand in &entry.input_extents {
            let Some(input) = envelope
                .inputs()
                .iter()
                .find(|input| input.key == operand.key)
            else {
                return Err(ArtifactCodecError::UnknownExtentOperandKey {
                    key: operand.key.as_str().to_owned(),
                });
            };
            let rank = input.rank();
            if usize::try_from(operand.axis.get()).unwrap_or(usize::MAX) >= rank {
                return Err(ArtifactCodecError::ExtentOperandAxis {
                    key: operand.key.as_str().to_owned(),
                    axis: operand.axis.get(),
                    rank,
                });
            }
            if operand.value_type != AbiType::Unsigned {
                return Err(ArtifactCodecError::ExtentOperandType {
                    key: operand.key.as_str().to_owned(),
                    axis: operand.axis.get(),
                });
            }
        }
    }
    Ok(())
}

/// Refuses every live-extent operand row over an axis the published interface
/// fixes.
///
/// A fixed semantic axis must not acquire a caller-selected extent, and this is
/// the decode-side half of the construction-time `ExtentOperandStaticAxis`
/// association.
///
/// **Per-axis since `tiler.artifact-program.v21`.** The decoded interface
/// grammar used to state only literal extents, so *every* axis a row could name
/// was fixed and this refused every declared row by name. The grammar now states
/// each axis literal-or-symbol, so the question is asked of the axis the row
/// actually names: a literal one is refused and reports its own extent, and a
/// symbolic one is the case the row exists for and is passed on to
/// [`check_extent_operand_association`], which proves the root.
///
/// *(Corrected 2026-08-22. This read that a symbolic axis "passes on to the
/// association checks" from the day the grammar narrowed, when no association
/// check ran after this one at all: a decoded row over a symbolic axis was
/// admitted whatever its symbol was rooted at, including roots that make the
/// row one no builder could have written. The sentence describes what happens
/// now that `check_extent_operand_association` runs.)*
///
/// Deliberately after [`check_extent_operands`] and `check_entry_mappings`, so
/// a row that is *also* structurally broken — misordered, duplicated, out of
/// rank, mistyped, or misplaced on its transport — still reports the narrower
/// structural refusal those checks own.
fn check_extent_operand_static_axes(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    for entry in envelope
        .variants()
        .iter()
        .flat_map(|variant| &variant.entries)
    {
        for operand in &entry.input_extents {
            let input = envelope
                .inputs()
                .iter()
                .find(|input| input.key == operand.key)
                .expect("check_extent_operands proved every operand key is declared");
            let axis = usize::try_from(operand.axis.get())
                .expect("check_extent_operands proved the axis is inside the input's rank");
            if let Some(literal) = input.extents[axis].as_static() {
                return Err(ArtifactCodecError::ExtentOperandStaticAxis {
                    key: operand.key.as_str().to_owned(),
                    axis: operand.axis.get(),
                    extent: literal.get(),
                });
            }
        }
    }
    Ok(())
}

/// Proves the two spellings of one symbolic boundary cannot disagree.
///
/// The published interface names a symbol per axis; the retained environment
/// declares symbols and roots some of them at an exact `(input, axis)`. Those
/// are independent statements about one program, and the wire carries both, so
/// this refuses the only two ways they could contradict each other rather than
/// resolving one in favour of the other.
///
/// 1. **Declaration.** An interface axis naming a symbol the retained
///    environment does not declare has no binding a consumer could resolve it
///    through. Refused as `UndeclaredInterfaceSymbol`.
/// 2. **Root agreement.** Where the environment roots a symbol at `(key, axis)`
///    *and* that input's own interface entry names a symbol at that axis, the
///    two must be the same symbol. Otherwise one axis is spelled by two
///    different names and a consumer resolving the root would bind a quantity
///    the interface calls something else. Refused as `RootedAxisDisagreement`.
///
/// **A literal at a rooted axis is not a disagreement, and refusing it would be
/// a defect.** An environment may legitimately root a symbol at a statically
/// known dimension — the `S`/`C`/`T` carriers root at `input[0]` and `input[1]`
/// while the interface fixes both — and `tiler.artifact-program.v17` pins that
/// an unused retained environment is identity-bearing precisely so such a
/// program is representable. The symbol is simply determined there.
///
/// **A symbol on an axis the environment roots elsewhere is not one either.** In
/// the admitted same-shape population every input wears `n` while only one of
/// them roots it; that states an equality the retained constraints and the
/// runtime binding decide, not a contradiction in the spelling.
///
/// Outputs are checked for declaration only: an output axis has no root row of
/// its own, so agreement has nothing to compare against on that side.
///
/// A root naming an input the artifact does not declare, or an axis outside its
/// rank, is left alone here — `RetainedShapeEnvironment::evaluate` already
/// reports both as an invalid evaluation domain, and duplicating them would give
/// one fact two refusals.
fn check_interface_symbol_coherence(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let retained = &envelope.semantic().retained_shape;
    for (key, axis, symbol) in envelope
        .inputs()
        .iter()
        .flat_map(|input| declared_symbols(input.key.as_str(), &input.extents))
        .chain(
            envelope
                .outputs()
                .iter()
                .flat_map(|output| declared_symbols(output.key.as_str(), &output.extents)),
        )
    {
        if !retained
            .bindings()
            .iter()
            .any(|(declared, _)| declared == symbol)
        {
            return Err(ArtifactCodecError::UndeclaredInterfaceSymbol {
                key: key.to_owned(),
                axis,
                symbol: symbol.to_string(),
            });
        }
    }
    for (symbol, binding) in retained.bindings() {
        let BindingSource::InputDimension { input, axis } = binding.source() else {
            continue;
        };
        let Some(entry) = envelope.inputs().iter().find(|entry| entry.key == *input) else {
            continue;
        };
        let Some(declared) = usize::try_from(axis.get())
            .ok()
            .and_then(|axis| entry.extents.get(axis))
        else {
            continue;
        };
        let Some(spelled) = declared.symbol() else {
            continue;
        };
        if spelled != symbol {
            return Err(ArtifactCodecError::RootedAxisDisagreement {
                key: input.as_str().to_owned(),
                axis: axis.get(),
                rooted: symbol.to_string(),
                declared: spelled.to_string(),
            });
        }
    }
    Ok(())
}

/// Re-proves each live-extent operand row against the association its builder
/// proved: the axis the row names is symbolic, its symbol is declared by the
/// artifact's one retained environment, and that symbol is rooted at exactly
/// that `(key, axis)` input dimension.
///
/// This runs the model's own `builder::check_extent_operand_association` rather
/// than restating it. The gap it closes opened because one rule had two
/// implementations and only one of them moved: while the interface grammar
/// carried literal extents alone, every axis a row could name was fixed and
/// [`check_extent_operand_static_axes`] refused every declared row, which left
/// the symbolic arms unreachable and their absence here harmless. The per-axis
/// narrowing at `tiler.artifact-program.v21` made them reachable and nothing
/// carried them across. One authority cannot drift apart from itself that way,
/// and a fifth arm added to the association is proven on decoded bytes without
/// anyone remembering to copy it. The refusal carries the model's own typed
/// [`ArtifactBuildError`], as this module's other re-proofs do, so it reads the
/// same whether the artifact was refused at construction or at load.
///
/// Two of the association's four arms cannot fire from here, each because a
/// narrower check already owns its fact and reports a better cause:
///
/// - a **static** axis is refused first by [`check_extent_operand_static_axes`],
///   which names the one extent that axis is fixed at; and
/// - a symbol the retained environment does not declare is refused first by
///   [`check_interface_symbol_coherence`] as `UndeclaredInterfaceSymbol`. That
///   is the more fundamental of the two faults: such an interface has an axis
///   nothing can bind whether or not an operand row names it, so an operand
///   refusal would describe a consequence rather than the cause.
///
/// What remains, and what makes this check load-bearing, is the *root*. A
/// symbol rooted at a static extent or a target property is answered by that
/// authority and never by a per-invocation operand; a symbol rooted at a
/// different input dimension makes the row name an inferred occurrence rather
/// than the source-bearing axis, and a loader freezing that axis's extent would
/// bind an unrelated quantity that no retained constraint need prove equal.
fn check_extent_operand_association(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let bindings = envelope.semantic().retained_shape.bindings();
    for variant in envelope.variants() {
        // Enumerated per variant, which is the `entry` the model's own rules
        // are numbered by: `check_entry` reports `LaunchDisagreement` against
        // the same index, and a position flattened across variants would name a
        // different entry than the builder does.
        for (index, entry) in variant.entries.iter().enumerate() {
            for operand in &entry.input_extents {
                let input = envelope
                    .inputs()
                    .iter()
                    .find(|input| input.key == operand.key)
                    .expect("check_extent_operands proved every operand key is declared");
                super::super::builder::check_extent_operand_association(
                    index,
                    &operand.key,
                    operand.axis,
                    &input.extents,
                    bindings,
                )
                .map_err(rule)?;
            }
        }
    }
    Ok(())
}

/// Returns every symbolic axis of one declared boundary, with its position.
fn declared_symbols<'a>(
    key: &'a str,
    extents: &'a [SourcedExtent],
) -> impl Iterator<Item = (&'a str, u32, &'a ShapeSymbol)> {
    extents
        .iter()
        .enumerate()
        .filter_map(move |(axis, extent)| {
            extent
                .symbol()
                .map(|symbol| (key, u32::try_from(axis).unwrap_or(u32::MAX), symbol))
        })
}

fn check_target_component(
    components: &[super::super::model::InterfaceComponentData],
    binding: &super::super::model::BindingData,
) -> Result<(), ArtifactCodecError> {
    let component = components
        .iter()
        .find(|component| component.role == binding.component_role)
        .ok_or(ArtifactCodecError::UnknownBindingTargetComponent {
            role: binding
                .component_role
                .map(tiler_ir::semantic::EncodedComponentRole::get),
        })?;
    if component.storage_scalar != binding.storage_scalar
        || component.encoding != binding.encoding
        || component.access_type != binding.access_type
    {
        return Err(ArtifactCodecError::BindingComponentMismatch);
    }
    Ok(())
}

/// Proves each binding reads its carrier through the access type that carrier
/// stores.
///
/// The failure this exists to prevent is a width misread: a slot whose carrier
/// is two bytes wide and whose access type is four would address twice the bytes
/// the interface provides, and every framing, digest, and identity check would
/// still pass. So the pairing is stated by name for each carrier, with no
/// wildcard — a widened carrier vocabulary stops the build here rather than
/// falling into a neighbouring carrier's access type.
fn check_binding_access(
    binding: &super::super::model::BindingData,
) -> Result<(), ArtifactCodecError> {
    let compatible = match binding.encoding {
        StorageEncoding::Unpacked => {
            binding.access_type
                == match binding.storage_scalar {
                    StorageScalar::U8 => KernelType::U8,
                    StorageScalar::F32 => KernelType::F32,
                    StorageScalar::Bf16 => KernelType::Bf16,
                    StorageScalar::U32 => KernelType::U32,
                }
        }
        StorageEncoding::BitPacked(_) => {
            binding.storage_scalar == StorageScalar::U8 && binding.access_type == KernelType::U8
        }
    };
    if compatible {
        Ok(())
    } else {
        Err(ArtifactCodecError::BindingAccessTypeMismatch)
    }
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
        // Every delivery position, not the first: a consumer resolving position
        // `p` loads that position's object, so an object that maps none of the
        // entries it realizes must be refused wherever it sits.
        for payload in &entry.payloads {
            let Some(content) = envelope.payload_content()[position(*payload)] else {
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
                .ok_or(ArtifactCodecError::UnmappedBackendEntry { payload: *payload })?;
            if mapping.transports.len()
                != entry
                    .bindings
                    .len()
                    .saturating_add(entry.input_extents.len())
            {
                return Err(ArtifactCodecError::EntryTransportCardinality {
                    payload: *payload,
                    bindings: entry.bindings.len(),
                    extents: entry.input_extents.len(),
                    transports: mapping.transports.len(),
                });
            }
            let binding_count = u32::try_from(entry.bindings.len()).unwrap_or(u32::MAX);
            for (operand, slot) in mapping.transports[entry.bindings.len()..]
                .iter()
                .enumerate()
            {
                let expected =
                    binding_count.saturating_add(u32::try_from(operand).unwrap_or(u32::MAX));
                if *slot != expected {
                    return Err(ArtifactCodecError::ExtentOperandTransport {
                        payload: *payload,
                        operand,
                        declared: *slot,
                        expected,
                    });
                }
            }
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
///
/// A *realization* is an entry paired with a delivery position, so the walk is
/// over those rather than over entries. An entry naming one payload at two
/// positions repeats its `(payload, entry_key)` pair and is refused here, which
/// is what stops one object standing in for two consumer build targets.
fn check_backend_entries(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let mut claimed: BTreeSet<(u32, &[u8])> = BTreeSet::new();
    let mut referenced: BTreeSet<u32> = BTreeSet::new();
    for entry in envelope
        .variants()
        .iter()
        .flat_map(|variant| &variant.entries)
    {
        for payload in &entry.payloads {
            referenced.insert(*payload);
            if !claimed.insert((*payload, entry.entry_key.as_bytes())) {
                return Err(obligation(ArtifactDiagnostic::DuplicateBackendEntry));
            }
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

/// Proves every entry is realized at the artifact's own delivery positions.
///
/// Three obligations the builder discharged at construction and a decoder must
/// re-prove against bytes no builder wrote. Every entry names at least one
/// payload, every entry names the *same* number — a consumer resolves one
/// position for the whole artifact, so an entry short of it would have no object
/// for a route it must dispatch — and no payload is reached from two different
/// positions, which would make the artifact carry fewer objects than the
/// consumer targets it claims to have built for.
fn check_delivery_positions(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let positions = envelope.delivery_positions();
    let mut seen: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for (index, entry) in envelope
        .variants()
        .iter()
        .flat_map(|variant| &variant.entries)
        .enumerate()
    {
        if entry.payloads.is_empty() {
            return Err(rule(ArtifactBuildError::EmptyDelivery { entry: index }));
        }
        if entry.payloads.len() != positions {
            return Err(rule(ArtifactBuildError::DeliveryCardinality {
                entry: index,
                expected: positions,
                actual: entry.payloads.len(),
            }));
        }
        for (delivery, payload) in entry.payloads.iter().enumerate() {
            if *seen.entry(*payload).or_insert(delivery) != delivery {
                return Err(obligation(ArtifactDiagnostic::AmbiguousPayloadDelivery {
                    payload: *payload,
                }));
            }
        }
    }
    Ok(())
}

/// Re-proves the plan-determinism scope runs and every claimed cell's
/// byte-decidable coherence.
///
/// The neutral half of the builder's proof-bound `publish_plan` join, run
/// against bytes no builder wrote: each variant carries exactly one cell per
/// delivery position, and a `Plan` cell requires every entry's payload at that
/// position to carry a target-environment declaration, carry its object bytes,
/// and resolve to one shared declared-environment tuple. What stays
/// deliberately out of reach here is semantic provider validation — a neutral
/// decoder holds no provider schema, so it can frame an unknown provider but
/// never turn it into executable compatibility.
fn check_plan_determinism(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let positions = envelope.delivery_positions();
    for (rank, variant) in envelope.variants().iter().enumerate() {
        if variant.scope.len() != positions {
            return Err(obligation(
                ArtifactDiagnostic::PlanDeterminismScopeCardinality {
                    variant: rank,
                    cells: variant.scope.len(),
                    positions,
                },
            ));
        }
        for (delivery, cell) in variant.scope.iter().enumerate() {
            match cell {
                super::super::environment::PlanDeterminismScope::Unclaimed => continue,
                super::super::environment::PlanDeterminismScope::Plan => {}
            }
            let incoherent = |entry| {
                obligation(ArtifactDiagnostic::UnverifiedPlanDeterminismClaim {
                    variant: rank,
                    delivery,
                    entry,
                })
            };
            let mut first: Option<(usize, usize)> = None;
            for (entry, row) in variant.entries.iter().enumerate() {
                let payload = position(row.payloads[delivery]);
                let descriptor = &envelope.payloads()[payload];
                if descriptor.environment.is_none() || envelope.payload_content()[payload].is_none()
                {
                    return Err(incoherent(entry));
                }
                match first {
                    None => first = Some((entry, payload)),
                    Some((_, first_payload)) => {
                        let held = &envelope.payloads()[first_payload];
                        // The declared-environment tuple compared component-wise:
                        // deriving the canonical identity requires a validated
                        // declaration, which a neutral decoder cannot mint.
                        if descriptor.environment != held.environment
                            || descriptor.compatibility != held.compatibility
                            || descriptor.backend != held.backend
                            || descriptor.representation != held.representation
                        {
                            return Err(incoherent(entry));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn check_variant(
    envelope: &ArtifactEnvelope,
    variant: &VariantRow,
    facts: &ExpressionFacts,
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
        if predicate.requirement.query().available_at()
            != AvailabilityPhase::PreparedKernelPreflight
        {
            return Err(rule(ArtifactBuildError::UnsupportedDeferredQueryPhase {
                phase: predicate.requirement.query().available_at(),
            }));
        }
        if position(predicate.entry) >= variant.entries.len() {
            return Err(rule(ArtifactBuildError::DeferredQueryEntryOutOfRange {
                entry: predicate.entry,
                entries: variant.entries.len(),
            }));
        }
        if !deferred_predicate_matches_requirement(
            envelope.expressions(),
            predicate.predicate,
            &predicate.requirement,
        ) {
            return Err(rule(ArtifactBuildError::DeferredQueryPredicateMismatch));
        }
        facts.check_use(
            predicate.predicate,
            AbiExprUse::DeferredPredicate,
            AbiType::Boolean,
            predicate.requirement.query().available_at(),
            false,
        )?;
    }
    check_route_requirements(variant)?;
    check_ordered(
        &variant.entries,
        |entry| entry.stage.as_bytes(),
        OrderedSubject::Entry,
    )?;
    for (index, entry) in variant.entries.iter().enumerate() {
        check_entry(envelope, entry, index, facts, static_facts)?;
    }
    Ok(())
}

/// Re-proves one variant's route requirements are canonical and non-contradictory.
///
/// Two checks, because they refuse different things. The order check refuses a
/// well-formed encoding that is not *the* encoding of this artifact, which is
/// what keeps one artifact to one byte identity. The subject check refuses two
/// rows that constrain one subject: they state two answers to one question, and
/// nothing in the envelope can say which the producer meant, so admitting them
/// would let a reader satisfy the weaker row and route.
///
/// Both are re-proven here rather than inherited from construction, because the
/// envelope being validated may have been decoded from bytes no builder wrote.
fn check_route_requirements(variant: &VariantRow) -> Result<(), ArtifactCodecError> {
    if canonical_requirement_order(&variant.route_requirements)
        != (0..variant.route_requirements.len()).collect::<Vec<_>>()
    {
        return Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::RouteRequirement,
        });
    }
    // Canonical order puts equal subjects adjacent, because a row's canonical
    // bytes lead with its subject, so neighbours decide distinctness.
    for pair in variant.route_requirements.windows(2) {
        if pair[0].subject() == pair[1].subject() {
            return Err(rule(ArtifactBuildError::DuplicateRouteRequirementSubject {
                subject: Box::new(pair[0].subject()),
            }));
        }
    }
    Ok(())
}

fn check_entry(
    envelope: &ArtifactEnvelope,
    entry: &EntryRow,
    index: usize,
    facts: &ExpressionFacts,
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
    check_ordered_nodes(
        envelope.expressions(),
        &entry.launch.preconditions,
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

/// Proves a canonically ordered collection is sorted and free of repeats.
///
/// The order key is borrowed from each item rather than collected first.
/// Canonical order is decided by adjacent pairs alone, so materializing the key
/// table would copy every stage subject an envelope carries — bounded by the
/// manifest, and therefore by a quantity a producer chooses — to learn something
/// two borrowed slices already answer.
fn check_ordered<'a, T>(
    items: &'a [T],
    key: impl Fn(&'a T) -> &'a [u8],
    subject: OrderedSubject,
) -> Result<(), ArtifactCodecError> {
    for pair in items.windows(2) {
        match key(&pair[0]).cmp(key(&pair[1])) {
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

/// Proves a canonically ordered run of arena nodes is sorted and free of repeats.
///
/// [`compare_expr_nodes`] rather than a table of canonical content keys, and for
/// the reason that comparator exists: a key frames each operand's whole key
/// inside its node's, so a key table over the arena costs bytes quadratic in
/// arena depth — a quantity a producer, or a forger, chooses. A comparison walks
/// both subtrees and stops at the first difference, so it materializes nothing.
/// It is the same relation the identity encoder folds through
/// `canonical_precondition_order`, which is what makes the stored order and the
/// checked order one definition rather than two that happen to agree.
fn check_ordered_nodes(
    nodes: &[ExprNode],
    run: &[u32],
    subject: OrderedSubject,
) -> Result<(), ArtifactCodecError> {
    for pair in run.windows(2) {
        match compare_expr_nodes(nodes, pair[0], pair[1]) {
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

fn obligation(cause: ArtifactDiagnostic) -> ArtifactCodecError {
    ArtifactCodecError::ModelObligation { cause }
}
