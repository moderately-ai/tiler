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
//! capability's provider, governed key, and output-affecting revision, the
//! applicability guard, every
//! accessible byte range and launch formula — all are read from
//! [`tiler_compiler::session`]. The payload's compilation subject is filled by
//! `tiler-build` fills the payload's compilation subject from the emission and
//! prepared toolchain, and its content digest is *derived by the artifact layer*
//! from those bytes rather than supplied.
//!
//! One value is spelled here rather than handed over, and the reason is that its
//! owning authority states it in a form that is not an expression. A binding's
//! accessible *offset* is `tiler_ir::program::ByteWindow::offset`, a constant on
//! the packaged program's own view, and `tiler_compiler::session::AbiEntry`
//! exposes arena positions only. So this module reads that constant from the
//! program it is packaging and mints the literal naming it. That is a
//! transcription of the plan and not a choice: `push_variant` re-derives the same
//! window and refuses a binding whose declared offset differs from it.
//!
//! The other thing this module computes is the transliteration of the compiler's
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
    AbiExprId, ArtifactBuildError, ArtifactProgramBuilder, ArtifactVerificationError,
    BackendEntryKey, BackendEntryRef, BindingKind, BindingSpec, CapabilityKey,
    CompilationEnvironment, EntrySpec, FeasibilityRuleSetKey, FeasibilityRuleSetRef, LaunchSpec,
    SelectedProvider, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
    VariantSpec, VerifiedArtifactProgram,
};
use tiler_build::CompiledMetalPayload;
#[cfg(test)]
use tiler_build::PreparedMetalPayload;
use tiler_compiler::session::{Compilation, PlanAlternative};
use tiler_ir::program::abi::ExprNode;
use tiler_ir::semantic::SemanticProgram;

use std::fmt;

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
    payload: CompiledMetalPayload,
) -> Result<VerifiedArtifactProgram, BundleError> {
    let roots = variant_roots(plan.abi());
    assemble_from(semantic, compilation, plan, payload, &roots)
}

/// Packages the descriptor-only form of one plan before its payload is compiled.
///
/// The resulting identity is the cache subject. It is assembled through the
/// same path as [`assemble`], differing only in whether the payload declaration
/// carries its object.
///
/// # Errors
///
/// Returns [`BundleError`] naming the artifact boundary that refused.
#[cfg(test)]
pub fn assemble_pending(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
    payload: &PreparedMetalPayload<'_>,
) -> Result<VerifiedArtifactProgram, BundleError> {
    let roots = variant_roots(plan.abi());
    let mapped: Vec<BackendEntryKey> = payload
        .metadata()
        .entries
        .iter()
        .map(|mapping| mapping.entry_key.clone())
        .collect();
    assemble_declared(
        semantic,
        compilation,
        plan,
        &mapped,
        |builder, profile| Ok(payload.push_pending(builder, profile)?),
        &roots,
    )
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
    payload: CompiledMetalPayload,
    replay_roots: &[u32],
) -> Result<VerifiedArtifactProgram, BundleError> {
    let mapped: Vec<BackendEntryKey> = payload
        .content()
        .metadata
        .entries
        .iter()
        .map(|mapping| mapping.entry_key.clone())
        .collect();
    assemble_declared(
        semantic,
        compilation,
        plan,
        &mapped,
        |builder, profile| Ok(payload.push_carried(builder, profile)?),
        replay_roots,
    )
}

fn assemble_declared(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
    mapped: &[BackendEntryKey],
    declare_payload: impl FnOnce(
        &mut ArtifactProgramBuilder,
        TargetProfileRef,
    ) -> Result<tiler_artifact::program::PayloadId, BundleError>,
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
            capability_revision: selected.capability_revision(),
        })?;
    }

    // The payload's own compatibility contract. These bytes were compiled for
    // this profile, which is the same one the variant was assessed against; a
    // payload shared across profiles would state a different one.
    let payload_id = declare_payload(&mut builder, profile.clone())?;

    let abi = plan.abi();
    let arena = abi.expressions();
    let program = abi.kernel_program();
    // Replayed even though the variant no longer names these: the root set is a
    // parameter so the pruning obligation can be exercised as a property, and
    // the builder deduplicates by content, so this resolves to the same nodes it
    // adopts from the program rather than adding any.
    let minted = replay(&mut builder, arena, replay_roots)?;
    debug_assert!(
        minted.iter().any(Option::is_some),
        "a non-empty root set must replay at least one node"
    );

    let mut entries = Vec::with_capacity(abi.entries().len());
    for (entry, stage) in abi.entries().zip(program.stages()) {
        let entry_key =
            BackendEntryKey::from_bytes(stage.kernel().canonical_identity().as_bytes())?;
        if !mapped.contains(&entry_key) {
            return Err(BundleError::UnmappedEntry);
        }
        // The accessible range, launch geometry, and applicability guard are no
        // longer restated here: `ArtifactProgramBuilder` derives them from the
        // program it is given. This producer used to transliterate them by hand
        // and was the only one that did it correctly, which is exactly why the
        // artifact layer took the job over.
        let bindings = entry
            .accessible_bytes()
            .map(|_| BindingSpec {
                kind: BindingKind::Buffer,
            })
            .collect();
        entries.push(EntrySpec {
            bindings,
            launch: LaunchSpec {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{assemble, assemble_from, reachable_from, variant_roots};
    use crate::{COLUMNS, ROWS, emit_and_compile, serial_sum_program};
    use tiler_artifact::program::{
        SectionPurpose, SectionView, StageDependencyReason, decode_artifact,
    };
    use tiler_compiler::session::{
        Compilation, NumericalContract, PlanAlternative, compile_governed,
    };

    /// Governed feature key a carried-payload artifact must declare.
    const EMBEDDED_PAYLOAD_CODE: &str = "tiler.artifact.feature.embedded-payload-code";

    /// Compiles the proof program under the contract this target honours.
    fn compilations() -> Vec<Compilation> {
        compile_governed(
            &serial_sum_program(ROWS, COLUMNS),
            NumericalContract::FlushSubnormalsToZeroF32,
        )
        .expect("the governed program compiles")
    }

    /// Emits and offline-compiles the payload one plan alternative dispatches.
    fn payload_for(plan: PlanAlternative<'_>) -> tiler_build::CompiledMetalPayload {
        let kernels: Vec<_> = plan.kernels().iter().collect();
        let (_unit, payload) = emit_and_compile(&kernels);
        payload
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
        let expected_digest = payload
            .content()
            .identity()
            .expect("the subject has an identity");

        let artifact = assemble(
            &serial_sum_program(ROWS, COLUMNS),
            compilation,
            selected,
            payload,
        )
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

    /// The arena a variant is handed names every one of its own nodes, so the
    /// pruned and wholesale replays are the same artifact.
    ///
    /// # Two retractions, recorded rather than quietly fixed
    ///
    /// The prediction inherited from `carry-the-metal-payload-in-an-artifact-envelope`
    /// was that a wholesale replay fails whole-artifact verification with
    /// `ArtifactDiagnostic::UnusedExpression`, because the compiler's canonical
    /// graph served both alternatives and held the materialized plan's stage-0
    /// launch count, which the fused variant named nowhere. **Both halves are
    /// now false, and they stopped being true for different reasons.**
    ///
    /// The conclusion was already wrong before this ticket:
    /// `ArtifactProgramBuilder::push_node` deduplicates by content key, and that
    /// unreachable node was `UnsignedLiteral(input_elements)` — byte-for-byte
    /// the node the input binding's byte range already multiplied. Replaying it
    /// returned the handle already minted, so no unreferenced node ever entered
    /// the builder's arena.
    ///
    /// The premise stopped being true with
    /// `complete-program-identity-with-abi-guards-and-routing`. The arena is now
    /// the *program's* rather than one canonical graph shared by both
    /// alternatives, and `tiler_ir::program`'s verifier rejects an arena node no
    /// use site reaches — because program identity folds each use site by
    /// content key and would otherwise retain bytes it does not cover. So the
    /// unreachable set is now provably empty rather than merely harmless.
    ///
    /// [`assemble`] still prunes. The reachability walk is what makes the
    /// assembler independent of which arena it is handed, and this case pins
    /// that it is currently a no-op rather than assuming it always will be.
    #[test]
    fn the_pruned_and_wholesale_arena_replays_agree_because_a_program_names_its_whole_arena() {
        let compilations = compilations();
        let compilation = compilations.first().expect("one governed target");
        let selected = compilation.selected().expect("a selected alternative");
        let arena = selected.abi().expressions();

        let reached = reachable_from(arena, &variant_roots(selected.abi()))
            .expect("every root names a position in its own arena");
        assert!(
            reached.iter().all(|hit| *hit),
            "a verified kernel program names every node of its own ABI arena",
        );

        let all: Vec<u32> =
            (0..u32::try_from(arena.len()).expect("a bounded arena fits u32")).collect();
        let pruned = assemble(
            &serial_sum_program(ROWS, COLUMNS),
            compilation,
            selected,
            payload_for(selected),
        )
        .expect("the pruned replay assembles");
        let wholesale = assemble_from(
            &serial_sum_program(ROWS, COLUMNS),
            compilation,
            selected,
            payload_for(selected),
            &all,
        )
        .expect("a wholesale replay of a fully reachable arena assembles");
        assert_eq!(
            pruned.encode().expect("the envelope encodes"),
            wholesale.encode().expect("the envelope encodes"),
            "pruning a fully reachable arena leaves one artifact",
        );
    }

    /// A multi-stage variant round trips, and a decoder recovers its sequence.
    ///
    /// This inverts what it asserted until `carry-the-stage-execution-order-in-
    /// the-envelope`: the projector still derives
    /// `tiler.artifact.feature.multi-stage-program`, but the envelope now
    /// carries the execution order and the typed dependency edges that order
    /// discharges, so the reader sequences the variant instead of refusing it.
    ///
    /// The assertion is the *sequence*, not merely that it decoded. A decode
    /// that returned the entries in canonical stage-key order and called it an
    /// execution order would pass a test that only checked for `Ok`, and that is
    /// precisely the silent behaviour the old refusal existed to prevent.
    #[test]
    fn a_multi_stage_variant_round_trips_with_a_recoverable_sequence() {
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
            &serial_sum_program(ROWS, COLUMNS),
            compilation,
            materialized,
            payload_for(materialized),
        )
        .expect("a multi-stage bundle assembles and verifies");
        let bytes = artifact.encode().expect("the envelope encodes");
        let decoded =
            decode_artifact(&bytes).expect("this reader now sequences a multi-stage program");
        let variant = decoded.variants().next().expect("one packaged variant");

        let order: Vec<&[u8]> = variant
            .execution_order()
            .map(tiler_artifact::program::DecodedEntry::stage_key)
            .collect();
        assert_eq!(
            order.len(),
            materialized.kernels().len(),
            "the recovered order sequences every stage exactly once",
        );

        // The edges are what make the order checkable rather than asserted, so
        // the test reads them rather than trusting the order alone. A serial sum
        // materialized into two stages reduces what the first stage wrote, so
        // the obligation is a data dependency and not a storage handoff.
        let edges: Vec<_> = variant.stage_dependencies().collect();
        assert!(
            !edges.is_empty(),
            "a materialized plan whose second stage reads the first must carry an edge",
        );
        for edge in &edges {
            assert_eq!(
                edge.reason(),
                StageDependencyReason::Data,
                "this plan's stages are ordered by what they read, not by storage reuse",
            );
            let predecessor = order
                .iter()
                .position(|stage| *stage == edge.predecessor().stage_key())
                .expect("the edge names a sequenced entry");
            let successor = order
                .iter()
                .position(|stage| *stage == edge.successor().stage_key())
                .expect("the edge names a sequenced entry");
            assert!(
                predecessor < successor,
                "the recovered order discharges every edge it carries",
            );
        }
    }
}
