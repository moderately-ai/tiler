//! The complete producer declaration's canonical byte grammar.
//!
//! Identity-bearing. The domain separators below and [`complete_descriptor`]
//! are what an artifact identity and every cache subject derived from a profile
//! are folded from, so a change here moves bytes that other producers have
//! already minted against. The per-row encodings live with their families in
//! [`super::rows`]; this module owns only the order, the separators, the shared
//! source table, and the derivation that licenses each conditional section.

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::program::abi::TargetPropertyQuery;

use crate::target::accuracy::ElementaryRealization;
use crate::target::feasibility::{DeclaredSubgroupRealization, DeclaredSynchronizationRealization};
use crate::target::honourability::SourceTable;
use crate::target::key::TargetProfileKey;
use crate::target::rows::{
    CostRowFact, DTypeDispatchabilityFact, EvaluationOrderFact, QuantitativeCapabilityDeclaration,
    QuantitativeCapabilityQueryDeclaration, ScalarHonourabilityDeclaration,
    WorkgroupTreeWidthPolicyFact,
};

/// Domain of the complete producer declaration carried into artifact identity.
///
/// This is a new grammar, not a continuation of feasibility's checked
/// descriptor: that one remains an internal feasibility component, while this
/// declaration encodes the same capability and numerical semantics plus exact
/// dtype dispatch and synchronization realization through one shared provenance
/// table. A reader of an older domain therefore cannot mistake these bytes for
/// the new grammar.
///
/// `v11` appends the synchronization-realization rows. Every profile's bytes
/// move, including a profile that declares none: the row family writes its
/// domain separator and a count, so "this target says nothing about
/// synchronization" becomes a recorded fact rather than an absence recoverable
/// from bytes that never stated it. That is the point of the step — a `v10`
/// declaration could not distinguish a target that had been asked from one that
/// had not, and those admit different candidates.
pub(super) const COMPLETE_PROFILE_DESCRIPTOR_DOMAIN: &[u8] =
    b"tiler.target-profile.declaration.v11\0";
const PROFILE_SOURCE_DOMAIN: &[u8] = b"tiler.target-profile.fact-sources.v4\0";
const DISPATCHABILITY_DOMAIN: &[u8] = b"tiler.target-profile.dtype-dispatchability.v2\0";
/// Domain separating the synchronization-realization rows of one declaration.
///
/// A separator of its own, exactly as the dispatchability rows have one: the two
/// grammars are independent, so a reader must not be able to consume one row
/// family's bytes at the other's offset.
const SYNCHRONIZATION_DOMAIN: &[u8] = b"tiler.target-profile.synchronization-realization.v1\0";
/// Domain separating the evaluation-order-preservation rows of one declaration.
///
/// Its own separator, for the reason the two families above have one: the
/// grammars are independent, so no reader may consume one family's bytes at
/// another's offset. Unlike the synchronization separator this one is written
/// **only when the family is non-empty**, and
/// [`complete_descriptor`] states the derivation that licenses the difference.
pub(super) const EVALUATION_ORDER_DOMAIN: &[u8] =
    b"tiler.target-profile.evaluation-order-preservation.v1\0";
/// Domain separating the measured cost rows of one declaration.
///
/// Its own separator, for the reason the three families above have one, and
/// written **only when the family is non-empty** for the reason the
/// evaluation-order family is: silence about a cost row means *no preference*,
/// which is what a profile that never carried the family already recorded, so
/// writing a zero count would move every existing profile's bytes to record
/// nothing new. [`complete_descriptor`] states the derivation.
const COST_ROW_DOMAIN: &[u8] = b"tiler.target-profile.cost-row.v1\0";
/// Domain separating the elementary-realization rows of one declaration.
///
/// Its own separator, for the reason the families above have one, and written
/// **only when the family is non-empty** for the reason the evaluation-order
/// and cost-row families are: an empty family and an absent family both mean
/// no installed realization, which is what every profile encoded before this
/// family existed. Writing a zero count would move every existing profile's
/// bytes to record nothing new. [`complete_descriptor`] states the derivation.
pub(super) const ELEMENTARY_REALIZATION_DOMAIN: &[u8] =
    b"tiler.target-profile.elementary-realization.v1\0";
/// Domain separating the workgroup-tree-width-policy rows of one declaration.
///
/// Its own separator, for the reason the families above have one, and written
/// **only when the family is non-empty** so a profile that never carried the
/// family keeps the bytes it already encoded. Silence here is not a preference:
/// it makes the single-workgroup tree unavailable. Writing a zero count would
/// move every existing profile's identity to record that it still has no
/// policy. [`complete_descriptor`] states the derivation.
pub(super) const WORKGROUP_TREE_WIDTH_POLICY_DOMAIN: &[u8] =
    b"tiler.target-profile.workgroup-tree-width-policy.v1\0";
/// Domain separating the subgroup-realization rows of one declaration.
///
/// Its own separator, for the reason the families above have one, and written
/// **only when the family is non-empty** so a profile that never carried the
/// family keeps the bytes it already encoded. Silence here is `Unknown` for
/// every subject: it is not a default width and not a neighbouring realization.
/// Writing a zero count would move every existing profile's identity to record
/// that it still has no subgroup row. [`complete_descriptor`] states the
/// derivation.
pub(super) const SUBGROUP_REALIZATION_DOMAIN: &[u8] =
    b"tiler.target-profile.subgroup-realization.v1\0";
/// Domain separating one declaration's prepared subgroup-width query.
///
/// Its own separator, written **only when the query exists**. Presence is
/// equivalent, by the construction contract, to the subgroup-realization
/// family carrying a `Realized` row, so a profile without the query keeps the
/// bytes it already encoded — including every profile minted before this
/// family existed, and the previously constructible `Realized`-without-query
/// population, which is now refused at construction rather than re-encoded.
/// [`complete_descriptor`] states the derivation.
pub(super) const SUBGROUP_WIDTH_QUERY_DOMAIN: &[u8] =
    b"tiler.target-profile.subgroup-width-query.v1\0";

pub(super) fn encode_compact_index(bytes: &mut Vec<u8>, mut value: usize) {
    loop {
        let low = u8::try_from(value & 0x7f).expect("seven masked bits fit in u8");
        value >>= 7;
        if value == 0 {
            bytes.push(low);
            break;
        }
        bytes.push(low | 0x80);
    }
}

/// Encodes one complete producer declaration.
///
/// # Why the evaluation-order family did not step [`COMPLETE_PROFILE_DESCRIPTOR_DOMAIN`]
///
/// The rule is a byte rule: the domain steps when previously-encodable bytes
/// **move**, because a reader of the older domain would then be reading the same
/// bytes under a different grammar. The evaluation-order family moves none. It
/// is written last, behind its own separator, and **only when it holds a row**,
/// so every profile assembled before it existed — the governed baseline, the
/// bound macOS Metal declaration, every test profile — encodes byte for byte
/// what it encoded at `v11`. Its sources join the shared source table through an
/// iterator that is empty for those profiles, so no source index shifts either.
///
/// Injectivity survives the conditional section because every earlier section is
/// self-delimiting: two descriptors agreeing on the `v11` prefix agree on every
/// earlier row, and the remainder is then either empty or this family's bytes,
/// which its separator distinguishes from any continuation. An empty family and
/// an absent family denote the same thing here — `Unknown` for every subject and
/// licence, which no admission path can act on differently — so nothing a
/// candidate's admission depends on is lost by not writing a zero count.
///
/// The synchronization family one section above frames itself *unconditionally*
/// and therefore had to step `v10` to `v11`. That was a choice about what
/// silence should record, not a rule this family breaks: `v11` decided that "no
/// synchronization was declared" should be a recorded fact rather than an
/// absence, and paid for the decision by moving every profile's bytes. This
/// family records silence as absence, and pays nothing.
///
/// # The cost-row family takes the same shape, and for a stronger reason
///
/// It is written last, behind its own separator, and only when it holds a row,
/// so it too moves no earlier byte. The reason it *must* is the silence rule the
/// activating ticket's acceptance made testable rather than aspirational: **a
/// profile declaring no cost row selects bit-identically to a build without the
/// family at all.** Selection reads the row, and a profile's canonical descriptor
/// is folded into every artifact identity and cache subject derived from it — so
/// an unconditional section would move every existing profile's identity to
/// record that it still has no preference. Injectivity survives for the reason it
/// survives above: every earlier section is self-delimiting, and this family's
/// separator distinguishes its bytes from any continuation of the last one.
///
/// # The elementary-realization family is the same silence rule, rederived
///
/// It is written last, behind its own separator, and only when it holds a row.
/// An empty family and an absent family both mean no installed realization,
/// which is already what every profile encoded before this family existed —
/// including the governed profile, which does not regain its three Metal rows
/// here. Writing a zero count would move every existing descriptor to record
/// that it still has no elementary row. Injectivity survives for the same
/// reason as the two families above: every earlier section is self-delimiting,
/// and this family's separator distinguishes its bytes from any continuation.
///
/// # The workgroup-tree-width-policy family is the same silence-as-absence
///
/// It is written last, behind its own separator, and only when it holds a row.
/// An empty family and an absent family both mean no accepted policy, which is
/// already what every profile encoded before this family existed. Writing a
/// zero count would move every existing descriptor to record that it still has
/// no policy. Injectivity survives for the same reason as the families above.
/// The owning declaration domain therefore stays at `v11`.
///
/// # The subgroup-realization family is the same silence-as-absence
///
/// It is written last, behind its own separator, and only when it holds a row.
/// An empty family and an absent family both mean `Unknown` for every subgroup
/// subject, which is already what every profile encoded before this family
/// existed — including every standard profile, which stays silent until its
/// own evidence ticket and prepared-entry gate complete. Writing a zero count
/// would move every existing descriptor to record that it still has no
/// subgroup row. Injectivity survives for the same reason as the families
/// above. The owning declaration domain therefore stays at `v11`.
///
/// # The prepared subgroup-width query is the same silence-as-absence
///
/// It is written last, behind its own separator, and only when a query was
/// declared. Presence is equivalent, by the builder's own validation, to the
/// subgroup family carrying a `Realized` row, so every profile carrying the
/// query bytes is a new value: the previously constructible
/// `Realized`-without-query population is refused at construction rather than
/// re-encoded, and every profile without a query keeps the exact bytes it
/// already encoded. Injectivity survives for the same reason as the families
/// above, and the owning declaration domain therefore stays at `v11`.
#[allow(
    clippy::too_many_arguments,
    reason = "one parameter per declared row family, threaded explicitly so the encoder reads as the grammar it writes; grouping them behind a struct would put the canonical byte order under two authorities"
)]
pub(super) fn complete_descriptor(
    key: &TargetProfileKey,
    quantitative: &[QuantitativeCapabilityDeclaration],
    queries: &[QuantitativeCapabilityQueryDeclaration],
    scalar: &[ScalarHonourabilityDeclaration],
    dispatchability: &[DTypeDispatchabilityFact],
    synchronization: &[DeclaredSynchronizationRealization],
    evaluation_order: &[EvaluationOrderFact],
    cost_rows: &[CostRowFact],
    tree_width_policies: &[WorkgroupTreeWidthPolicyFact],
    elementary: &[ElementaryRealization],
    subgroup: &[DeclaredSubgroupRealization],
    subgroup_query: Option<&TargetPropertyQuery>,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, COMPLETE_PROFILE_DESCRIPTOR_DOMAIN);
    push_slice(&mut bytes, key.as_str().as_bytes());
    push_slice(&mut bytes, PROFILE_SOURCE_DOMAIN);
    let sources = SourceTable::collect(
        quantitative
            .iter()
            .map(|fact| fact.source.as_ref())
            .chain(scalar.iter().map(|declaration| declaration.source.as_ref()))
            .chain(dispatchability.iter().map(|fact| fact.source.as_ref()))
            .chain(
                synchronization
                    .iter()
                    .map(DeclaredSynchronizationRealization::source_ref),
            )
            .chain(evaluation_order.iter().map(|fact| fact.source.as_ref()))
            .chain(cost_rows.iter().map(|fact| fact.source.as_ref()))
            .chain(tree_width_policies.iter().map(|fact| fact.source.as_ref()))
            .chain(elementary.iter().map(ElementaryRealization::source))
            .chain(subgroup.iter().map(DeclaredSubgroupRealization::source_ref)),
    );
    push_len(&mut bytes, sources.entries().len());
    for source_bytes in sources.entries() {
        push_slice(&mut bytes, source_bytes);
    }
    push_len(&mut bytes, quantitative.len());
    for fact in quantitative {
        push_slice(&mut bytes, fact.axis.key().as_bytes());
        bytes.extend_from_slice(&fact.bound.to_le_bytes());
        let source_index = sources.index_of(fact.source.as_ref());
        QuantitativeCapabilityDeclaration::encode_source_index(&mut bytes, source_index);
    }
    push_len(&mut bytes, queries.len());
    for query in queries {
        push_slice(&mut bytes, query.axis.key().as_bytes());
        push_slice(&mut bytes, &query.query.canonical_bytes());
    }
    let mut subjects = scalar
        .iter()
        .map(|declaration| {
            let mut subject = Vec::new();
            declaration.subject.encode(&mut subject);
            subject
        })
        .collect::<Vec<_>>();
    subjects.sort();
    subjects.dedup();
    push_len(&mut bytes, subjects.len());
    for subject in &subjects {
        push_slice(&mut bytes, subject);
    }
    let mut scalar_rows = Vec::with_capacity(scalar.len());
    for declaration in scalar {
        let mut subject = Vec::new();
        declaration.subject.encode(&mut subject);
        let subject_index = subjects
            .binary_search(&subject)
            .expect("every numerical subject was inserted into the subject table");
        let source_index = sources.index_of(declaration.source.as_ref());
        let mut row = Vec::new();
        encode_compact_index(&mut row, subject_index);
        row.push(declaration.dimension.tag());
        declaration.behaviour.encode(&mut row);
        declaration.means.encode(&mut row);
        encode_compact_index(&mut row, source_index);
        scalar_rows.push(row);
    }
    scalar_rows.sort_unstable();
    push_len(&mut bytes, scalar_rows.len());
    for row in scalar_rows {
        bytes.extend_from_slice(&row);
    }
    push_slice(&mut bytes, DISPATCHABILITY_DOMAIN);
    push_len(&mut bytes, dispatchability.len());
    for fact in dispatchability {
        let source_index = sources.index_of(fact.source.as_ref());
        fact.encode(&mut bytes, source_index);
    }
    // The complete subject and its verdict, in uniqueness-key order
    // `(subject, phase)`. Insertion order is not identity: two profiles that
    // declare the same rows in different sequences encode one descriptor.
    // Every dimension is encoded: two profiles differing only in which memory
    // domain they fence declare different realizations and must not share a
    // descriptor, which is the whole reason the fact is atomic.
    push_slice(&mut bytes, SYNCHRONIZATION_DOMAIN);
    push_len(&mut bytes, synchronization.len());
    for declared in synchronization {
        let subject = declared.subject();
        bytes.push(subject.kind.tag());
        bytes.push(subject.execution_scope.tag());
        bytes.push(subject.visibility_scope.tag());
        bytes.push(u8::from(subject.fenced_spaces.workgroup));
        bytes.push(u8::from(subject.fenced_spaces.device));
        bytes.push(subject.ordering.tag());
        bytes.push(declared.realization().tag());
        let source_index = sources.index_of(declared.source_ref());
        encode_compact_index(&mut bytes, source_index);
    }
    // The conditional sections. See this function's header for the derivation
    // that keeps `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` at `v11`.
    if !evaluation_order.is_empty() {
        push_slice(&mut bytes, EVALUATION_ORDER_DOMAIN);
        push_len(&mut bytes, evaluation_order.len());
        for fact in evaluation_order {
            let source_index = sources.index_of(fact.source.as_ref());
            fact.encode(&mut bytes, source_index);
        }
    }
    if !cost_rows.is_empty() {
        push_slice(&mut bytes, COST_ROW_DOMAIN);
        push_len(&mut bytes, cost_rows.len());
        for fact in cost_rows {
            let source_index = sources.index_of(fact.source.as_ref());
            fact.encode(&mut bytes, source_index);
        }
    }
    if !tree_width_policies.is_empty() {
        push_slice(&mut bytes, WORKGROUP_TREE_WIDTH_POLICY_DOMAIN);
        push_len(&mut bytes, tree_width_policies.len());
        for fact in tree_width_policies {
            let source_index = sources.index_of(fact.source.as_ref());
            fact.encode(&mut bytes, source_index);
        }
    }
    if !elementary.is_empty() {
        push_slice(&mut bytes, ELEMENTARY_REALIZATION_DOMAIN);
        push_len(&mut bytes, elementary.len());
        for realization in elementary {
            push_slice(
                &mut bytes,
                realization.contract().canonical_encoding().as_bytes(),
            );
            push_slice(
                &mut bytes,
                &realization.bound_evidence().canonical_encoding(),
            );
            push_slice(
                &mut bytes,
                &realization.exceptional_evidence().canonical_encoding(),
            );
            let source_index = sources.index_of(realization.source());
            encode_compact_index(&mut bytes, source_index);
        }
    }
    if !subgroup.is_empty() {
        push_slice(&mut bytes, SUBGROUP_REALIZATION_DOMAIN);
        push_len(&mut bytes, subgroup.len());
        for declared in subgroup {
            declared.subject().encode(&mut bytes);
            bytes.push(declared.realization().tag());
            let source_index = sources.index_of(declared.source_ref());
            encode_compact_index(&mut bytes, source_index);
        }
    }
    if let Some(query) = subgroup_query {
        push_slice(&mut bytes, SUBGROUP_WIDTH_QUERY_DOMAIN);
        push_slice(&mut bytes, &query.canonical_bytes());
    }
    bytes
}
