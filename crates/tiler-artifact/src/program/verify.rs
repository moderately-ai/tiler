//! Whole-artifact verification.
//!
//! Everything locally decidable is already rejected at insertion. What remains
//! is the set of obligations that only the assembled artifact can discharge:
//! the portfolio is non-empty, the plan attributes itself to some reached
//! authority, the expression arena and the payload table are exactly the
//! reachable sets, no two entries claim the same backend entry, and every
//! canonical key an identity cross-reference depends on is unambiguous.

use std::collections::BTreeSet;

use super::error::{ArtifactDiagnostic, ArtifactEntityKind};
use super::expr::ExprNode;
use super::model::{ArtifactProgramData, stage_key};

fn position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

/// Proves every whole-artifact obligation, returning all failures in stable order.
pub(super) fn verify_artifact(data: &ArtifactProgramData) -> Vec<ArtifactDiagnostic> {
    let mut diagnostics = Vec::new();
    if data.variants.is_empty() {
        diagnostics.push(ArtifactDiagnostic::EmptyPortfolio);
    }
    if data.providers.is_empty() {
        diagnostics.push(ArtifactDiagnostic::MissingSelectedProvider);
    }
    if !expressions_are_reachable(data) {
        diagnostics.push(ArtifactDiagnostic::UnusedExpression);
    }
    if !payloads_are_referenced(data) {
        diagnostics.push(ArtifactDiagnostic::UnusedPayload);
    }
    if backend_entries_collide(data) {
        diagnostics.push(ArtifactDiagnostic::DuplicateBackendEntry);
    }
    if stage_keys_collide(data) {
        diagnostics.push(ArtifactDiagnostic::AmbiguousCanonicalKey {
            entity: ArtifactEntityKind::Entry,
        });
    }
    diagnostics
}

/// Returns whether every arena node is reachable from a declared use site.
///
/// Identity encodes expressions by content key at their use sites, so an
/// unreferenced node does not change it — which is exactly the hazard. The node
/// would still be retained by the verified product and written by a codec,
/// making two byte-different artifacts share one identity. Rejecting keeps the
/// arena a function of what the artifact actually says.
fn expressions_are_reachable(data: &ArtifactProgramData) -> bool {
    let mut reached = vec![false; data.expressions.len()];
    let mut work: Vec<u32> = Vec::new();
    for variant in &data.variants {
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
        match &data.expressions[position(node)] {
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
    reached.iter().all(|node| *node)
}

/// Returns whether every declared payload descriptor realizes some entry.
fn payloads_are_referenced(data: &ArtifactProgramData) -> bool {
    let referenced: BTreeSet<u32> = data
        .variants
        .iter()
        .flat_map(|variant| &variant.entries)
        .map(|entry| entry.implementation.payload)
        .collect();
    (0..data.payloads.len())
        .map(|payload| u32::try_from(payload).expect("a bounded payload table fits u32"))
        .all(|payload| referenced.contains(&payload))
}

/// Returns whether two executable entries claim the same backend entry.
///
/// Two neutral entries mapping to one backend entry would make the backend
/// mapping non-injective, which the payload validator could not later repair.
fn backend_entries_collide(data: &ArtifactProgramData) -> bool {
    let mut claimed: BTreeSet<(u32, &[u8])> = BTreeSet::new();
    data.variants
        .iter()
        .flat_map(|variant| &variant.entries)
        .any(|entry| {
            !claimed.insert((
                entry.implementation.payload,
                entry.implementation.entry_key.as_bytes(),
            ))
        })
}

/// Returns whether one variant's program holds two stages with equal keys.
///
/// Artifact identity cross-references an entry's stage by content key, so equal
/// keys would make the entry-to-stage mapping unrecoverable.
fn stage_keys_collide(data: &ArtifactProgramData) -> bool {
    data.variants.iter().any(|variant| {
        let mut keys: Vec<Vec<u8>> = variant.program.stages().map(stage_key).collect();
        keys.sort_unstable();
        keys.windows(2).any(|pair| pair[0] == pair[1])
    })
}
