//! Carrying one compilation and its Metal payload in a neutral artifact envelope.
//!
//! This is the first artifact assembler in the workspace. It exists in the
//! producer because the producer is the only component that holds both halves at
//! once: `tiler-compiler` owns the plan and (ADR 0068) the ABI expressions, and
//! `tiler-artifact` owns the envelope, and neither may depend on the other.
//!
//! # Nothing here is derived a second time
//!
//! Every value that enters artifact identity is minted by the authority that
//! owns it and handed over whole. The target profile key and its exact
//! descriptor, the feasibility rule set key and revision, each selected
//! capability's provider and governed key, the applicability guard, every
//! accessible byte range and launch formula — all are read from
//! [`tiler_compiler::session`]. The payload's compilation subject is filled by
//! [`super::payload`] from the emission and the toolchain, and its content digest
//! is *derived by the artifact layer* from those bytes rather than supplied.
//!
//! The one thing this module computes is the transliteration of the compiler's
//! expression arena onto the builder's own arena, and that is mechanical: the
//! decision about what each expression *says* was made in the compiler.
//!
//! # Why the replay is pruned
//!
//! `tiler-artifact`'s whole-artifact verifier rejects an arena node that no use
//! site reaches (`ArtifactDiagnostic::UnusedExpression`), because an unreferenced
//! node changes an envelope's bytes without changing its identity. The compiler's
//! canonical graph serves *both* plan alternatives, so a variant's own use sites
//! reach a strict subset of it: the fused variant names nothing that resolves to
//! the materialized plan's stage-0 launch count. [`replay`] therefore walks only
//! the sub-DAG reachable from the variant's own roots.
//!
//! Pruning is not currently *observable* on this graph — the builder deduplicates
//! by content key and that unreachable node repeats content a reachable one
//! already carries, so replaying it mints nothing new. The test module measures
//! exactly that and says so. Pruning stays because the assembler must not depend
//! on which kind of arena it is handed: a node with unique content and no use
//! site would fail verification, and nothing about the compiler's graph promises
//! that one never appears.
//!
//! One forward pass suffices. Operands always precede the node naming them in the
//! compiler's arena, and the reachable set is operand-closed by construction, so
//! every operand is already minted when its node is reached.

use tiler_artifact::program::{
    AbiExprId, ArtifactBuildError, ArtifactExecutionPolicy, ArtifactProgramBuilder,
    ArtifactVerificationError, BackendEntryKey, BackendEntryRef, BackendKey, BindingKind,
    BindingSpec, CapabilityKey, CompilationEnvironment, EntrySpec, FeasibilityRuleSetKey,
    FeasibilityRuleSetRef, LaunchSpec, PayloadContent, RepresentationKey, SchemaVersion,
    SelectedProvider, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
    VariantSpec, VerifiedArtifactProgram,
};
use tiler_compiler::session::{Compilation, PlanAlternative};
use tiler_ir::program::abi::ExprNode;
use tiler_ir::semantic::SemanticProgram;

use std::fmt;

/// Governed backend family key of the Apple Metal backend.
const BACKEND_KEY: &str = "tiler.metal";
/// Governed executable-representation key of a linked Metal library.
const REPRESENTATION_KEY: &str = "metallib";
/// Component schema version of the carried payload this producer writes.
const PAYLOAD_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0);

/// Packages one plan alternative and its compiled payload as an artifact.
///
/// The payload is consumed: `push_carried_payload` derives the descriptor's
/// content digest from the exact metadata bytes, so a carried payload cannot
/// claim a compilation subject other than the one it carries.
///
/// # Errors
///
/// Returns [`BundleError`] naming the boundary that refused. A rejection is
/// never worked around here: the artifact layer's checks exist so a
/// well-formed-looking forgery cannot pass, and a real payload should not need
/// an exception.
pub fn assemble(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
    payload: PayloadContent,
) -> Result<VerifiedArtifactProgram, BundleError> {
    let roots = variant_roots(plan.abi());
    assemble_from(semantic, compilation, plan, payload, &roots)
}

/// Packages one alternative, replaying the arena from a stated root set.
///
/// The root set is a parameter rather than a constant so the pruning obligation
/// can be exercised as a property instead of asserted in prose: passing every
/// arena position reproduces the naive wholesale replay, and the artifact layer
/// must reject it. `assemble` is the only production entry point and always
/// passes the variant's own roots.
fn assemble_from(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
    payload: PayloadContent,
    replay_roots: &[u32],
) -> Result<VerifiedArtifactProgram, BundleError> {
    let profile = target_profile(compilation)?;
    let rules = feasibility_rules(compilation)?;

    // The environment is the set of providers this compilation offered. The
    // builder proves each selection was really offered, so it is populated from
    // the same authority rather than from a wider guess.
    let environment = CompilationEnvironment::new(
        plan.selected_capabilities()
            .map(|selected| selected.provider().clone()),
    )?;
    let mut builder = ArtifactProgramBuilder::new(semantic, environment)?;
    for selected in plan.selected_capabilities() {
        builder.select_provider(SelectedProvider {
            provider: selected.provider().clone(),
            capability: CapabilityKey::new(selected.capability_key())?,
            capability_api_version: capability_version(selected.capability_revision())?,
        })?;
    }

    // Held before the payload moves into the builder: the artifact's executable
    // entries must name backend entry keys this payload actually maps, and
    // proving that needs the mappings after the content is gone.
    let mapped: Vec<BackendEntryKey> = payload
        .metadata
        .entries
        .iter()
        .map(|mapping| mapping.entry_key.clone())
        .collect();
    let payload_id = builder.push_carried_payload(
        BackendKey::new(BACKEND_KEY)?,
        RepresentationKey::new(REPRESENTATION_KEY)?,
        PAYLOAD_SCHEMA,
        // The payload's own compatibility contract. These bytes were compiled
        // for this profile, which is the same one the variant was assessed
        // against; a payload shared across profiles would state a different one.
        profile.clone(),
        // A `metallib` is loaded directly by `newLibraryWithData:` — see the
        // runtime proof — so it is a native image rather than something a device
        // must translate first.
        ArtifactExecutionPolicy::NativeImage,
        payload,
    )?;

    let abi = plan.abi();
    let arena = abi.expressions();
    let program = abi.kernel_program();
    let minted = replay(&mut builder, arena, replay_roots)?;

    let mut entries = Vec::with_capacity(abi.entries().len());
    for (entry, stage) in abi.entries().zip(program.stages()) {
        let entry_key =
            BackendEntryKey::from_bytes(stage.kernel().canonical_identity().as_bytes())?;
        if !mapped.contains(&entry_key) {
            return Err(BundleError::UnmappedEntry);
        }
        let mut bindings = Vec::with_capacity(entry.accessible_bytes().len());
        for accessible in entry.accessible_bytes() {
            bindings.push(BindingSpec {
                kind: BindingKind::Buffer,
                accessible_bytes: resolve(&minted, accessible)?,
            });
        }
        entries.push(EntrySpec {
            bindings,
            launch: LaunchSpec {
                grid_threads: resolve(&minted, entry.grid_threads())?,
                threads_per_workgroup: resolve(&minted, entry.threads_per_workgroup())?,
                // Not a producer choice: `tiler_ir::schedule`'s intrinsic
                // verifier refuses a scheduled region whose launch plan does not
                // skip a zero-thread dispatch, so every verified region this
                // plan packages already carries it.
                zero_work_skips_dispatch: true,
                // The bounded profile defers nothing to launch time; its whole
                // preflight graph is literals resolved at `CompileProfile`.
                preconditions: Vec::new(),
            },
            implementation: BackendEntryRef {
                payload: payload_id,
                entry_key,
            },
        });
    }

    builder.push_variant(
        program,
        VariantSpec {
            applicability_guard: resolve(&minted, abi.applicability_guard())?,
            target_profile: profile,
            feasibility_rules: rules,
            deferred_predicates: Vec::new(),
            entries,
        },
    )?;
    Ok(builder.build()?)
}

/// Returns the arena positions one variant names directly.
///
/// These are exactly the roots the artifact layer's reachability check walks
/// from, so pruning to their closure is what makes the replayed arena equal the
/// set the verifier requires.
fn variant_roots(abi: tiler_compiler::session::AbiConstruction<'_>) -> Vec<u32> {
    let mut roots = vec![abi.applicability_guard()];
    for entry in abi.entries() {
        roots.extend(entry.accessible_bytes());
        roots.push(entry.grid_threads());
        roots.push(entry.threads_per_workgroup());
    }
    roots
}

/// Transliterates the reachable sub-DAG of one arena onto the builder's arena.
///
/// Returns one slot per source position, `Some` exactly for the positions
/// replayed. The builder deduplicates by content key, so two source positions
/// holding the same expression resolve to one handle — which is why the result
/// is a position map rather than a compacted list.
fn replay(
    builder: &mut ArtifactProgramBuilder,
    arena: &[ExprNode],
    roots: &[u32],
) -> Result<Vec<Option<AbiExprId>>, BundleError> {
    let reachable = reachable_from(arena, roots)?;
    let mut minted: Vec<Option<AbiExprId>> = vec![None; arena.len()];
    for (position, node) in arena.iter().enumerate() {
        if !reachable[position] {
            continue;
        }
        let id = match node {
            ExprNode::Root(root) => builder.push_root(root.clone())?,
            ExprNode::Unary { op, operand } => {
                builder.push_unary(*op, resolve(&minted, *operand)?)?
            }
            ExprNode::Binary { op, left, right } => {
                builder.push_binary(*op, resolve(&minted, *left)?, resolve(&minted, *right)?)?
            }
            ExprNode::Select {
                condition,
                if_true,
                if_false,
            } => builder.push_select(
                resolve(&minted, *condition)?,
                resolve(&minted, *if_true)?,
                resolve(&minted, *if_false)?,
            )?,
        };
        minted[position] = Some(id);
    }
    Ok(minted)
}

/// Marks every arena position reachable from a set of use sites.
fn reachable_from(arena: &[ExprNode], roots: &[u32]) -> Result<Vec<bool>, BundleError> {
    let mut reached = vec![false; arena.len()];
    let mut work: Vec<u32> = roots.to_vec();
    while let Some(node) = work.pop() {
        let at = position_of(node);
        if at >= arena.len() {
            return Err(BundleError::ExpressionOutOfRange { position: node });
        }
        if reached[at] {
            continue;
        }
        reached[at] = true;
        match &arena[at] {
            ExprNode::Root(_) => {}
            ExprNode::Unary { operand, .. } => work.push(*operand),
            ExprNode::Binary { left, right, .. } => {
                work.push(*left);
                work.push(*right);
            }
            ExprNode::Select {
                condition,
                if_true,
                if_false,
            } => {
                work.push(*condition);
                work.push(*if_true);
                work.push(*if_false);
            }
        }
    }
    Ok(reached)
}

/// Resolves one source arena position to the handle it was replayed onto.
///
/// A miss is reported rather than assumed away. It would mean the arena's
/// operands-precede-their-node invariant does not hold or the reachable set is
/// not operand-closed, and either is a defect worth naming at the position that
/// exposed it.
fn resolve(minted: &[Option<AbiExprId>], position: u32) -> Result<AbiExprId, BundleError> {
    minted
        .get(position_of(position))
        .copied()
        .flatten()
        .ok_or(BundleError::UnmintedExpression { position })
}

fn position_of(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

/// Builds the declared target profile both halves of which the compiler minted.
fn target_profile(compilation: &Compilation) -> Result<TargetProfileRef, BundleError> {
    Ok(TargetProfileRef {
        key: TargetProfileKey::new(compilation.target_profile_key())?,
        // The descriptor bytes *are* the identity, not a hash of one, so they
        // are wrapped rather than digested here. Digesting them would mint a
        // second identity this producer is not the authority for.
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )?,
    })
}

/// Builds the feasibility rule set reference the compiler minted.
fn feasibility_rules(compilation: &Compilation) -> Result<FeasibilityRuleSetRef, BundleError> {
    Ok(FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new(compilation.feasibility_rule_set_key())?,
        revision: compilation.feasibility_rule_set_revision(),
    })
}

/// Narrows the compiler's capability revision into the artifact's version slot.
///
/// **This carries a real value into an adjacent slot, and the mismatch is
/// recorded rather than hidden.** `SelectedProvider::capability_api_version` is
/// documented as "version of the capability API the selection was made against",
/// and the compiler has no such notion — its `capability_revision` is the
/// capability's *output-affecting* revision. `docs/operation-extensions.md` says
/// a selected plan records the `{provider identity, capability revision}` pair,
/// which is the value carried here; it also says compiler and capability-API
/// versions participate in identity, and no producer can supply the latter
/// today. `record-the-capability-revision-in-selected-provider-identity` owns
/// closing that.
///
/// The narrowing is checked rather than truncating. A revision beyond `u16` is
/// refused, because silently keeping the low half would put a wrong revision
/// into artifact identity — worse than refusing, since it would look correct.
fn capability_version(revision: u32) -> Result<u16, BundleError> {
    u16::try_from(revision).map_err(|_| BundleError::CapabilityRevisionWidth { revision })
}

/// Why a compilation and its payload did not package into an artifact.
#[derive(Debug)]
pub enum BundleError {
    /// The artifact layer refused a declaration at insertion.
    Build(ArtifactBuildError),
    /// Whole-artifact verification refused the assembled draft.
    Verify(ArtifactVerificationError),
    /// An entry names a backend entry key the carried payload does not map.
    UnmappedEntry,
    /// A use site named an arena position outside the arena.
    ExpressionOutOfRange {
        /// The position that was named.
        position: u32,
    },
    /// A node's operand was not replayed before the node naming it.
    UnmintedExpression {
        /// The source arena position that had no handle.
        position: u32,
    },
    /// A capability revision does not fit the artifact's version field.
    CapabilityRevisionWidth {
        /// The revision the compiler minted.
        revision: u32,
    },
}

impl From<ArtifactBuildError> for BundleError {
    fn from(value: ArtifactBuildError) -> Self {
        Self::Build(value)
    }
}

impl From<ArtifactVerificationError> for BundleError {
    fn from(value: ArtifactVerificationError) -> Self {
        Self::Verify(value)
    }
}

impl fmt::Display for BundleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(cause) => write!(formatter, "the artifact layer refused: {cause}"),
            Self::Verify(cause) => write!(
                formatter,
                "whole-artifact verification refused: {:?}",
                cause.diagnostics(),
            ),
            Self::UnmappedEntry => formatter.write_str(
                "an executable entry names a backend entry the carried payload does not map",
            ),
            Self::ExpressionOutOfRange { position } => write!(
                formatter,
                "a use site names arena position {position}, which the arena does not hold",
            ),
            Self::UnmintedExpression { position } => write!(
                formatter,
                "arena position {position} was named before it was replayed",
            ),
            Self::CapabilityRevisionWidth { revision } => write!(
                formatter,
                "capability revision {revision} does not fit the artifact's version field",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{assemble, assemble_from, reachable_from, variant_roots};
    use crate::{emit_and_compile, payload, serial_sum_program};
    use tiler_artifact::program::{
        ArtifactCodecFailure, PayloadContent, SectionPurpose, SectionView, decode_artifact,
    };
    use tiler_compiler::session::{
        Compilation, NumericalContract, PlanAlternative, compile_governed,
    };

    /// Governed feature key a carried-payload artifact must declare.
    const EMBEDDED_PAYLOAD_CODE: &str = "tiler.artifact.feature.embedded-payload-code";

    /// Compiles the proof program under the contract this target honours.
    fn compilations() -> Vec<Compilation> {
        compile_governed(
            &serial_sum_program(),
            NumericalContract::FlushSubnormalsToZeroF32,
        )
        .expect("the governed program compiles")
    }

    /// Emits and offline-compiles the payload one plan alternative dispatches.
    fn payload_for(plan: PlanAlternative<'_>) -> PayloadContent {
        let kernels: Vec<_> = plan.kernels().iter().collect();
        let (unit, compiled) = emit_and_compile(&kernels);
        payload::carried_payload(&unit, &compiled.provenance, &compiled.metallib)
            .expect("the payload assembles")
    }

    /// A real compilation and a real `metallib` survive the envelope round trip.
    ///
    /// Every assertion here is about a bundle built from an actual compilation
    /// and an actual `xcrun` link, not from a fixture. The codec's own cases
    /// already pin these properties against synthetic content; this is where
    /// they meet a real one, and nothing was relaxed to make it fit.
    #[test]
    fn a_real_compilation_round_trips_through_the_neutral_envelope() {
        let compilations = compilations();
        let compilation = compilations.first().expect("one governed target");
        let selected = compilation.selected().expect("a selected alternative");
        let payload = payload_for(selected);
        let expected_digest = payload.identity().expect("the subject has an identity");

        let artifact = assemble(&serial_sum_program(), compilation, selected, payload)
            .expect("the real bundle assembles");
        let bytes = artifact.encode().expect("the envelope encodes");

        // A successful decode *is* the identity proof: `decode_artifact`
        // re-derives the identity from decoded content and refuses when it does
        // not equal the one the manifest carries.
        let decoded = decode_artifact(&bytes).expect("the envelope decodes");
        assert_eq!(
            decoded.re_encode().expect("the envelope re-encodes"),
            bytes,
            "a field the decoder dropped could not be written back",
        );
        assert!(!decoded.identity().as_bytes().is_empty());
        assert_eq!(decoded.variant_count(), 1);

        // The two payload sections, by purpose rather than by position.
        let purposes: Vec<_> = decoded.sections().map(SectionView::purpose).collect();
        assert!(
            purposes.contains(&SectionPurpose::BackendPayloadMetadata),
            "the compilation subject travels as its own section: {purposes:?}",
        );
        assert!(
            purposes.contains(&SectionPurpose::BackendPayloadCode),
            "the emitted object travels as its own section: {purposes:?}",
        );

        // The descriptor's digest is the payload identity of the metadata bytes.
        // The decode above already re-derived it from the metadata *section* and
        // compared; this pins the same value against the content the producer
        // handed over, so the two cannot agree by both being wrong.
        let [descriptor] = decoded.payloads() else {
            panic!("one carried payload");
        };
        assert_eq!(descriptor.digest, expected_digest);
        assert_eq!(descriptor.backend.as_str(), "tiler.metal");
        assert_eq!(descriptor.representation.as_str(), "metallib");

        // The carried object is what makes the feature required, and the feature
        // set is derived from content rather than declared by this producer.
        assert!(
            decoded
                .features()
                .iter()
                .any(|feature| feature == EMBEDDED_PAYLOAD_CODE),
            "an artifact carrying object bytes says so: {:?}",
            decoded.features(),
        );

        // The payload's compatibility contract is the profile the plan was
        // assessed against, both halves minted by the compiler.
        assert_eq!(
            descriptor.compatibility.key.as_str(),
            compilation.target_profile_key(),
        );
        assert_eq!(
            descriptor.compatibility.descriptor.as_bytes(),
            compilation.target_profile_descriptor(),
        );
    }

    /// The arena the fused variant names is a strict subset, and replaying the
    /// whole of it is nonetheless safe *today* — for a reason worth pinning.
    ///
    /// # A retraction, recorded rather than quietly fixed
    ///
    /// The prediction inherited from `carry-the-metal-payload-in-an-artifact-envelope`
    /// was that a wholesale replay fails whole-artifact verification with
    /// `ArtifactDiagnostic::UnusedExpression`, because the compiler's canonical
    /// graph serves both alternatives and holds the materialized plan's stage-0
    /// launch count, which the fused variant names nowhere. **The first half is
    /// true and the conclusion is false.** `ArtifactProgramBuilder::push_node`
    /// deduplicates by content key, and that unreachable node is
    /// `UnsignedLiteral(input_elements)` — byte-for-byte the node the input
    /// binding's byte range already multiplies. Replaying it returns the handle
    /// already minted, so no unreferenced node ever enters the builder's arena.
    ///
    /// This case therefore asserts what is measurable rather than what was
    /// predicted: the unreachable set is non-empty, every node in it duplicates
    /// the content of a reachable one, and the two replays encode to the same
    /// bytes. [`assemble`] still prunes, because the safety here is a property of
    /// *this* nine-node graph rather than of the discipline — an arena holding a
    /// node with unique content and no use site would fail, and pruning is what
    /// makes the assembler independent of which of those it is handed.
    #[test]
    fn the_pruned_and_wholesale_arena_replays_agree_because_the_builder_dedupes() {
        let compilations = compilations();
        let compilation = compilations.first().expect("one governed target");
        let selected = compilation.selected().expect("a selected alternative");
        let arena = selected.abi().expressions();

        let reached = reachable_from(arena, &variant_roots(selected.abi()))
            .expect("every root names a position in its own arena");
        let unreachable: Vec<usize> = reached
            .iter()
            .enumerate()
            .filter_map(|(position, hit)| (!*hit).then_some(position))
            .collect();
        assert!(
            !unreachable.is_empty(),
            "the shared canonical graph holds a node this variant never names",
        );
        for position in &unreachable {
            assert!(
                arena
                    .iter()
                    .enumerate()
                    .any(|(other, node)| reached[other] && *node == arena[*position]),
                "arena position {position} is unreachable and its content is unique, \
                 so a wholesale replay would leave it unreferenced",
            );
        }

        let all: Vec<u32> =
            (0..u32::try_from(arena.len()).expect("a bounded arena fits u32")).collect();
        let pruned = assemble(
            &serial_sum_program(),
            compilation,
            selected,
            payload_for(selected),
        )
        .expect("the pruned replay assembles");
        let wholesale = assemble_from(
            &serial_sum_program(),
            compilation,
            selected,
            payload_for(selected),
            &all,
        )
        .expect("content deduplication absorbs the unreachable node");
        assert_eq!(
            pruned.encode().expect("the envelope encodes"),
            wholesale.encode().expect("the envelope encodes"),
            "deduplication makes the two replays one artifact",
        );
    }

    /// A multi-stage variant encodes and is then refused by this reader.
    ///
    /// Not a defect in the assembly and not a codec check to weaken. This
    /// profile's neutral program section carries a program's canonical identity
    /// and not its dependency graph, so a reader cannot recover the order two
    /// stages must run in. The projector therefore derives
    /// `tiler.artifact.feature.multi-stage-program`, which is deliberately
    /// absent from `SUPPORTED_FEATURES`, and the decoder refuses rather than
    /// treating declaration order as execution order.
    ///
    /// `carry-reconstructable-kernel-programs-in-the-neutral-envelope` owns
    /// closing it. Until then this is the exact measured boundary of what the
    /// envelope can carry, and asserting it keeps the limit from being
    /// rediscovered as a bug.
    #[test]
    fn a_multi_stage_variant_encodes_and_this_reader_refuses_it() {
        let compilations = compilations();
        let compilation = compilations.first().expect("one governed target");
        let materialized = compilation
            .alternatives()
            .find(|plan| !plan.is_fused())
            .expect("the materialized reference alternative is retained");
        assert!(
            materialized.kernels().len() > 1,
            "the materialized plan dispatches more than one stage",
        );

        let artifact = assemble(
            &serial_sum_program(),
            compilation,
            materialized,
            payload_for(materialized),
        )
        .expect("a multi-stage bundle assembles and verifies");
        let bytes = artifact.encode().expect("the envelope encodes");
        let refusal =
            decode_artifact(&bytes).expect_err("this reader refuses a multi-stage program");
        assert!(
            matches!(refusal, ArtifactCodecFailure::Unsupported { .. }),
            "a feature this build cannot supply is unsupported, not corruption: {refusal}",
        );
    }
}
