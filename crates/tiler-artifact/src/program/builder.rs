//! Transactional builder for the target-neutral artifact program model.
//!
//! Construction follows the ADR 0071 discipline: a public transactional builder
//! with private storage, insertion-time checks for every locally decidable
//! invariant, and a consuming [`ArtifactProgramBuilder::build`] that runs
//! whole-artifact verification and returns an opaque
//! [`VerifiedArtifactProgram`] or the intact builder with typed diagnostics.
//!
//! Two things are taken rather than declared, so a producer cannot assert them.
//!
//! The **semantic subject** comes from a verified
//! [`SemanticProgram`]: its four-subject identity bundle and its ordered named
//! interface. Every packaged variant must realize that exact graph.
//!
//! The **ABI's derived facts** come from the variant's own verified program and
//! kernels: a binding's element type, address space, access mode, alignment,
//! and program role are read from the kernel signature and the materialized
//! value it addresses, never taken from the producer. A producer supplies only
//! the expressions — accessible ranges, launch geometry, guards, preconditions,
//! and deferred predicates — and each is proven against the program it claims
//! to describe.

use tiler_ir::program::{MaterializedOrigin, StageRef, VerifiedKernelProgram};
use tiler_ir::schedule::NumericalRealization;
use tiler_ir::semantic::{
    InputKey, OutputKey, ProviderIdentity, SemanticIdentity, SemanticProgram,
};
use tiler_ir::shape::Shape;

use super::codec::{ArtifactEnvelope, PayloadContent};
use super::error::{
    AbiExprUse, ArtifactBuildError, ArtifactEntityKind, ArtifactLimitKind,
    ArtifactVerificationError, invalid_handle, limit,
};
use super::expr::{
    AbiBinaryOp, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue, AvailabilityPhase, ExprNode,
    binary_operand_type, evaluate, expr_key, node_is_interface_only, node_phase, node_type,
    unary_operand_type,
};
use super::facts::AbiFactBinder;
use super::handles::{
    AbiExprId, ArtifactBuilderId, PayloadId, VariantId, next_artifact_builder_id,
};
use super::keys::{BackendKey, FeasibilityRuleSetRef, RepresentationKey, TargetProfileRef};
use super::model::{
    ArtifactExecutionPolicy, ArtifactProgramData, ArtifactSchema, BackendEntryRef,
    BackendPayloadDescriptor, BindingData, BindingKind, DeferredPredicateData, EntryData,
    InterfaceEntryData, LaunchData, RoutingPolicy, SchemaVersion, SelectedProvider,
    StoredBackendEntry, VariantData, VerifiedArtifactProgram, encode_identity,
};
use super::{
    MAX_ABI_EXPRESSIONS, MAX_ARTIFACT_PAYLOADS, MAX_ARTIFACT_VARIANTS, MAX_DEFERRED_PREDICATES,
    MAX_ENTRY_BINDINGS, MAX_ENVIRONMENT_PROVIDERS, MAX_LAUNCH_PRECONDITIONS,
    MAX_SELECTED_PROVIDERS, MAX_VARIANT_ENTRIES,
};

/// The complete frozen set of capability providers offered to one compilation.
///
/// This is a construction-time authority, not artifact content. It exists so
/// that a selected provider can be proven to have actually been offered; the
/// providers it lists but the plan never reaches are deliberately **not**
/// retained by the verified artifact and can therefore never enter its
/// identity (ADR 0072). Keeping the complete registry snapshot is a
/// compilation-request concern that lives outside the runtime artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilationEnvironment {
    available: Vec<ProviderIdentity>,
}

impl CompilationEnvironment {
    /// Freezes the providers offered to one compilation.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::StructuralLimit`] beyond the governed
    /// environment bound.
    pub fn new(
        providers: impl IntoIterator<Item = ProviderIdentity>,
    ) -> Result<Self, ArtifactBuildError> {
        let mut available: Vec<ProviderIdentity> = providers.into_iter().collect();
        limit(
            available.len(),
            MAX_ENVIRONMENT_PROVIDERS,
            ArtifactLimitKind::EnvironmentProviders,
        )?;
        available.sort_unstable_by(|left, right| {
            (left.namespace(), left.name(), left.revision()).cmp(&(
                right.namespace(),
                right.name(),
                right.revision(),
            ))
        });
        available.dedup();
        Ok(Self { available })
    }

    /// Returns the offered providers in canonical order.
    #[must_use]
    pub fn available(&self) -> &[ProviderIdentity] {
        &self.available
    }

    fn offers(&self, provider: &ProviderIdentity) -> bool {
        self.available.contains(provider)
    }
}

/// The unforgeable ordered semantic interface every packaged variant realizes.
#[derive(Clone, Debug)]
struct SemanticInterface {
    inputs: Vec<(InputKey, Shape)>,
    outputs: Vec<(OutputKey, Shape)>,
}

/// The portfolio-wide facts the first packaged variant establishes.
#[derive(Clone, Debug)]
struct PortfolioSubject {
    inputs: Vec<InterfaceEntryData<InputKey>>,
    outputs: Vec<InterfaceEntryData<OutputKey>>,
    numerical: NumericalRealization,
    profile: TargetProfileRef,
}

/// One deferred feasibility predicate a plan variant declares.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeferredPredicateSpec {
    /// Predicate that must hold before routing commits.
    pub predicate: AbiExprId,
    /// Phase at which the predicate becomes decidable.
    pub phase: AvailabilityPhase,
    /// Selected provider that must answer the query.
    pub authority: ProviderIdentity,
}

/// One ABI binding a producer declares for an executable entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingSpec {
    /// Transport category of the binding.
    pub kind: BindingKind,
    /// Minimum accessible byte range the entry requires.
    pub accessible_bytes: AbiExprId,
}

/// The launch contract a producer declares for an executable entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchSpec {
    /// Total launch thread count.
    pub grid_threads: AbiExprId,
    /// Threads per workgroup.
    pub threads_per_workgroup: AbiExprId,
    /// Whether a zero-thread launch skips the dispatch entirely.
    pub zero_work_skips_dispatch: bool,
    /// Launch-instance preconditions evaluated before routing commits.
    pub preconditions: Vec<AbiExprId>,
}

/// One executable entry a producer declares for a program stage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntrySpec {
    /// ABI bindings in kernel buffer-parameter order.
    pub bindings: Vec<BindingSpec>,
    /// Launch contract of the entry.
    pub launch: LaunchSpec,
    /// Backend entry that realizes this entry.
    pub implementation: BackendEntryRef,
}

/// One complete plan variant a producer declares.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantSpec {
    /// Guard deciding whether this variant may be routed to.
    pub applicability_guard: AbiExprId,
    /// Declared target profile the variant was assessed against.
    pub target_profile: TargetProfileRef,
    /// Feasibility rule set the variant was assessed under.
    pub feasibility_rules: FeasibilityRuleSetRef,
    /// Deferred feasibility predicates, each with its query authority.
    pub deferred_predicates: Vec<DeferredPredicateSpec>,
    /// Executable entries, one per stage of the variant's program.
    pub entries: Vec<EntrySpec>,
}

/// A transactional artifact-program builder with private storage.
#[derive(Clone, Debug)]
pub struct ArtifactProgramBuilder {
    owner: ArtifactBuilderId,
    semantic: SemanticIdentity,
    interface: SemanticInterface,
    environment: CompilationEnvironment,
    providers: Vec<SelectedProvider>,
    payloads: Vec<BackendPayloadDescriptor>,
    payload_content: Vec<Option<PayloadContent>>,
    expressions: Vec<ExprNode>,
    expression_keys: Vec<Vec<u8>>,
    expression_types: Vec<AbiType>,
    expression_phases: Vec<AvailabilityPhase>,
    expression_interface_only: Vec<bool>,
    variants: Vec<VariantData>,
    subject: Option<PortfolioSubject>,
    routing: RoutingPolicy,
}

impl ArtifactProgramBuilder {
    /// Opens a builder bound to one verified semantic program and environment.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::BuilderIdentityExhausted`] when no fresh
    /// builder ownership identity remains.
    pub fn new(
        semantic: &SemanticProgram,
        environment: CompilationEnvironment,
    ) -> Result<Self, ArtifactBuildError> {
        let owner =
            next_artifact_builder_id().ok_or(ArtifactBuildError::BuilderIdentityExhausted)?;
        Ok(Self {
            owner,
            semantic: semantic.semantic_identity().clone(),
            interface: read_semantic_interface(semantic),
            environment,
            providers: Vec::new(),
            payloads: Vec::new(),
            payload_content: Vec::new(),
            expressions: Vec::new(),
            expression_keys: Vec::new(),
            expression_types: Vec::new(),
            expression_phases: Vec::new(),
            expression_interface_only: Vec::new(),
            variants: Vec::new(),
            subject: None,
            routing: RoutingPolicy::StablePriority,
        })
    }

    /// Records one capability provider the packaged plan actually reached.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::ProviderNotAvailable`] when the
    /// compilation environment never offered the provider,
    /// [`ArtifactBuildError::DuplicateSelectedProvider`] for a repeated
    /// selection, or a structural-limit error.
    pub fn select_provider(
        &mut self,
        selected: SelectedProvider,
    ) -> Result<(), ArtifactBuildError> {
        if !self.environment.offers(&selected.provider) {
            return Err(ArtifactBuildError::ProviderNotAvailable {
                provider: Box::new(selected.provider),
            });
        }
        if self.providers.contains(&selected) {
            return Err(ArtifactBuildError::DuplicateSelectedProvider {
                provider: Box::new(selected.provider),
            });
        }
        limit(
            self.providers.len().saturating_add(1),
            MAX_SELECTED_PROVIDERS,
            ArtifactLimitKind::SelectedProviders,
        )?;
        self.providers.push(selected);
        Ok(())
    }

    /// Declares one backend payload descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::DuplicatePayload`] for an identical
    /// descriptor, or a structural-limit error.
    pub fn push_payload(
        &mut self,
        descriptor: BackendPayloadDescriptor,
    ) -> Result<PayloadId, ArtifactBuildError> {
        if self.payloads.contains(&descriptor) {
            return Err(ArtifactBuildError::DuplicatePayload);
        }
        limit(
            self.payloads.len().saturating_add(1),
            MAX_ARTIFACT_PAYLOADS,
            ArtifactLimitKind::Payloads,
        )?;
        let id = PayloadId::from_len(self.owner, self.payloads.len()).ok_or(
            ArtifactBuildError::StructuralLimit {
                resource: ArtifactLimitKind::Payloads,
                actual: self.payloads.len().saturating_add(1),
                limit: MAX_ARTIFACT_PAYLOADS,
            },
        )?;
        self.payloads.push(descriptor);
        self.payload_content.push(None);
        Ok(id)
    }

    /// Declares one backend payload and carries its content in the artifact.
    ///
    /// The descriptor's content digest is not supplied: it is *derived* from
    /// the exact canonical payload-metadata bytes, so a carried payload cannot
    /// claim a compilation subject other than the one it carries. That is the
    /// identity decision this layer encodes — a payload is content-addressed
    /// over its compilation inputs, and the emitted object travels opaquely
    /// under an integrity digest that artifact identity deliberately excludes.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::DuplicatePayload`] for a descriptor this
    /// artifact already declares, a structural-limit error, or the identity
    /// error the digest constructor produced.
    #[allow(
        dead_code,
        reason = "the carried-payload entry point is the crate-private draft half of `prototype-metal-bundle-assembly` (ADR 0074 convention 7). It reserves the invariant that a carried payload's descriptor digest is derived from its subject rather than supplied; its first non-test consumer is the backend assembler that does not exist yet."
    )]
    pub(crate) fn push_carried_payload(
        &mut self,
        backend: BackendKey,
        representation: RepresentationKey,
        payload_schema: SchemaVersion,
        execution_policy: ArtifactExecutionPolicy,
        content: PayloadContent,
    ) -> Result<PayloadId, ArtifactBuildError> {
        let descriptor = BackendPayloadDescriptor {
            backend,
            representation,
            payload_schema,
            digest: content.identity()?,
            execution_policy,
        };
        let id = self.push_payload(descriptor)?;
        let position = self.payloads.len() - 1;
        self.payload_content[position] = Some(content);
        Ok(id)
    }

    /// Declares one typed root fact of the shared ABI expression arena.
    ///
    /// The arena is canonically deduplicated: an identical expression returns
    /// the handle already minted for it, so the arena stays a function of
    /// content rather than of how often a producer rebuilt the same formula.
    ///
    /// # Errors
    ///
    /// Returns a structural-limit error.
    pub fn push_root(&mut self, root: AbiRoot) -> Result<AbiExprId, ArtifactBuildError> {
        self.push_node(ExprNode::Root(root))
    }

    /// Declares one unary operation over an existing expression.
    ///
    /// # Errors
    ///
    /// Returns a handle error, [`ArtifactBuildError::OperandType`] for a
    /// mistyped operand, or a structural-limit error.
    pub fn push_unary(
        &mut self,
        op: AbiUnaryOp,
        operand: AbiExprId,
    ) -> Result<AbiExprId, ArtifactBuildError> {
        let operand = self.resolve_expression(operand)?;
        self.expect_type(operand, unary_operand_type(op))?;
        self.push_node(ExprNode::Unary { op, operand })
    }

    /// Declares one binary operation over two existing expressions.
    ///
    /// # Errors
    ///
    /// Returns a handle error, [`ArtifactBuildError::OperandType`] for a
    /// mistyped operand, or a structural-limit error.
    pub fn push_binary(
        &mut self,
        op: AbiBinaryOp,
        left: AbiExprId,
        right: AbiExprId,
    ) -> Result<AbiExprId, ArtifactBuildError> {
        let left = self.resolve_expression(left)?;
        let right = self.resolve_expression(right)?;
        self.expect_type(left, binary_operand_type(op))?;
        self.expect_type(right, binary_operand_type(op))?;
        self.push_node(ExprNode::Binary { op, left, right })
    }

    /// Declares one conditional selection between two equally typed branches.
    ///
    /// Only the selected branch is evaluated, so a branch that would fail on a
    /// zero-sized bound is legal behind a guarding condition.
    ///
    /// # Errors
    ///
    /// Returns a handle error, [`ArtifactBuildError::OperandType`] for a
    /// non-predicate condition, [`ArtifactBuildError::SelectBranchType`] for
    /// disagreeing branches, or a structural-limit error.
    pub fn push_select(
        &mut self,
        condition: AbiExprId,
        if_true: AbiExprId,
        if_false: AbiExprId,
    ) -> Result<AbiExprId, ArtifactBuildError> {
        let condition = self.resolve_expression(condition)?;
        let if_true = self.resolve_expression(if_true)?;
        let if_false = self.resolve_expression(if_false)?;
        self.expect_type(condition, AbiType::Boolean)?;
        let (left, right) = (
            self.expression_types[usize_of(if_true)],
            self.expression_types[usize_of(if_false)],
        );
        if left != right {
            return Err(ArtifactBuildError::SelectBranchType {
                if_true: left,
                if_false: right,
            });
        }
        self.push_node(ExprNode::Select {
            condition,
            if_true,
            if_false,
        })
    }

    /// Declares one complete plan variant at the next routing rank.
    ///
    /// Every provider a deferred predicate names must already be selected, so
    /// [`ArtifactProgramBuilder::select_provider`] precedes this call.
    ///
    /// # Errors
    ///
    /// Returns a handle error; a semantic-subject, interface, numerical, or
    /// target-profile disagreement; an entry or binding cardinality error; an
    /// expression type, root-phase, or interface-root rejection; an accessible
    /// range, launch, or zero-work disagreement with the bound program; a
    /// duplicate variant, deferred predicate, or launch precondition; an
    /// unselected deferred authority; a non-deferred predicate phase; or a
    /// structural-limit error.
    pub fn push_variant(
        &mut self,
        program: &VerifiedKernelProgram,
        spec: VariantSpec,
    ) -> Result<VariantId, ArtifactBuildError> {
        if program.semantic_graph_identity() != self.semantic.graph() {
            return Err(ArtifactBuildError::SemanticSubjectMismatch);
        }
        limit(
            self.variants.len().saturating_add(1),
            MAX_ARTIFACT_VARIANTS,
            ArtifactLimitKind::Variants,
        )?;
        let stages = program.stages().len();
        if stages != spec.entries.len() {
            return Err(ArtifactBuildError::EntryCardinality {
                expected: stages,
                actual: spec.entries.len(),
            });
        }
        limit(stages, MAX_VARIANT_ENTRIES, ArtifactLimitKind::Entries)?;
        let guard = self.check_use(
            spec.applicability_guard,
            AbiExprUse::ApplicabilityGuard,
            AbiType::Boolean,
            AvailabilityPhase::LiveDevicePreflight,
            false,
        )?;
        let deferred = self.check_deferred(&spec.deferred_predicates)?;
        let facts = static_facts(program);
        let entries = self.check_entries(program, &spec.entries, &facts)?;
        let subject = self.check_subject(program, &spec)?;
        let id = VariantId::from_len(self.owner, self.variants.len()).ok_or(
            ArtifactBuildError::StructuralLimit {
                resource: ArtifactLimitKind::Variants,
                actual: self.variants.len().saturating_add(1),
                limit: MAX_ARTIFACT_VARIANTS,
            },
        )?;
        if self.variants.iter().any(|existing| {
            existing.program.canonical_identity() == program.canonical_identity()
                && existing.guard == guard
        }) {
            return Err(ArtifactBuildError::DuplicateVariant);
        }
        self.subject = Some(subject);
        self.variants.push(VariantData {
            program: program.clone(),
            guard,
            profile: spec.target_profile,
            feasibility_rules: spec.feasibility_rules,
            deferred,
            entries,
        });
        Ok(id)
    }

    /// Verifies the whole artifact and freezes it, or returns the intact builder.
    ///
    /// # Errors
    ///
    /// Returns an [`ArtifactVerificationError`] carrying every whole-artifact
    /// diagnostic and the recoverable builder when verification fails.
    pub fn build(self) -> Result<VerifiedArtifactProgram, ArtifactVerificationError> {
        let data = self.assemble();
        let mut diagnostics = super::verify::verify_artifact(&data);
        if diagnostics.is_empty() {
            // Identity is derived from the canonical envelope so that the bytes
            // a producer stamps and the bytes a decoder re-derives come from
            // one encoder rather than from two that agree by inspection.
            match ArtifactEnvelope::project(&data).and_then(|envelope| encode_identity(&envelope)) {
                Ok(identity) => return Ok(VerifiedArtifactProgram { data, identity }),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }
        Err(ArtifactVerificationError {
            builder: Box::new(self),
            diagnostics,
        })
    }

    fn assemble(&self) -> ArtifactProgramData {
        let (inputs, outputs) = self.subject.as_ref().map_or_else(
            || (Vec::new(), Vec::new()),
            |subject| (subject.inputs.clone(), subject.outputs.clone()),
        );
        ArtifactProgramData {
            schema: ArtifactSchema::GOVERNED,
            semantic: self.semantic.clone(),
            routing: self.routing,
            inputs,
            outputs,
            providers: self.providers.clone(),
            payloads: self.payloads.clone(),
            payload_content: self.payload_content.clone(),
            expressions: self.expressions.clone(),
            expression_keys: self.expression_keys.clone(),
            expression_types: self.expression_types.clone(),
            variants: self.variants.clone(),
        }
    }

    fn push_node(&mut self, node: ExprNode) -> Result<AbiExprId, ArtifactBuildError> {
        let key = expr_key(&node, &self.expression_keys);
        if let Some(existing) = self.expression_keys.iter().position(|held| *held == key) {
            return AbiExprId::from_len(self.owner, existing).ok_or(
                ArtifactBuildError::StructuralLimit {
                    resource: ArtifactLimitKind::Expressions,
                    actual: existing,
                    limit: MAX_ABI_EXPRESSIONS,
                },
            );
        }
        limit(
            self.expressions.len().saturating_add(1),
            MAX_ABI_EXPRESSIONS,
            ArtifactLimitKind::Expressions,
        )?;
        let id = AbiExprId::from_len(self.owner, self.expressions.len()).ok_or(
            ArtifactBuildError::StructuralLimit {
                resource: ArtifactLimitKind::Expressions,
                actual: self.expressions.len().saturating_add(1),
                limit: MAX_ABI_EXPRESSIONS,
            },
        )?;
        self.expression_types
            .push(node_type(&node, &self.expression_types));
        self.expression_phases
            .push(node_phase(&node, &self.expression_phases));
        self.expression_interface_only.push(node_is_interface_only(
            &node,
            &self.expression_interface_only,
        ));
        self.expression_keys.push(key);
        self.expressions.push(node);
        Ok(id)
    }

    fn expect_type(&self, node: u32, expected: AbiType) -> Result<(), ArtifactBuildError> {
        let actual = self.expression_types[usize_of(node)];
        if actual == expected {
            Ok(())
        } else {
            Err(ArtifactBuildError::OperandType { expected, actual })
        }
    }

    /// Resolves one expression handle against a declared artifact use site.
    ///
    /// A use site fixes the value type, the latest phase its roots may require,
    /// and whether the expression must be computable from the bound semantic
    /// interface alone.
    fn check_use(
        &self,
        id: AbiExprId,
        use_site: AbiExprUse,
        expected: AbiType,
        admitted_through: AvailabilityPhase,
        interface_only: bool,
    ) -> Result<u32, ArtifactBuildError> {
        let node = self.resolve_expression(id)?;
        let actual = self.expression_types[usize_of(node)];
        if actual != expected {
            return Err(ArtifactBuildError::ExpressionType {
                use_site,
                expected,
                actual,
            });
        }
        let available_at = self.expression_phases[usize_of(node)];
        if available_at > admitted_through {
            return Err(ArtifactBuildError::RootPhaseEscape {
                use_site,
                available_at,
                admitted_through,
            });
        }
        if interface_only && !self.expression_interface_only[usize_of(node)] {
            return Err(ArtifactBuildError::NonInterfaceRoot { use_site });
        }
        Ok(node)
    }

    fn check_deferred(
        &self,
        specs: &[DeferredPredicateSpec],
    ) -> Result<Vec<DeferredPredicateData>, ArtifactBuildError> {
        limit(
            specs.len(),
            MAX_DEFERRED_PREDICATES,
            ArtifactLimitKind::DeferredPredicates,
        )?;
        let mut resolved = Vec::with_capacity(specs.len());
        for spec in specs {
            if spec.phase < AvailabilityPhase::LiveDevicePreflight {
                return Err(ArtifactBuildError::NonDeferredPredicatePhase { phase: spec.phase });
            }
            if !self
                .providers
                .iter()
                .any(|selected| selected.provider == spec.authority)
            {
                return Err(ArtifactBuildError::UnselectedDeferredAuthority {
                    provider: Box::new(spec.authority.clone()),
                });
            }
            let predicate = self.check_use(
                spec.predicate,
                AbiExprUse::DeferredPredicate,
                AbiType::Boolean,
                spec.phase,
                false,
            )?;
            if resolved.iter().any(|held: &DeferredPredicateData| {
                held.predicate == predicate
                    && held.phase == spec.phase
                    && held.authority == spec.authority
            }) {
                return Err(ArtifactBuildError::DuplicateDeferredPredicate);
            }
            resolved.push(DeferredPredicateData {
                predicate,
                phase: spec.phase,
                authority: spec.authority.clone(),
            });
        }
        Ok(resolved)
    }

    fn check_entries(
        &self,
        program: &VerifiedKernelProgram,
        specs: &[EntrySpec],
        facts: &AbiFacts,
    ) -> Result<Vec<EntryData>, ArtifactBuildError> {
        let mut resolved = Vec::with_capacity(specs.len());
        for (index, (spec, stage)) in specs.iter().zip(program.stages()).enumerate() {
            let bindings = self.check_bindings(index, spec, stage, facts)?;
            let launch = self.check_launch(index, &spec.launch, stage, facts)?;
            let payload = self.resolve_payload(spec.implementation.payload)?;
            resolved.push(EntryData {
                bindings,
                launch,
                implementation: StoredBackendEntry {
                    payload,
                    entry_key: spec.implementation.entry_key.clone(),
                },
            });
        }
        Ok(resolved)
    }

    fn check_bindings(
        &self,
        entry: usize,
        spec: &EntrySpec,
        stage: StageRef<'_>,
        facts: &AbiFacts,
    ) -> Result<Vec<BindingData>, ArtifactBuildError> {
        let buffers: Vec<_> = stage.kernel().buffers().collect();
        if buffers.len() != spec.bindings.len() {
            return Err(ArtifactBuildError::BindingCardinality {
                entry,
                expected: buffers.len(),
                actual: spec.bindings.len(),
            });
        }
        limit(
            spec.bindings.len(),
            MAX_ENTRY_BINDINGS,
            ArtifactLimitKind::EntryBindings,
        )?;
        let mut resolved = Vec::with_capacity(spec.bindings.len());
        for (slot, ((binding, buffer), access)) in spec
            .bindings
            .iter()
            .zip(&buffers)
            .zip(stage.accesses())
            .enumerate()
        {
            let node = self.check_use(
                binding.accessible_bytes,
                AbiExprUse::AccessibleBytes,
                AbiType::Unsigned,
                AvailabilityPhase::LiveDevicePreflight,
                true,
            )?;
            let computed = self.evaluate_static(node, AbiExprUse::AccessibleBytes, facts)?;
            let expected = access.view().window().length;
            if computed != expected {
                return Err(ArtifactBuildError::AccessibleBytesDisagreement {
                    entry,
                    binding: slot,
                    expected,
                    actual: computed,
                });
            }
            let value = access.view().value();
            resolved.push(BindingData {
                kind: binding.kind,
                element_type: buffer.element_type,
                address_space: buffer.address_space,
                access: buffer.access,
                alignment: value.alignment(),
                value_role: value.role(),
                accessible_bytes: node,
            });
        }
        Ok(resolved)
    }

    fn check_launch(
        &self,
        entry: usize,
        spec: &LaunchSpec,
        stage: StageRef<'_>,
        facts: &AbiFacts,
    ) -> Result<LaunchData, ArtifactBuildError> {
        let grid_threads = self.check_use(
            spec.grid_threads,
            AbiExprUse::LaunchThreads,
            AbiType::Unsigned,
            AvailabilityPhase::LiveDevicePreflight,
            true,
        )?;
        let threads_per_workgroup = self.check_use(
            spec.threads_per_workgroup,
            AbiExprUse::ThreadsPerWorkgroup,
            AbiType::Unsigned,
            AvailabilityPhase::LiveDevicePreflight,
            true,
        )?;
        let required = u64::from(stage.kernel().requirements().threads_per_workgroup);
        let declared = self.evaluate_static(
            threads_per_workgroup,
            AbiExprUse::ThreadsPerWorkgroup,
            facts,
        )?;
        if declared != required {
            return Err(ArtifactBuildError::LaunchDisagreement {
                entry,
                expected: required,
                actual: declared,
            });
        }
        let threads = self.evaluate_static(grid_threads, AbiExprUse::LaunchThreads, facts)?;
        if threads == 0 && !spec.zero_work_skips_dispatch {
            return Err(ArtifactBuildError::ZeroWorkPolicy { entry });
        }
        limit(
            spec.preconditions.len(),
            MAX_LAUNCH_PRECONDITIONS,
            ArtifactLimitKind::LaunchPreconditions,
        )?;
        let mut preconditions = Vec::with_capacity(spec.preconditions.len());
        for precondition in &spec.preconditions {
            let node = self.check_use(
                *precondition,
                AbiExprUse::LaunchPrecondition,
                AbiType::Boolean,
                AvailabilityPhase::LaunchPreflight,
                false,
            )?;
            if preconditions.contains(&node) {
                return Err(ArtifactBuildError::DuplicateLaunchPrecondition { entry });
            }
            preconditions.push(node);
        }
        Ok(LaunchData {
            grid_threads,
            threads_per_workgroup,
            zero_work_skips_dispatch: spec.zero_work_skips_dispatch,
            preconditions,
        })
    }

    /// Evaluates one interface-only expression against the declared static shapes.
    ///
    /// This is a compile-time consistency check, not a runtime evaluation: the
    /// facts are the program's own declared input shapes, so a producer cannot
    /// declare an accessible range or a launch geometry that its own program
    /// contradicts.
    fn evaluate_static(
        &self,
        node: u32,
        use_site: AbiExprUse,
        facts: &AbiFacts,
    ) -> Result<u64, ArtifactBuildError> {
        match evaluate(&self.expressions, node, facts) {
            Ok(AbiValue::Unsigned(value)) => Ok(value),
            Ok(AbiValue::Boolean(_)) => Err(ArtifactBuildError::ExpressionType {
                use_site,
                expected: AbiType::Unsigned,
                actual: AbiType::Boolean,
            }),
            Err(cause) => Err(ArtifactBuildError::StaticEvaluation { use_site, cause }),
        }
    }

    fn check_subject(
        &self,
        program: &VerifiedKernelProgram,
        spec: &VariantSpec,
    ) -> Result<PortfolioSubject, ArtifactBuildError> {
        let (inputs, outputs) = project_interface(program, &self.interface)?;
        let numerical = program
            .stages()
            .next()
            .ok_or(ArtifactBuildError::EntryCardinality {
                expected: 1,
                actual: 0,
            })?
            .kernel()
            .numerical();
        if program
            .stages()
            .any(|stage| stage.kernel().numerical() != numerical)
        {
            return Err(ArtifactBuildError::NumericalContractMismatch);
        }
        if let Some(subject) = &self.subject {
            if subject.inputs != inputs || subject.outputs != outputs {
                return Err(ArtifactBuildError::InterfaceMismatch);
            }
            if subject.numerical != numerical {
                return Err(ArtifactBuildError::NumericalContractMismatch);
            }
            if subject.profile != spec.target_profile {
                return Err(ArtifactBuildError::TargetProfileMismatch);
            }
        }
        Ok(PortfolioSubject {
            inputs,
            outputs,
            numerical,
            profile: spec.target_profile.clone(),
        })
    }

    /// Evaluates one draft expression, for tests that never package a variant.
    ///
    /// Evaluation is otherwise a property of the verified product; this exists
    /// so an expression-language test does not have to assemble a whole
    /// artifact around the node it is exercising.
    #[cfg(test)]
    pub(super) fn evaluate_draft_expression(
        &self,
        id: AbiExprId,
        facts: &AbiFacts,
    ) -> Result<AbiValue, super::expr::AbiEvaluationError> {
        let node = self
            .resolve_expression(id)
            .expect("a test evaluates a node this builder minted");
        evaluate(&self.expressions, node, facts)
    }

    fn resolve_expression(&self, id: AbiExprId) -> Result<u32, ArtifactBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ArtifactEntityKind::Expression, true));
        }
        if id.as_usize() >= self.expressions.len() {
            return Err(invalid_handle(ArtifactEntityKind::Expression, false));
        }
        Ok(id.index)
    }

    fn resolve_payload(&self, id: PayloadId) -> Result<u32, ArtifactBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ArtifactEntityKind::Payload, true));
        }
        if id.as_usize() >= self.payloads.len() {
            return Err(invalid_handle(ArtifactEntityKind::Payload, false));
        }
        Ok(id.index)
    }
}

fn usize_of(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

/// Reads the ordered named interface of a verified semantic program.
fn read_semantic_interface(semantic: &SemanticProgram) -> SemanticInterface {
    let inputs = semantic
        .inputs()
        .map(|input| {
            let shape = semantic
                .shape(input.value())
                .expect("a verified program resolves its own input value")
                .clone();
            (input.key().clone(), shape)
        })
        .collect();
    let outputs = semantic
        .outputs()
        .map(|output| {
            let shape = semantic
                .shape(output.value())
                .expect("a verified program resolves its own output value")
                .clone();
            (output.key().clone(), shape)
        })
        .collect();
    SemanticInterface { inputs, outputs }
}

/// Projects the artifact's published interface from one variant's program.
///
/// The order is the semantic interface's, which the kernel program does not
/// retain. The storage element type comes from the materialized value the
/// program binds, so the published contract is a fact of the plan rather than a
/// producer claim.
#[allow(
    clippy::type_complexity,
    reason = "the pair is the artifact's two interface projections and is destructured at its only call site"
)]
fn project_interface(
    program: &VerifiedKernelProgram,
    interface: &SemanticInterface,
) -> Result<
    (
        Vec<InterfaceEntryData<InputKey>>,
        Vec<InterfaceEntryData<OutputKey>>,
    ),
    ArtifactBuildError,
> {
    let mut inputs = Vec::with_capacity(interface.inputs.len());
    for (key, shape) in &interface.inputs {
        let value = program
            .values()
            .find(|value| match value.origin() {
                MaterializedOrigin::ProgramInput { key: bound } => bound == key,
                MaterializedOrigin::Internal => false,
            })
            .ok_or(ArtifactBuildError::InterfaceMismatch)?;
        if value.shape() != shape {
            return Err(ArtifactBuildError::InterfaceMismatch);
        }
        inputs.push(InterfaceEntryData {
            key: key.clone(),
            shape: shape.clone(),
            element_type: value.element_type(),
        });
    }
    let mut outputs = Vec::with_capacity(interface.outputs.len());
    for (key, shape) in &interface.outputs {
        let published = program
            .outputs()
            .find(|output| output.key() == key)
            .ok_or(ArtifactBuildError::InterfaceMismatch)?;
        if published.value().shape() != shape {
            return Err(ArtifactBuildError::InterfaceMismatch);
        }
        outputs.push(InterfaceEntryData {
            key: key.clone(),
            shape: shape.clone(),
            element_type: published.value().element_type(),
        });
    }
    Ok((inputs, outputs))
}

/// Builds the declared-shape fact environment one program's ABI is checked in.
fn static_facts(program: &VerifiedKernelProgram) -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    for value in program.values() {
        if let MaterializedOrigin::ProgramInput { key } = value.origin() {
            binder
                .bind_input_shape(key, value.shape())
                .expect("a verified program declares each input exactly once");
        }
    }
    binder.build()
}
