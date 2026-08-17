//! Whole-artifact verification.
//!
//! Everything locally decidable is already rejected at insertion. What remains
//! is the set of obligations that only the assembled artifact can discharge:
//! the portfolio is non-empty, the plan attributes itself to some reached
//! authority, the expression arena and the payload table are exactly the
//! reachable sets, no two realizations claim the same backend entry, no payload
//! is reached from two delivery positions, and every canonical key an identity
//! cross-reference depends on is unambiguous.

use std::collections::{BTreeMap, BTreeSet};

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
    if let Some(payload) = payload_at_two_delivery_positions(data) {
        diagnostics.push(ArtifactDiagnostic::AmbiguousPayloadDelivery { payload });
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
/// Identity writes the arena once in a traversal seeded by declared use sites
/// and names every use by canonical position. Its pre-validation encoder can
/// append an unreached position only so a malformed draft receives this typed
/// rejection rather than panicking during identity derivation. Rejecting keeps
/// a verified artifact's arena exactly equal to its use-site-reached arena.
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
///
/// A payload is reached through one *realization* — an entry paired with a
/// delivery position — rather than through an entry, so a two-position artifact
/// keeps every one of its objects referenced without needing a second entry for
/// each. The obligation is unchanged: an unreferenced payload would change the
/// envelope's bytes without changing the artifact's identity, giving one
/// artifact two byte identities.
fn payloads_are_referenced(data: &ArtifactProgramData) -> bool {
    let referenced: BTreeSet<u32> = data
        .variants
        .iter()
        .flat_map(|variant| &variant.entries)
        .flat_map(|entry| &entry.implementation.payloads)
        .copied()
        .collect();
    (0..data.payloads.len())
        .map(|payload| u32::try_from(payload).expect("a bounded payload table fits u32"))
        .all(|payload| referenced.contains(&payload))
}

/// Returns whether two realizations claim the same backend entry of one payload.
///
/// Two neutral realizations mapping to one backend entry would make the backend
/// mapping non-injective, which the payload validator could not later repair.
///
/// It also decides one case delivery positions introduced: an entry naming the
/// same payload at two positions repeats `(payload, entry_key)` and is refused
/// here, so a consumer resolving either position cannot be handed one object
/// standing in for two build targets.
fn backend_entries_collide(data: &ArtifactProgramData) -> bool {
    let mut claimed: BTreeSet<(u32, &[u8])> = BTreeSet::new();
    data.variants
        .iter()
        .flat_map(|variant| &variant.entries)
        .flat_map(|entry| {
            entry
                .implementation
                .payloads
                .iter()
                .map(|payload| (*payload, entry.implementation.entry_key.as_bytes()))
        })
        .any(|realization| !claimed.insert(realization))
}

/// Returns a payload reached from two different delivery positions, if any.
///
/// [`backend_entries_collide`] already refuses one *entry* naming a payload
/// twice; this refuses the cross-entry case it cannot see — position 0 of one
/// entry and position 1 of another naming one object. That artifact declares
/// more consumer build targets than it carries objects for, and the neutral
/// layer cannot decide which target the shared object was built for, so it
/// refuses the shape rather than guessing.
fn payload_at_two_delivery_positions(data: &ArtifactProgramData) -> Option<u32> {
    let mut seen: BTreeMap<u32, usize> = BTreeMap::new();
    for entry in data.variants.iter().flat_map(|variant| &variant.entries) {
        for (delivery, payload) in entry.implementation.payloads.iter().enumerate() {
            if *seen.entry(*payload).or_insert(delivery) != delivery {
                return Some(*payload);
            }
        }
    }
    None
}

/// Returns whether one variant's program holds two stages with equal keys.
///
/// Artifact identity cross-references an entry's stage by content key, so equal
/// keys would make the entry-to-stage mapping unrecoverable.
fn stage_keys_collide(data: &ArtifactProgramData) -> bool {
    data.variants.iter().any(|variant| {
        let mut keys: Vec<Vec<u8>> = variant
            .program
            .stages()
            .map(|stage| stage_key(&variant.program, stage))
            .collect();
        keys.sort_unstable();
        keys.windows(2).any(|pair| pair[0] == pair[1])
    })
}
