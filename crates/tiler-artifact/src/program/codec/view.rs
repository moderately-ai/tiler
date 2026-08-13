//! The public read side of the codec: encode an artifact, decode bytes back.
//!
//! # Why this exists rather than a promoted encoder
//!
//! An out-of-crate assembler must be able to prove that what it packaged
//! survives a round trip; before this module nothing outside `tiler-artifact`
//! could encode an artifact, decode one, or observe an envelope digest, so a
//! real compilation could never meet the codec's own checks.
//!
//! The obvious promotion — making [`super::encode`], [`super::decode`], and
//! `ArtifactEnvelope` public — was rejected on review. The envelope is an
//! internal projection the codec is still changing, and publishing it would
//! commit the boundary to its field layout. This module exposes the
//! *capability* instead: bytes out, a validated view back, and accessors rather
//! than fields.
//!
//! # A decoded artifact is a dispatch record
//!
//! Tom decided on `carry-reconstructable-kernel-programs-in-the-neutral-envelope`
//! that a decoded envelope is a **dispatch record, not a reconstruction**, and
//! this module is that decision implemented. A consumer holding only encoded
//! bytes can name the artifact's interface, walk each packaged variant's
//! entries, read every binding's transport category and what it addresses,
//! evaluate the accessible ranges, guards, preconditions and launch geometry,
//! and resolve each entry to the backend symbol and transport slots the carried
//! payload maps it to. For a multi-stage variant it can also walk the proven
//! execution order and its typed dependency edges. That is what a dispatch
//! needs and it is now reachable without the producer's code.
//!
//! It is still **not** a [`VerifiedArtifactProgram`]. A variant's program
//! reaches the envelope as its canonical identity bytes alone (`super::model`'s
//! projection stores `canonical_identity()`), so nothing decoded can rebuild a
//! `VerifiedKernelProgram`. The exclusion is structural rather than an omission:
//! `KernelProgramBuilder::new` requires a `SemanticProgram`, which requires a
//! frozen registry of `Arc<dyn OperationInferencer>` — behaviour, not data, and
//! no serialization carries it.
//!
//! # Which facts a decoder re-derives, and which it takes on trust
//!
//! The distinction decides what a rejection here is worth, so it is stated
//! rather than left for a reader to work out.
//!
//! Framing, integrity, canonical form, schema and feature compatibility, table
//! closure, expression typing and availability phase, and the artifact's own
//! identity are all **re-derived from the decoded content**. Holding a
//! [`DecodedArtifact`] is the evidence.
//!
//! A binding's element type, address space, access mode, alignment, accessible
//! byte range and — the one this module adds — **what it addresses** are facts
//! about the packaged kernel program, and the program is not carried. The
//! producer did not assert them either: [`super::super::ArtifactProgramBuilder`]
//! *derived* each from the program's own stage access at build time and refuses
//! a variant whose plan contradicts one. But that proof happened on the writing
//! side. What binds it to these bytes is artifact identity, which folds every
//! one of those fields, so a forged envelope that restates a binding target
//! becomes a *different artifact* rather than a lie about this one — and a
//! consumer that compares the decoded identity against the one it expected has
//! rejected the forgery. A consumer that compares nothing has not.
//!
//! That is a weaker guarantee than re-derivation and it is the accepted cost of
//! the dispatch record. It is the same posture every other envelope row already
//! has; it is recorded here because the binding target is the first row whose
//! misreading silently binds the wrong buffer instead of failing.
//!
//! # What this view still does not publish
//!
//! The *structure* of an ABI expression. [`DecodedExpr`] answers what an
//! expression evaluates to and not how it is built, because a dispatch needs
//! the value and an explain surface needs the tree, and no explain surface over
//! a decoded artifact exists yet. Adding one is additive; guessing its shape now
//! would commit the boundary to it.
//!
use super::super::expr::{AbiEvaluationError, AbiFacts, AbiType, AbiValue, evaluate, node_type};
use super::super::keys::{BackendEntryKey, FeasibilityRuleSetRef, TargetProfileRef};
use super::super::model::{
    BackendPayloadDescriptor, BindingData, BindingKind, BindingTarget, BindingTargetData,
    CanonicalArtifactProgramIdentity, DeferredPredicateData, InterfaceComponentData,
    InterfaceEntryData, RoutingPolicy, StageDependencyData, StageDependencyReason,
    VerifiedArtifactProgram,
};
use super::super::realization::DeliveredRealizationRecord;
use super::super::requirement::RouteRequirement;
use super::error::ArtifactCodecError;
use super::model::{ArtifactEnvelope, EntryRow, NumericalFacts, SectionKind, VariantRow, position};
use super::payload::{PayloadEntryMapping, PayloadMetadata, decode_metadata};
use std::error::Error;
use std::fmt;

use tiler_ir::kernel::{AddressSpace, BufferAccess, KernelType};
use tiler_ir::program::abi::PreparedEntryTargetRequirement;
use tiler_ir::schedule::{
    ExceptionalValueAssumption, NumericalPermission, ResourceRequirements, SubnormalMode,
};
use tiler_ir::semantic::{InputKey, OutputKey};
use tiler_ir::shape::Shape;

use super::decode::decode;
use super::encode::encode;

impl VerifiedArtifactProgram {
    /// Encodes this artifact into its canonical envelope bytes.
    ///
    /// The bytes are a function of the artifact's identity rather than of the
    /// order a producer declared things in, so two producers that built the
    /// same artifact emit the same bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCodecFailure`] when the canonical encoding exceeds a
    /// governed envelope bound.
    ///
    /// # Panics
    ///
    /// Panics if the artifact's data no longer projects into an envelope.
    /// [`ArtifactProgramBuilder::build`] already performed that projection to
    /// derive this artifact's identity, and the data is immutable afterward, so
    /// a failure here is a defect in this crate rather than a caller error —
    /// which is why it is not a returned variant a caller would have to handle.
    ///
    /// [`ArtifactProgramBuilder::build`]: super::super::ArtifactProgramBuilder::build
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactCodecFailure> {
        let envelope = ArtifactEnvelope::project(&self.data)
            .expect("a verified artifact projected into an envelope when its identity was derived");
        encode(&envelope).map_err(ArtifactCodecFailure::from)
    }
}

/// Decodes and fully validates one encoded artifact envelope.
///
/// Validation is not optional and not separable: framing, manifest and section
/// digests, schema, canonical order, arena closure, and identity re-derivation
/// all run before this returns. A rejection never yields a partially validated
/// view, so holding a [`DecodedArtifact`] is itself the evidence that the bytes
/// passed every check.
///
/// # Errors
///
/// Returns the typed [`ArtifactCodecFailure`] naming the first boundary that
/// rejected.
pub fn decode_artifact(bytes: &[u8]) -> Result<DecodedArtifact, ArtifactCodecFailure> {
    let envelope = decode(bytes).map_err(ArtifactCodecFailure::from)?;
    // Parsed once here rather than on every lookup. `super::validate` already
    // proved each carried subject parses and that its bytes are the payload's
    // declared identity, so this cannot fail for an envelope that decoded; the
    // rejection is propagated rather than asserted because a decoder that
    // panicked on hostile bytes would be the wrong failure mode even for an
    // unreachable branch.
    let mut payload_metadata = Vec::with_capacity(envelope.payloads().len());
    for content in envelope.payload_content() {
        payload_metadata.push(match content {
            Some(sections) => Some(
                decode_metadata(&envelope.sections()[position(sections.metadata)].bytes)
                    .map_err(ArtifactCodecFailure::from)?,
            ),
            None => None,
        });
    }
    Ok(DecodedArtifact {
        envelope,
        payload_metadata,
    })
}

/// A validated read view over one decoded artifact envelope.
///
/// Accessors rather than fields, so this commits the public boundary to what an
/// artifact carries and not to how the codec lays it out.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedArtifact {
    envelope: ArtifactEnvelope,
    /// Parsed compilation subject of each payload, aligned with `payloads()`.
    ///
    /// A function of the envelope rather than a second authority over it: it is
    /// derived from the framed metadata section at decode and never written
    /// back, so [`DecodedArtifact::re_encode`] still encodes the envelope alone.
    /// The emitted object is deliberately *not* copied here — it is reached
    /// through its section, so carrying a payload does not double its cost.
    payload_metadata: Vec<Option<PayloadMetadata>>,
}

impl DecodedArtifact {
    /// Returns the identity re-derived from the decoded content.
    ///
    /// Re-derived, never read from the bytes: [`decode_artifact`] already
    /// proved this equals the identity the encoded manifest carried, so a
    /// forged manifest cannot present a chosen identity.
    ///
    /// # Panics
    ///
    /// Panics if the identity no longer derives. [`decode_artifact`] already
    /// derived it to compare against the encoded manifest, and this view is
    /// immutable, so a failure here is a defect in this crate.
    #[must_use]
    pub fn identity(&self) -> CanonicalArtifactProgramIdentity {
        self.envelope
            .canonical_identity()
            .expect("a decoded envelope derived its identity during validation")
    }

    /// Returns the governed features a reader must implement to use this
    /// artifact.
    #[must_use]
    pub fn features(&self) -> &[String] {
        self.envelope.features()
    }

    /// Returns the routing policy the artifact declares.
    #[must_use]
    pub const fn routing(&self) -> RoutingPolicy {
        self.envelope.routing
    }

    /// Returns the numerical realization this artifact delivered.
    ///
    /// A **total** reader: decoded bytes are given no optional path of their
    /// own. The record decoded on its own terms, its profile was proven to be
    /// the artifact's, every packaged entry was proven to reference an existing
    /// policy subject, and the eight overlapping resolutions were proven to
    /// equal every bound entry's own realization statement — all before this
    /// view existed. There is no `UnrecordedRealization` state left for a caller
    /// to rediscover.
    ///
    /// Entry bindings name the flat canonical packaged-entry ordinal: variants
    /// in routing priority order, and within each variant its entries in
    /// [`DecodedVariant::entries`] order.
    #[must_use]
    pub const fn delivered_realization(&self) -> &DeliveredRealizationRecord {
        self.envelope.realization()
    }

    /// Returns the carried backend payload descriptors in canonical order.
    #[must_use]
    pub fn payloads(&self) -> &[BackendPayloadDescriptor] {
        &self.envelope.payloads
    }

    /// Returns one view per framed section, in canonical order.
    #[must_use]
    pub fn sections(&self) -> impl ExactSizeIterator<Item = SectionView<'_>> {
        self.envelope.sections.iter().map(|section| SectionView {
            kind: section.kind,
            bytes: &section.bytes,
        })
    }

    /// Returns the number of packaged plan variants, in routing priority order.
    #[must_use]
    pub fn variant_count(&self) -> usize {
        self.envelope.variants.len()
    }

    /// Returns how many delivery positions this artifact carries a payload for.
    ///
    /// One for the ordinary single-target artifact. A consumer resolves exactly
    /// one of these from its own build target and passes it to
    /// [`DecodedEntry::payload`]; a decode proves every entry is realized at
    /// every one of them.
    #[must_use]
    pub fn delivery_positions(&self) -> usize {
        self.envelope.delivery_positions()
    }

    /// Returns the named program inputs in semantic interface order.
    ///
    /// Order is meaning here and is retained rather than canonicalized. These
    /// are also the names an [`AbiRoot::InputExtent`] fact binds, so a consumer
    /// evaluating a launch formula reads its free variables from this list.
    ///
    /// [`AbiRoot::InputExtent`]: super::super::AbiRoot::InputExtent
    #[must_use]
    pub fn inputs(&self) -> impl ExactSizeIterator<Item = DecodedInput<'_>> {
        self.envelope.inputs.iter().map(DecodedInput)
    }

    /// Returns the named program outputs in semantic interface order.
    #[must_use]
    pub fn outputs(&self) -> impl ExactSizeIterator<Item = DecodedOutput<'_>> {
        self.envelope.outputs.iter().map(DecodedOutput)
    }

    /// Returns the packaged plan variants in routing priority order.
    ///
    /// Declaration order is meaning under
    /// [`RoutingPolicy::StablePriority`](super::super::RoutingPolicy::StablePriority):
    /// a consumer evaluates each variant's applicability guard in this order and
    /// takes the first that holds.
    #[must_use]
    pub fn variants(&self) -> impl ExactSizeIterator<Item = DecodedVariant<'_>> {
        (0..self.envelope.variants.len()).map(move |variant| DecodedVariant {
            artifact: self,
            variant,
        })
    }

    /// Returns the compilation subject of one carried payload, by descriptor position.
    ///
    /// `None` for a position past the descriptor table and for the
    /// descriptor-only payload the model has always admitted: the artifact names
    /// a backend object it does not contain, so there is no subject to report.
    /// The two are deliberately one answer — neither yields a payload this
    /// envelope can execute from.
    #[must_use]
    pub fn payload_metadata(&self, payload: usize) -> Option<&PayloadMetadata> {
        self.payload_metadata.get(payload)?.as_ref()
    }

    /// Returns the emitted object bytes of one carried payload, by descriptor position.
    ///
    /// The exact bytes the producer packaged: the decoder strips the framing and
    /// the section body *is* the object. `None` for a position past the
    /// descriptor table and for a descriptor-only payload.
    ///
    /// This is the descriptor-to-object association itself, which the envelope
    /// records per descriptor and which no reader could otherwise recover — the
    /// section table is content-addressed and deduplicates equal objects, so
    /// counting object sections cannot attribute them.
    #[must_use]
    pub fn payload_object(&self, payload: usize) -> Option<&[u8]> {
        let sections = (*self.envelope.payload_content.get(payload)?)?;
        Some(&self.envelope.sections[position(sections.code)].bytes)
    }

    /// Re-encodes this decoded artifact.
    ///
    /// A decode followed by this must reproduce the original bytes exactly.
    /// That is the round-trip property worth asserting: it proves the decoder
    /// read every field the encoder wrote, because a field silently dropped on
    /// the way in cannot be written back out.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCodecFailure`] when the canonical encoding exceeds a
    /// governed envelope bound.
    pub fn re_encode(&self) -> Result<Vec<u8>, ArtifactCodecFailure> {
        encode(&self.envelope).map_err(ArtifactCodecFailure::from)
    }
}

/// One named program input of a decoded artifact's interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodedInput<'a>(&'a InterfaceEntryData<InputKey>);

impl<'a> DecodedInput<'a> {
    /// Returns the stable interface key a consumer binds this input by.
    #[must_use]
    pub const fn key(self) -> &'a InputKey {
        &self.0.key
    }

    /// Returns the logical tensor shape the input must be bound with.
    #[must_use]
    pub const fn shape(self) -> &'a Shape {
        &self.0.shape
    }

    /// Returns the logical resolved-type identity encoding.
    #[must_use]
    pub fn resolved_type_encoding(self) -> &'a [u8] {
        &self.0.logical_type
    }

    /// Returns physical components in semantic order.
    #[must_use]
    pub fn components(self) -> impl ExactSizeIterator<Item = DecodedComponent<'a>> {
        self.0.components.iter().map(DecodedComponent)
    }
}

/// One named program output of a decoded artifact's interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodedOutput<'a>(&'a InterfaceEntryData<OutputKey>);

impl<'a> DecodedOutput<'a> {
    /// Returns the stable interface key this output is published under.
    #[must_use]
    pub const fn key(self) -> &'a OutputKey {
        &self.0.key
    }

    /// Returns the logical tensor shape the output is published with.
    #[must_use]
    pub const fn shape(self) -> &'a Shape {
        &self.0.shape
    }

    /// Returns the logical resolved-type identity encoding.
    #[must_use]
    pub fn resolved_type_encoding(self) -> &'a [u8] {
        &self.0.logical_type
    }

    /// Returns physical components in semantic order.
    #[must_use]
    pub fn components(self) -> impl ExactSizeIterator<Item = DecodedComponent<'a>> {
        self.0.components.iter().map(DecodedComponent)
    }
}

/// One decoded physical component of a logical interface value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodedComponent<'a>(&'a InterfaceComponentData);

impl<'a> DecodedComponent<'a> {
    /// Returns the stable semantic role, absent for a dense singleton.
    #[must_use]
    pub fn role(self) -> Option<tiler_ir::semantic::EncodedComponentRole> {
        self.0.role
    }

    /// Returns the physical component shape.
    #[must_use]
    pub fn shape(self) -> &'a Shape {
        &self.0.shape
    }

    /// Returns the semantic component type encoding, absent for a dense singleton.
    #[must_use]
    pub fn resolved_type_encoding(self) -> Option<&'a [u8]> {
        self.0.resolved_type.as_deref()
    }

    /// Returns the scalar carrier stored in physical memory.
    #[must_use]
    pub fn storage_scalar(self) -> tiler_ir::program::StorageScalar {
        self.0.storage_scalar
    }

    /// Returns the type through which kernels access the stored component.
    #[must_use]
    pub fn access_type(self) -> KernelType {
        self.0.access_type
    }

    /// Returns the complete physical storage encoding.
    #[must_use]
    pub fn storage_encoding(self) -> tiler_ir::program::StorageEncoding {
        self.0.encoding
    }
}

/// One ordering obligation between two entries of a decoded variant.
///
/// The envelope carries the obligation's *kind* and not its subject: the shared
/// IR names the value read or the allocation reused, and neither has a durable
/// name at the artifact layer. A consumer learns that this edge is a
/// read-after-write rather than a storage reuse, which is what decides how a
/// violation breaks; what each entry addresses it reads from the bindings.
#[derive(Clone, Copy, Debug)]
pub struct DecodedStageDependency<'a> {
    artifact: &'a DecodedArtifact,
    variant: usize,
    edge: usize,
}

impl<'a> DecodedStageDependency<'a> {
    /// Returns the entry that must be dispatched first.
    #[must_use]
    pub fn predecessor(self) -> DecodedEntry<'a> {
        DecodedEntry {
            artifact: self.artifact,
            variant: self.variant,
            entry: position(self.data().predecessor),
        }
    }

    /// Returns the entry that must be dispatched after it.
    #[must_use]
    pub fn successor(self) -> DecodedEntry<'a> {
        DecodedEntry {
            artifact: self.artifact,
            variant: self.variant,
            entry: position(self.data().successor),
        }
    }

    /// Returns why the two are ordered.
    #[must_use]
    pub fn reason(self) -> StageDependencyReason {
        self.data().reason
    }

    fn data(self) -> &'a StageDependencyData {
        &self.artifact.envelope.variants[self.variant].dependencies[self.edge]
    }
}

/// One packaged plan variant of a decoded artifact.
#[derive(Clone, Copy, Debug)]
pub struct DecodedVariant<'a> {
    artifact: &'a DecodedArtifact,
    variant: usize,
}

impl<'a> DecodedVariant<'a> {
    /// Returns the zero-based routing rank; lower is tried first.
    #[must_use]
    pub const fn routing_rank(self) -> usize {
        self.variant
    }

    /// Returns the canonical identity of the kernel program this variant executes.
    ///
    /// The identity alone. The program is not carried and cannot be rebuilt from
    /// an envelope, so this is what proves *which* program the variant packages
    /// — enough to bind a program a consumer already holds, and deliberately not
    /// enough to pretend to have reconstructed one.
    #[must_use]
    pub fn kernel_program_identity(self) -> &'a [u8] {
        &self.envelope().sections[position(self.data().program_section)].bytes
    }

    /// Returns the guard that must hold before this variant may be routed to.
    #[must_use]
    pub fn applicability_guard(self) -> DecodedExpr<'a> {
        self.expression(self.data().guard)
    }

    /// Returns the declared target profile this variant was assessed against.
    #[must_use]
    pub fn target_profile(self) -> &'a TargetProfileRef {
        &self.data().profile
    }

    /// Returns the feasibility rule set this variant was assessed under.
    #[must_use]
    pub fn feasibility_rules(self) -> &'a FeasibilityRuleSetRef {
        &self.data().feasibility_rules
    }

    /// Returns the additional requirements this route places on a live device.
    ///
    /// In canonical content order, distinct by subject, both re-proven at
    /// decode. Empty is a state rather than an absence: a route consuming no
    /// additional requirement declares none, and this layer cannot tell that
    /// apart from a producer that omitted one — only a producer-owned exhaustive
    /// declaration of what the payload uses can.
    ///
    /// A consumer must decide **every** row before committing a route. The
    /// neutral quantitative rows it can compare itself; a backend-scoped row is
    /// decided by the adapter its owner names, and one no adapter owns is a
    /// refusal rather than a row to skip.
    #[must_use]
    pub fn route_requirements(self) -> &'a [RouteRequirement] {
        &self.data().route_requirements
    }

    /// Returns the feasibility predicates deferred to a runtime query.
    #[must_use]
    pub fn deferred_predicates(
        self,
    ) -> impl ExactSizeIterator<Item = DecodedDeferredPredicate<'a>> {
        let artifact = self.artifact;
        let variant = self.variant;
        self.data()
            .deferred
            .iter()
            .map(move |predicate| DecodedDeferredPredicate {
                artifact,
                variant,
                predicate,
            })
    }

    /// Returns the executable entries, one per stage of the variant's program.
    ///
    /// The order is canonical stage-key order rather than the producer's
    /// declaration order, and it is **not** execution order. A single-entry
    /// variant makes the distinction moot; for a multi-stage one, dispatch in
    /// [`Self::execution_order`] and not in this iterator's order.
    #[must_use]
    pub fn entries(self) -> impl ExactSizeIterator<Item = DecodedEntry<'a>> {
        let artifact = self.artifact;
        let variant = self.variant;
        (0..self.data().entries.len()).map(move |entry| DecodedEntry {
            artifact,
            variant,
            entry,
        })
    }

    /// Returns this variant's entries in the order they must be dispatched.
    ///
    /// A permutation of [`Self::entries`], proven so at decode. This is the
    /// order to follow: the entry table itself is in canonical stage-key order,
    /// which is identity's order and carries no execution meaning.
    ///
    /// Derived by the producer from the packaged program's own topological
    /// order, and checked here against [`Self::stage_dependencies`], so an order
    /// that contradicts the program's dependency graph does not decode.
    #[must_use]
    pub fn execution_order(self) -> impl ExactSizeIterator<Item = DecodedEntry<'a>> {
        let artifact = self.artifact;
        let variant = self.variant;
        self.data()
            .execution_order
            .iter()
            .map(move |entry| DecodedEntry {
                artifact,
                variant,
                entry: position(*entry),
            })
    }

    /// Returns the ordering obligations the execution order discharges.
    ///
    /// Each names the two entries it orders and *why* they are ordered — a read
    /// of what the predecessor wrote, or reuse of storage it released. A
    /// consumer that reorders stages needs the reason, not only the order: the
    /// two kinds break differently when violated.
    #[must_use]
    pub fn stage_dependencies(self) -> impl ExactSizeIterator<Item = DecodedStageDependency<'a>> {
        let artifact = self.artifact;
        let variant = self.variant;
        (0..self.data().dependencies.len()).map(move |edge| DecodedStageDependency {
            artifact,
            variant,
            edge,
        })
    }

    fn envelope(self) -> &'a ArtifactEnvelope {
        &self.artifact.envelope
    }

    fn data(self) -> &'a VariantRow {
        &self.artifact.envelope.variants[self.variant]
    }

    fn expression(self, node: u32) -> DecodedExpr<'a> {
        DecodedExpr {
            artifact: self.artifact,
            node,
        }
    }
}

/// One feasibility predicate a variant defers to a runtime query.
#[derive(Clone, Copy, Debug)]
pub struct DecodedDeferredPredicate<'a> {
    artifact: &'a DecodedArtifact,
    variant: usize,
    predicate: &'a DeferredPredicateData,
}

impl<'a> DecodedDeferredPredicate<'a> {
    /// Returns the predicate that must hold before routing commits.
    #[must_use]
    pub const fn predicate(self) -> DecodedExpr<'a> {
        DecodedExpr {
            artifact: self.artifact,
            node: self.predicate.predicate,
        }
    }

    /// Returns the complete target-property requirement.
    #[must_use]
    pub const fn requirement(self) -> &'a PreparedEntryTargetRequirement {
        &self.predicate.requirement
    }

    /// Returns the exact prepared entry whose property must be observed.
    #[must_use]
    pub fn entry(self) -> DecodedEntry<'a> {
        DecodedEntry {
            artifact: self.artifact,
            variant: self.variant,
            entry: position(self.predicate.entry),
        }
    }
}

/// One executable entry of a decoded artifact's plan variant.
#[derive(Clone, Copy, Debug)]
pub struct DecodedEntry<'a> {
    artifact: &'a DecodedArtifact,
    variant: usize,
    entry: usize,
}

impl<'a> DecodedEntry<'a> {
    /// Returns the canonical content key of the program stage this entry dispatches.
    ///
    /// Opaque to this layer: it is the shared IR's own stage subject, compared
    /// and encoded rather than interpreted here.
    #[must_use]
    pub fn stage_key(self) -> &'a [u8] {
        self.data().stage.as_bytes()
    }

    /// Returns the exact resource requirements the bound kernel proved.
    #[must_use]
    pub fn resources(self) -> ResourceRequirements {
        self.data().resources
    }

    /// Returns the numerical realization the bound kernel preserves.
    #[must_use]
    pub fn numerical(self) -> DecodedNumerical<'a> {
        DecodedNumerical(&self.data().numerical)
    }

    /// Returns the ABI bindings in kernel buffer-parameter order.
    #[must_use]
    pub fn bindings(self) -> impl ExactSizeIterator<Item = DecodedBinding<'a>> {
        let artifact = self.artifact;
        let variant = self.variant;
        let entry = self.entry;
        (0..self.data().bindings.len()).map(move |binding| DecodedBinding {
            artifact,
            variant,
            entry,
            binding,
        })
    }

    /// Returns the total launch thread count expression.
    #[must_use]
    pub fn launch_threads(self) -> DecodedExpr<'a> {
        self.expression(self.data().launch.grid_threads)
    }

    /// Returns the per-workgroup thread count expression.
    #[must_use]
    pub fn threads_per_workgroup(self) -> DecodedExpr<'a> {
        self.expression(self.data().launch.threads_per_workgroup)
    }

    /// Returns whether a zero-work launch skips the dispatch entirely.
    #[must_use]
    pub fn zero_work_skips_dispatch(self) -> bool {
        self.data().launch.zero_work_skips_dispatch
    }

    /// Returns the launch-instance preconditions of this entry.
    #[must_use]
    pub fn launch_preconditions(self) -> impl ExactSizeIterator<Item = DecodedExpr<'a>> {
        let artifact = self.artifact;
        self.data()
            .launch
            .preconditions
            .iter()
            .map(move |node| DecodedExpr {
                artifact,
                node: *node,
            })
    }

    /// Returns the payload descriptor position realizing this entry at one
    /// delivery position.
    ///
    /// A position into [`DecodedArtifact::payloads`], and therefore also the
    /// argument [`DecodedArtifact::payload_object`] takes. `None` for a delivery
    /// position this artifact does not declare.
    ///
    /// A delivery position is the ordered slot a consumer's *build target*
    /// resolves to, so there is no default: an artifact carrying several objects
    /// has no "the" payload, and taking the first would hand a consumer the
    /// object built for somebody else's target. See
    /// [`DecodedArtifact::delivery_positions`].
    #[must_use]
    pub fn payload(self, delivery: usize) -> Option<usize> {
        self.data().payloads.get(delivery).copied().map(position)
    }

    /// Returns how many delivery positions realize this entry.
    ///
    /// The artifact's own count: a decode proves every entry declares the same
    /// non-zero number, so this and [`DecodedArtifact::delivery_positions`]
    /// always agree.
    #[must_use]
    pub fn delivery_positions(self) -> usize {
        self.data().payloads.len()
    }

    /// Returns the opaque backend entry key, the same in every realizing payload.
    #[must_use]
    pub fn backend_entry_key(self) -> &'a BackendEntryKey {
        &self.data().entry_key
    }

    /// Returns the backend's own entry-point symbol at one delivery position.
    ///
    /// `None` for a delivery position this artifact does not declare, and
    /// exactly when the realizing payload there is descriptor-only, since a
    /// payload this envelope does not carry has no mapping to report. When the
    /// payload *is* carried the answer is always `Some`: a decode proves the
    /// mapping covers every entry key the artifact dispatches at every position,
    /// so a carried payload with a missing symbol is refused rather than
    /// reported here.
    #[must_use]
    pub fn backend_symbol(self, delivery: usize) -> Option<&'a str> {
        self.mapping(delivery)
            .map(|mapping| mapping.symbol.as_str())
    }

    /// Returns the backend transport slot each ABI binding occupies, in slot order.
    ///
    /// `transports[i]` is where binding slot `i` goes, so this is the last step
    /// between an artifact's neutral ABI and a real encoder. `None` under the
    /// same conditions as [`Self::backend_symbol`], and never a short list: a
    /// decode proves the count equals this entry's binding count.
    #[must_use]
    pub fn transport_slots(self, delivery: usize) -> Option<&'a [u32]> {
        self.mapping(delivery)
            .map(|mapping| mapping.transports.as_slice())
    }

    /// Resolves this entry's mapping inside one delivery position's payload subject.
    fn mapping(self, delivery: usize) -> Option<&'a PayloadEntryMapping> {
        self.artifact
            .payload_metadata(self.payload(delivery)?)?
            .entries
            .iter()
            .find(|mapping| mapping.entry_key == self.data().entry_key)
    }

    fn data(self) -> &'a EntryRow {
        &self.artifact.envelope.variants[self.variant].entries[self.entry]
    }

    const fn expression(self, node: u32) -> DecodedExpr<'a> {
        DecodedExpr {
            artifact: self.artifact,
            node,
        }
    }
}

/// The declared numerical realization of one decoded entry's bound kernel.
///
/// A view rather than [`NumericalRealization`] because that record spells its
/// profile key `&'static str`, so it can name a compile-time constant of the
/// reading build and cannot represent a key read from bytes;
/// `own-the-numerical-realization-profile-key` records the durable fix.
///
/// [`NumericalRealization`]: tiler_ir::schedule::NumericalRealization
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodedNumerical<'a>(&'a NumericalFacts);

impl<'a> DecodedNumerical<'a> {
    /// Returns the governed numerical profile key the kernel was realized under.
    #[must_use]
    pub fn profile_key(self) -> &'a str {
        &self.0.profile_key
    }

    /// Returns the bit pattern arithmetic must deliver for a canonical NaN.
    #[must_use]
    pub const fn canonical_arithmetic_nan_bits(self) -> u32 {
        self.0.canonical_arithmetic_nan_bits
    }

    /// Returns how subnormal operands are treated.
    #[must_use]
    pub const fn input_subnormals(self) -> SubnormalMode {
        self.0.input_subnormals
    }

    /// Returns how subnormal results are treated.
    #[must_use]
    pub const fn result_subnormals(self) -> SubnormalMode {
        self.0.result_subnormals
    }

    /// Returns whether contraction into a fused operation is permitted.
    #[must_use]
    pub const fn contraction(self) -> NumericalPermission {
        self.0.contraction
    }

    /// Returns whether reassociation is permitted.
    #[must_use]
    pub const fn reassociation(self) -> NumericalPermission {
        self.0.reassociation
    }

    /// Returns whether reduction contributors may be permuted.
    #[must_use]
    pub const fn permutation(self) -> NumericalPermission {
        self.0.permutation
    }

    /// Returns whether transformations may eliminate signed zero distinctions.
    #[must_use]
    pub const fn signed_zero(self) -> NumericalPermission {
        self.0.signed_zero
    }

    /// Returns whether NaNs may be assumed absent, and on what evidence.
    #[must_use]
    pub const fn nan_assumptions(self) -> ExceptionalValueAssumption {
        self.0.nan_assumptions
    }

    /// Returns whether infinities may be assumed absent, and on what evidence.
    #[must_use]
    pub const fn infinity_assumptions(self) -> ExceptionalValueAssumption {
        self.0.infinity_assumptions
    }
}

/// One ABI binding of a decoded artifact's executable entry.
#[derive(Clone, Copy, Debug)]
pub struct DecodedBinding<'a> {
    artifact: &'a DecodedArtifact,
    variant: usize,
    entry: usize,
    binding: usize,
}

impl<'a> DecodedBinding<'a> {
    /// Returns the zero-based ABI slot; the order is the kernel signature's.
    ///
    /// The same index into the entry's [`DecodedEntry::transport_slots`], which
    /// is what carries this slot to a backend's own binding index.
    #[must_use]
    pub const fn slot(self) -> usize {
        self.binding
    }

    /// Returns what this slot addresses.
    ///
    /// The fact that makes a decoded artifact dispatchable rather than merely
    /// describable, and the one this module's documentation singles out as
    /// producer-derived rather than decoder-re-derived.
    #[must_use]
    pub fn target(self) -> BindingTarget<'a> {
        match &self.data().target {
            BindingTargetData::ProgramInput(key) => BindingTarget::ProgramInput(key),
            BindingTargetData::ProgramOutput(keys) => BindingTarget::ProgramOutput(keys),
            BindingTargetData::Internal => BindingTarget::Internal,
        }
    }

    /// Returns the transport category of the binding.
    #[must_use]
    pub fn kind(self) -> BindingKind {
        self.data().kind
    }

    /// Returns the scalar carrier stored in physical memory.
    #[must_use]
    pub fn storage_scalar(self) -> tiler_ir::program::StorageScalar {
        self.data().storage_scalar
    }

    /// Returns the type through which the kernel accesses this binding.
    #[must_use]
    pub fn access_type(self) -> KernelType {
        self.data().access_type
    }

    /// Returns the stable addressed component role, absent for a dense singleton.
    #[must_use]
    pub fn component_role(self) -> Option<tiler_ir::semantic::EncodedComponentRole> {
        self.data().component_role
    }

    /// Returns the complete physical storage encoding.
    #[must_use]
    pub fn storage_encoding(self) -> tiler_ir::program::StorageEncoding {
        self.data().encoding
    }

    /// Returns the logical address space the binding requires.
    #[must_use]
    pub fn address_space(self) -> AddressSpace {
        self.data().address_space
    }

    /// Returns whether the entry reads or writes through the binding.
    #[must_use]
    pub fn access(self) -> BufferAccess {
        self.data().access
    }

    /// Returns the byte alignment the bound storage must satisfy.
    #[must_use]
    pub fn alignment(self) -> crate::program::AlignmentRequirement {
        self.data().alignment
    }

    /// Returns the first addressed byte of the bound value, as an expression.
    ///
    /// A slot may address part of its value, so this is a placement a loader
    /// must honour rather than a field that is always zero. Together with
    /// [`Self::accessible_bytes`] it is the exact range the entry reaches.
    #[must_use]
    pub fn accessible_offset(self) -> DecodedExpr<'a> {
        DecodedExpr {
            artifact: self.artifact,
            node: self.data().accessible_offset,
        }
    }

    /// Returns the minimum accessible byte range expression.
    ///
    /// Counted from [`Self::accessible_offset`], not from the start of the
    /// addressed value.
    #[must_use]
    pub fn accessible_bytes(self) -> DecodedExpr<'a> {
        DecodedExpr {
            artifact: self.artifact,
            node: self.data().accessible_bytes,
        }
    }

    fn data(self) -> &'a BindingData {
        &self.artifact.envelope.variants[self.variant].entries[self.entry].bindings[self.binding]
    }
}

/// One ABI expression of a decoded artifact, resolvable against bound facts.
///
/// Deliberately not a structural view. A dispatch needs the *value* — a byte
/// count, a thread count, a predicate — and publishing the tree as well would
/// duplicate a public vocabulary whose payload type is
/// [`AbiExprRef`](super::super::AbiExprRef), which hangs off a
/// `VerifiedArtifactProgram` no decode produces. An explain surface over a
/// decoded artifact would need it and none exists yet.
#[derive(Clone, Copy, Debug)]
pub struct DecodedExpr<'a> {
    artifact: &'a DecodedArtifact,
    node: u32,
}

impl DecodedExpr<'_> {
    /// Returns the value type this expression produces.
    #[must_use]
    pub fn value_type(self) -> AbiType {
        let nodes = &self.artifact.envelope.expressions;
        let mut types: Vec<AbiType> = Vec::with_capacity(position(self.node) + 1);
        for node in &nodes[..=position(self.node)] {
            types.push(node_type(node, &types));
        }
        types[position(self.node)]
    }

    /// Evaluates this expression against an already-bound fact environment.
    ///
    /// Checked arithmetic throughout, and a root the environment did not bind is
    /// a typed failure rather than a default. A decode already proved every
    /// operand's type and that no root escapes its availability phase, so what
    /// remains here is genuinely about the facts a caller supplied.
    ///
    /// # Errors
    ///
    /// Returns [`AbiEvaluationError`] for an unbound root, checked-arithmetic
    /// overflow or underflow, a zero divisor, an inexact exact division, or a
    /// narrowing that does not fit.
    pub fn evaluate(self, facts: &AbiFacts) -> Result<AbiValue, AbiEvaluationError> {
        evaluate(&self.artifact.envelope.expressions, self.node, facts)
    }
}

/// One framed section of a decoded artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SectionView<'a> {
    kind: SectionKind,
    bytes: &'a [u8],
}

impl<'a> SectionView<'a> {
    /// Returns what this section carries.
    #[must_use]
    pub const fn purpose(self) -> SectionPurpose {
        match self.kind {
            SectionKind::KernelProgramSubject => SectionPurpose::KernelProgramSubject,
            SectionKind::BackendPayloadMetadata => SectionPurpose::BackendPayloadMetadata,
            SectionKind::BackendPayloadCode => SectionPurpose::BackendPayloadCode,
        }
    }

    /// Returns the section's exact framed bytes.
    #[must_use]
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }
}

/// What one framed section of an artifact carries.
///
/// A public mirror of the codec's internal section vocabulary, written by an
/// exhaustive match rather than shared by re-export, so the wire vocabulary can
/// gain a purpose without that being a public change by default (ADR 0074
/// convention 3).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum SectionPurpose {
    /// The canonical identity of one packaged variant's kernel program.
    ///
    /// The identity alone. The program is not carried.
    KernelProgramSubject,
    /// The canonical compilation subject of one carried backend payload.
    ///
    /// These exact bytes are the payload's identity subject, so a descriptor's
    /// digest is a function of this section.
    BackendPayloadMetadata,
    /// The emitted object bytes of one carried backend payload.
    ///
    /// Carried opaquely under an integrity digest that artifact identity
    /// deliberately excludes, so relinking the same source yields the same
    /// artifact identity and different envelope bytes.
    BackendPayloadCode,
}

/// Why an artifact's bytes were rejected, or could not be produced.
///
/// # Why this is coarser than the codec's own vocabulary
///
/// `super::error`'s rejection enum names the exact boundary that refused, and
/// its variants carry internal subject enums — which section role, which
/// ordered table, which reference class. Publishing it would publish those too,
/// and they are the codec's working vocabulary rather than a contract. This
/// classifies instead: enough for a caller to decide what to *do*, with the
/// exact boundary preserved in [`fmt::Display`] for a person reading a log.
///
/// The classes answer different questions. Bytes that are not a Tiler artifact
/// at all, bytes that are one but were damaged, an artifact from a newer writer
/// this build cannot read, one that is well-formed but breaks an invariant, and
/// one that exceeds a governed bound are five different things to do next, and
/// collapsing them would make a version skew look like corruption.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a new class lands
/// additively.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ArtifactCodecFailure {
    /// The bytes are not a well-formed artifact envelope.
    ///
    /// Framing, magic, domain separation, or a length that does not agree with
    /// what it frames. Usually the wrong bytes entirely rather than bad ones.
    Malformed {
        /// The exact boundary that refused, for diagnosis.
        detail: String,
    },
    /// A digest did not match the content it covers.
    ///
    /// Distinct from [`Self::Malformed`] because the framing was readable: this
    /// is damage or tampering, not a category error.
    IntegrityFailure {
        /// The exact boundary that refused, for diagnosis.
        detail: String,
    },
    /// The artifact declares a schema, encoding, or feature this build does not
    /// implement.
    ///
    /// The artifact is not wrong; this reader is older than its writer. Failing
    /// closed here rather than ignoring the unknown part is what keeps a
    /// forward-incompatible artifact from being partially honoured.
    Unsupported {
        /// The exact schema, encoding, or feature that is not implemented.
        detail: String,
    },
    /// The artifact is well-formed but violates an invariant it must satisfy.
    ///
    /// Canonical order, arena closure, a dangling reference, a type or phase
    /// disagreement, or a re-derived identity that does not match the manifest.
    Invalid {
        /// The exact invariant that was violated.
        detail: String,
    },
    /// The artifact exceeds a governed structural bound.
    Limit {
        /// The exact bound that was exceeded.
        detail: String,
    },
}

impl fmt::Display for ArtifactCodecFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (class, detail) = match self {
            Self::Malformed { detail } => ("malformed", detail),
            Self::IntegrityFailure { detail } => ("integrity", detail),
            Self::Unsupported { detail } => ("unsupported", detail),
            Self::Invalid { detail } => ("invalid", detail),
            Self::Limit { detail } => ("limit", detail),
        };
        write!(formatter, "artifact.{class}: {detail}")
    }
}

impl Error for ArtifactCodecFailure {}

impl From<ArtifactCodecError> for ArtifactCodecFailure {
    /// Classifies one internal rejection.
    ///
    /// The match is exhaustive over every variant rather than falling through a
    /// wildcard, so a new codec boundary is a build error here and has to be
    /// classified deliberately instead of silently becoming whichever class the
    /// wildcard named.
    fn from(error: ArtifactCodecError) -> Self {
        let detail = error.to_string();
        match error {
            ArtifactCodecError::Truncated { .. }
            | ArtifactCodecError::TrailingBytes { .. }
            | ArtifactCodecError::TrailingManifestBytes { .. }
            | ArtifactCodecError::BadMagic
            | ArtifactCodecError::BadManifestDomain
            | ArtifactCodecError::BadPayloadMetadataDomain
            | ArtifactCodecError::TotalLengthMismatch { .. }
            | ArtifactCodecError::SectionLengthMismatch { .. }
            | ArtifactCodecError::SectionCountMismatch { .. }
            | ArtifactCodecError::InvalidText
            | ArtifactCodecError::InvalidGovernedKey { .. }
            | ArtifactCodecError::InvalidInterfaceKey { .. }
            | ArtifactCodecError::InvalidProviderIdentity { .. }
            | ArtifactCodecError::InvalidShape { .. }
            | ArtifactCodecError::InvalidAlignment { .. }
            | ArtifactCodecError::UnknownTag { .. } => Self::Malformed { detail },

            ArtifactCodecError::ManifestDigestMismatch
            | ArtifactCodecError::SectionDigestMismatch { .. }
            | ArtifactCodecError::PayloadIdentityMismatch { .. }
            | ArtifactCodecError::ArtifactIdentityMismatch => Self::IntegrityFailure { detail },

            ArtifactCodecError::UnsupportedEnvelopeFormat { .. }
            | ArtifactCodecError::UnsupportedCanonicalEncoding { .. }
            | ArtifactCodecError::UnsupportedManifestSchema { .. }
            | ArtifactCodecError::UnsupportedComponentSchema { .. }
            | ArtifactCodecError::UnsupportedDigestAlgorithm { .. }
            | ArtifactCodecError::UnsupportedRequiredFeature { .. }
            | ArtifactCodecError::UnsupportedSectionSchema { .. }
            | ArtifactCodecError::UnsupportedPayloadMetadataSchema { .. } => {
                Self::Unsupported { detail }
            }

            ArtifactCodecError::Limit { .. } => Self::Limit { detail },

            ArtifactCodecError::SectionDispositionMismatch { .. }
            | ArtifactCodecError::SectionPurposeMismatch { .. }
            | ArtifactCodecError::NonCanonicalSectionId { .. }
            | ArtifactCodecError::NonCanonicalOrder { .. }
            | ArtifactCodecError::DuplicateItem { .. }
            | ArtifactCodecError::PlatformFieldWithoutPlatform { .. }
            | ArtifactCodecError::NonCanonicalManifest
            | ArtifactCodecError::UnreferencedSection { .. }
            | ArtifactCodecError::EmptyBindingTarget
            | ArtifactCodecError::UnknownBindingTargetKey { .. }
            | ArtifactCodecError::UnmappedBackendEntry { .. }
            | ArtifactCodecError::EntryTransportCardinality { .. }
            | ArtifactCodecError::DeclaredFeatureMismatch
            | ArtifactCodecError::MissingReference { .. }
            | ArtifactCodecError::ExpressionOperandOrder { .. }
            | ArtifactCodecError::ExpressionOperandType { .. }
            | ArtifactCodecError::ExpressionSelectBranchType { .. }
            | ArtifactCodecError::ModelRule { .. }
            | ArtifactCodecError::ModelObligation { .. }
            | ArtifactCodecError::IdentityDerivation { .. }
            | ArtifactCodecError::StageOrderNotAPermutation { .. }
            | ArtifactCodecError::StageDependencyOutOfOrder { .. }
            | ArtifactCodecError::StageDependencyOnItself { .. }
            | ArtifactCodecError::MalformedInterfaceComponents
            | ArtifactCodecError::UnknownBindingTargetComponent { .. }
            | ArtifactCodecError::BindingComponentMismatch
            // The record failing on its own terms is classified `Invalid` and
            // not `Malformed`, including its unknown-tag and truncation rules.
            // The framing was readable — the manifest's own length prefix
            // delimited the run and the run's domain separator opened it — so
            // what refused is an artifact that is well formed and states an
            // invalid numerical realization, which is what `Invalid` names.
            | ArtifactCodecError::DeliveredRealization { .. }
            | ArtifactCodecError::BindingAccessTypeMismatch => Self::Invalid { detail },
        }
    }
}
