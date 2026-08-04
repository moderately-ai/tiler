//! Whole-program structural, coverage, dependency, storage, and lifetime
//! verification.
//!
//! Region-local and kernel-local verification is necessary but not sufficient:
//! it cannot see fan-out lifetimes, output completeness, cross-stage storage, or
//! allocation reuse. This module proves the obligations that only exist at whole
//! program scope, in a fixed phase order so a rejected program names the exact
//! violated rule rather than a generic mismatch.
//!
//! What it deliberately does **not** prove: that a stage's bound kernel computes
//! the semantic operations the stage claims to cover. Structural coverage is a
//! completeness and disjointness obligation over one exact graph; semantic
//! equivalence evidence is compiler-owned refinement evidence (ADR 0071), and
//! this layer must not be read as supplying it.

use std::collections::BTreeSet;

use super::abi::ExprNode;
use super::builder::SemanticSubject;
use super::error::{KernelProgramDiagnostic, ProgramEntityKind};
use super::model::{
    CanonicalKeys, DependencyReasonData, DerivedProgramFacts, KernelProgramData,
    ROUTING_COMMIT_TRANSITIONS, RoutingCommitState, StageAccessMode, ValueDefinition, ValueRole,
    canonical_keys,
};

/// One stage access to one materialized value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ValueAccess {
    stage: u32,
    mode: StageAccessMode,
}

/// Verifies one assembled program and derives the facts the product retains.
///
/// # Errors
///
/// Returns the first violated [`KernelProgramDiagnostic`] in phase order.
pub(super) fn verify_program(
    data: &KernelProgramData,
    subject: &SemanticSubject,
) -> Result<(DerivedProgramFacts, CanonicalKeys), KernelProgramDiagnostic> {
    if data.stages.is_empty()
        || data.values.is_empty()
        || data.allocations.is_empty()
        || data.outputs.is_empty()
    {
        return Err(KernelProgramDiagnostic::EmptyProgram);
    }
    let accesses = value_accesses(data);
    let definitions = definitions(data)?;
    verify_coverage(data, subject)?;

    let keys = canonical_keys(data, &definitions);
    verify_unambiguous(&keys)?;

    // Every edge must state an obligation its two stages realize before the
    // graph those edges induce is ordered: a cycle closed by a meaningless edge
    // should name the meaningless edge.
    verify_dependencies(data, &accesses, &definitions)?;
    let successors = successor_lists(data);
    let execution_order = canonical_execution_order(data, &keys.stages, &successors)
        .ok_or(KernelProgramDiagnostic::DependencyCycle)?;

    // After the dependency phase, so a split whose passes are unordered reports
    // the missing edge rather than the split, and before storage, so a split
    // that stages its partials in an aliasable place is named as a split.
    verify_partial_reductions(data, &accesses, &definitions)?;

    verify_usage(data, &accesses)?;
    verify_storage(data)?;
    verify_reuse(data, &accesses, &definitions, &execution_order, &successors)?;
    verify_outputs(data, subject)?;
    verify_components(data, subject)?;

    // Last, because a program whose structure does not hold has a more basic
    // defect than an incomplete routing contract, and reporting the contract
    // first would send a reader to the wrong place.
    verify_abi(data)?;
    verify_routing_commit(data)?;

    Ok((
        DerivedProgramFacts {
            definitions,
            execution_order,
        },
        keys,
    ))
}

fn position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("a bounded program arena fits u32")
}

/// Proves the program declares a guard and retains no unreachable ABI node.
///
/// Identity folds each ABI use site by content key, and a content key names the
/// node's whole subtree, so an arena node no use site reaches would be retained
/// bytes that identity does not cover. Rejecting it here is what lets the
/// identity encoder fold the arena transitively instead of a second time.
fn verify_abi(data: &KernelProgramData) -> Result<(), KernelProgramDiagnostic> {
    let Some(guard) = data.applicability_guard else {
        return Err(KernelProgramDiagnostic::MissingApplicabilityGuard);
    };
    let mut reached = vec![false; data.abi_expressions.len()];
    let mut frontier = vec![guard];
    for stage in &data.stages {
        frontier.push(stage.launch.grid_threads);
        frontier.push(stage.launch.threads_per_workgroup);
        for access in &stage.accesses {
            frontier.push(access.accessible_bytes);
        }
    }
    while let Some(node) = frontier.pop() {
        if reached[position(node)] {
            continue;
        }
        reached[position(node)] = true;
        match &data.abi_expressions[position(node)] {
            ExprNode::Root(_) => {}
            ExprNode::Unary { operand, .. } => frontier.push(*operand),
            ExprNode::Binary { left, right, .. } => {
                frontier.push(*left);
                frontier.push(*right);
            }
            ExprNode::Select {
                condition,
                if_true,
                if_false,
            } => {
                frontier.push(*condition);
                frontier.push(*if_true);
                frontier.push(*if_false);
            }
        }
    }
    if reached.iter().any(|node| !node) {
        return Err(KernelProgramDiagnostic::UnreferencedAbiExpression);
    }
    Ok(())
}

/// Proves the declared routing-commit steps span the whole ordered lifecycle.
///
/// The builder proves each step continues the chain and that only the step
/// leaving [`RoutingCommitState::Preflight`] permits fallback. What only whole-
/// program scope can see is that the chain was carried to its end: a program
/// stopping at `Committed` would leave a runtime with no declared contract for
/// the transitions it must still make.
fn verify_routing_commit(data: &KernelProgramData) -> Result<(), KernelProgramDiagnostic> {
    let complete = data.routing_commit.len() == ROUTING_COMMIT_TRANSITIONS
        && data
            .routing_commit
            .last()
            .is_some_and(|last| last.to == RoutingCommitState::Published);
    if complete {
        Ok(())
    } else {
        Err(KernelProgramDiagnostic::IncompleteRoutingCommitContract {
            declared: data.routing_commit.len(),
            required: ROUTING_COMMIT_TRANSITIONS,
        })
    }
}

/// Groups every stage access by the materialized value it addresses.
fn value_accesses(data: &KernelProgramData) -> Vec<Vec<ValueAccess>> {
    let mut accesses = vec![Vec::new(); data.values.len()];
    for (index, stage) in data.stages.iter().enumerate() {
        for access in &stage.accesses {
            let value = position(data.views[position(access.view)].value);
            accesses[value].push(ValueAccess {
                stage: ordinal(index),
                mode: access.mode,
            });
        }
    }
    accesses
}

/// Derives the unique defining stage and write position of every value.
fn definitions(
    data: &KernelProgramData,
) -> Result<Vec<Option<ValueDefinition>>, KernelProgramDiagnostic> {
    let mut definitions: Vec<Option<ValueDefinition>> = vec![None; data.values.len()];
    for (index, stage) in data.stages.iter().enumerate() {
        let mut write_position = 0_u32;
        for access in &stage.accesses {
            if access.mode != StageAccessMode::Write {
                continue;
            }
            let value = position(data.views[position(access.view)].value);
            if definitions[value].is_some() {
                return Err(KernelProgramDiagnostic::MultipleWriters);
            }
            definitions[value] = Some(ValueDefinition {
                stage: ordinal(index),
                write_position,
            });
            write_position = write_position.saturating_add(1);
        }
    }
    for (index, value) in data.values.iter().enumerate() {
        let written = definitions[index].is_some();
        match value.role {
            ValueRole::Input if written => {
                return Err(KernelProgramDiagnostic::ExternalValueWritten);
            }
            ValueRole::Temporary | ValueRole::Output if !written => {
                return Err(KernelProgramDiagnostic::MissingWriter);
            }
            ValueRole::Input | ValueRole::Temporary | ValueRole::Output => {}
        }
    }
    Ok(definitions)
}

/// Proves the stages cover exactly the operations of the bound semantic program.
fn verify_coverage(
    data: &KernelProgramData,
    subject: &SemanticSubject,
) -> Result<(), KernelProgramDiagnostic> {
    let covered: BTreeSet<u32> = data
        .stages
        .iter()
        .flat_map(|stage| stage.coverage.iter().map(|occurrence| occurrence.get()))
        .collect();
    let covered_count = ordinal(covered.len());
    if covered_count != subject.operations
        || covered
            .last()
            .is_some_and(|last| *last >= subject.operations)
    {
        return Err(KernelProgramDiagnostic::IncompleteCoverage {
            covered: covered_count,
            required: subject.operations,
        });
    }
    Ok(())
}

/// Proves each canonical key category is pairwise distinct.
///
/// Identity encodes every cross-reference by content key. A collision would let
/// two structurally different programs share bytes, so it fails closed here
/// instead of being resolved by an arena position.
fn verify_unambiguous(keys: &CanonicalKeys) -> Result<(), KernelProgramDiagnostic> {
    for (entity, category) in [
        (ProgramEntityKind::Stage, &keys.stages),
        (ProgramEntityKind::Value, &keys.values),
        (ProgramEntityKind::View, &keys.views),
        (ProgramEntityKind::Allocation, &keys.allocations),
    ] {
        let mut sorted: Vec<&Vec<u8>> = category.iter().collect();
        sorted.sort_unstable();
        if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(KernelProgramDiagnostic::AmbiguousCanonicalKey { entity });
        }
    }
    Ok(())
}

/// Builds the deduplicated successor list of the stage dependency graph.
fn successor_lists(data: &KernelProgramData) -> Vec<Vec<usize>> {
    let mut edges: Vec<(usize, usize)> = data
        .dependencies
        .iter()
        .map(|dependency| {
            (
                position(dependency.predecessor),
                position(dependency.successor),
            )
        })
        .collect();
    edges.sort_unstable();
    edges.dedup();
    let mut successors = vec![Vec::new(); data.stages.len()];
    for (predecessor, successor) in edges {
        successors[predecessor].push(successor);
    }
    successors
}

/// Returns the canonical topological stage order, or `None` for a cyclic graph.
///
/// Ties among ready stages are broken by canonical stage key, so the order is a
/// function of program content and never of builder insertion.
fn canonical_execution_order(
    data: &KernelProgramData,
    stage_keys: &[Vec<u8>],
    successors: &[Vec<usize>],
) -> Option<Vec<u32>> {
    let count = data.stages.len();
    let mut indegree = vec![0_usize; count];
    for list in successors {
        for successor in list {
            indegree[*successor] = indegree[*successor].saturating_add(1);
        }
    }
    let mut ready: Vec<usize> = (0..count).filter(|stage| indegree[*stage] == 0).collect();
    let mut order = Vec::with_capacity(count);
    while !ready.is_empty() {
        let choice = ready
            .iter()
            .enumerate()
            .min_by(|left, right| stage_keys[*left.1].cmp(&stage_keys[*right.1]))
            .map(|(slot, _)| slot)?;
        let stage = ready.swap_remove(choice);
        order.push(ordinal(stage));
        for successor in &successors[stage] {
            indegree[*successor] = indegree[*successor].saturating_sub(1);
            if indegree[*successor] == 0 {
                ready.push(*successor);
            }
        }
    }
    (order.len() == count).then_some(order)
}

/// Proves every dependency states a real obligation and every read has one.
fn verify_dependencies(
    data: &KernelProgramData,
    accesses: &[Vec<ValueAccess>],
    definitions: &[Option<ValueDefinition>],
) -> Result<(), KernelProgramDiagnostic> {
    for dependency in &data.dependencies {
        let realized = match dependency.reason {
            DependencyReasonData::Data(value) => {
                let value = position(value);
                definitions[value]
                    .is_some_and(|definition| definition.stage == dependency.predecessor)
                    && accesses[value].iter().any(|access| {
                        access.stage == dependency.successor && access.mode == StageAccessMode::Read
                    })
            }
            DependencyReasonData::StorageHandoff(allocation) => {
                // A handoff releases storage from one value to a *different*
                // one; an allocation holding a single value can never carry it.
                let bound = bound_values(data, position(allocation));
                bound.iter().any(|old| {
                    accesses[*old]
                        .iter()
                        .any(|access| access.stage == dependency.predecessor)
                        && bound.iter().any(|new| {
                            new != old
                                && definitions[*new].is_some_and(|definition| {
                                    definition.stage == dependency.successor
                                })
                        })
                })
            }
        };
        if !realized {
            return Err(KernelProgramDiagnostic::MisattributedDependency);
        }
    }
    for (value, definition) in definitions.iter().enumerate() {
        let Some(definition) = definition else {
            continue;
        };
        for access in &accesses[value] {
            if access.mode != StageAccessMode::Read {
                continue;
            }
            let declared = data.dependencies.iter().any(|dependency| {
                dependency.predecessor == definition.stage
                    && dependency.successor == access.stage
                    && dependency.reason == DependencyReasonData::Data(ordinal(value))
            });
            if !declared {
                return Err(KernelProgramDiagnostic::MissingDataDependency);
            }
        }
    }
    Ok(())
}

/// Proves every declared split reduction is one the program actually realizes.
///
/// The obligations here are exactly the ones a single region cannot see. That
/// the partial is initialized before it is used is proven in two parts: the
/// named producer is its *unique* defining stage, and the named combiner reads
/// it — for which the dependency phase already required a data edge from the
/// definer, which is the visibility transition across the dispatch boundary.
/// That the final pass covers every contributor exactly once is proven by the
/// partition count agreeing with the two materialized extents and the split's
/// own coverage product being representable.
fn verify_partial_reductions(
    data: &KernelProgramData,
    accesses: &[Vec<ValueAccess>],
    definitions: &[Option<ValueDefinition>],
) -> Result<(), KernelProgramDiagnostic> {
    // A dispatch that computes no operation of the bound graph has to be
    // accounted for. The only account this profile admits is being the final
    // pass of a declared split, whose partial pass already claims the reduction
    // both of them realize.
    for (index, stage) in data.stages.iter().enumerate() {
        if stage.coverage.is_empty()
            && !data
                .partial_reductions
                .iter()
                .any(|split| split.combiner == ordinal(index))
        {
            return Err(KernelProgramDiagnostic::UncoveringStage);
        }
    }
    for split in &data.partial_reductions {
        let partial = position(split.partial);
        let result = position(split.result);
        if definitions[partial].is_none_or(|writer| writer.stage != split.producer) {
            return Err(KernelProgramDiagnostic::PartialNotInitializedByProducer);
        }
        if definitions[result].is_none_or(|writer| writer.stage != split.combiner) {
            return Err(KernelProgramDiagnostic::PartialResultNotProducedByCombiner);
        }
        if !accesses[partial]
            .iter()
            .any(|access| access.stage == split.combiner && access.mode == StageAccessMode::Read)
        {
            return Err(KernelProgramDiagnostic::PartialNotConsumedByCombiner);
        }
        if data.values[partial].role != ValueRole::Temporary {
            return Err(KernelProgramDiagnostic::PartialNotMaterialized);
        }
        // A split covering nothing has no final pass to speak of, and one whose
        // product is unrepresentable states a coverage no reader can check.
        if split
            .partitions
            .checked_mul(split.contributors_per_partition)
            .is_none()
            || split.partitions == 0
        {
            return Err(KernelProgramDiagnostic::PartialCoverageUnrepresentable);
        }
        // Exactly one partial per (result position, partition): more would
        // leave partials no pass combines, fewer would drop contributors.
        let expected = data.values[result]
            .shape
            .element_count()
            .and_then(|results| u64::try_from(results).ok())
            .and_then(|results| results.checked_mul(split.partitions));
        let staged = data.values[partial]
            .shape
            .element_count()
            .and_then(|partials| u64::try_from(partials).ok());
        if expected.is_none() || expected != staged {
            return Err(KernelProgramDiagnostic::PartialExtentMismatch);
        }
    }
    Ok(())
}

/// Returns the value positions bound to one allocation.
fn bound_values(data: &KernelProgramData, allocation: usize) -> Vec<usize> {
    let allocation = ordinal(allocation);
    data.values
        .iter()
        .enumerate()
        .filter(|(_, value)| value.allocation == allocation)
        .map(|(index, _)| index)
        .collect()
}

/// Proves every declared value, view, and allocation is actually used.
fn verify_usage(
    data: &KernelProgramData,
    accesses: &[Vec<ValueAccess>],
) -> Result<(), KernelProgramDiagnostic> {
    if accesses.iter().any(Vec::is_empty) {
        return Err(KernelProgramDiagnostic::UnusedValue);
    }
    let mut used_views = vec![false; data.views.len()];
    for stage in &data.stages {
        for access in &stage.accesses {
            used_views[position(access.view)] = true;
        }
    }
    if used_views.iter().any(|used| !used) {
        return Err(KernelProgramDiagnostic::UnusedView);
    }
    if (0..data.allocations.len()).any(|allocation| bound_values(data, allocation).is_empty()) {
        return Err(KernelProgramDiagnostic::UnusedAllocation);
    }
    Ok(())
}

/// Proves the baseline aliasing contract of the first execution profile.
///
/// Inputs and outputs never share storage. Only internal temporaries may share
/// one allocation, and only under the reuse obligations proven separately.
fn verify_storage(data: &KernelProgramData) -> Result<(), KernelProgramDiagnostic> {
    for allocation in 0..data.allocations.len() {
        let bound = bound_values(data, allocation);
        if bound.len() > 1
            && bound
                .iter()
                .any(|value| data.values[*value].role != ValueRole::Temporary)
        {
            return Err(KernelProgramDiagnostic::ForbiddenAlias);
        }
    }
    Ok(())
}

/// Proves every shared allocation satisfies the conservative reuse contract.
fn verify_reuse(
    data: &KernelProgramData,
    accesses: &[Vec<ValueAccess>],
    definitions: &[Option<ValueDefinition>],
    execution_order: &[u32],
    successors: &[Vec<usize>],
) -> Result<(), KernelProgramDiagnostic> {
    let mut slot = vec![0_usize; data.stages.len()];
    for (index, stage) in execution_order.iter().enumerate() {
        slot[position(*stage)] = index;
    }
    for allocation in 0..data.allocations.len() {
        let mut bound = bound_values(data, allocation);
        if bound.len() < 2 {
            continue;
        }
        bound.sort_unstable_by_key(|value| {
            definitions[*value].map_or(usize::MAX, |definition| slot[position(definition.stage)])
        });
        for pair in bound.windows(2) {
            let (old, new) = (pair[0], pair[1]);
            let Some(writer) = definitions[new] else {
                return Err(KernelProgramDiagnostic::MissingWriter);
            };
            let writer_slot = slot[position(writer.stage)];
            let last_user = accesses[old]
                .iter()
                .max_by_key(|access| slot[position(access.stage)])
                .ok_or(KernelProgramDiagnostic::UnusedValue)?;
            if slot[position(last_user.stage)] >= writer_slot {
                return Err(KernelProgramDiagnostic::ReuseLifetimeOverlap);
            }
            let handed_off = data.dependencies.iter().any(|dependency| {
                dependency.predecessor == last_user.stage
                    && dependency.successor == writer.stage
                    && dependency.reason
                        == DependencyReasonData::StorageHandoff(ordinal(allocation))
            });
            if !handed_off {
                return Err(KernelProgramDiagnostic::ReuseMissingHandoff);
            }
            for access in &accesses[old] {
                if !reaches(successors, position(access.stage), position(writer.stage)) {
                    return Err(KernelProgramDiagnostic::ReuseLiveAlias);
                }
            }
        }
    }
    Ok(())
}

/// Returns whether `target` is reachable from `source` in the dependency graph.
///
/// Reachability, not a position in one chosen linearization, is the reuse
/// proof: a future concurrent execution profile must preserve it.
fn reaches(successors: &[Vec<usize>], source: usize, target: usize) -> bool {
    let mut seen = vec![false; successors.len()];
    let mut frontier = vec![source];
    seen[source] = true;
    while let Some(stage) = frontier.pop() {
        if stage == target {
            return true;
        }
        for successor in &successors[stage] {
            if !seen[*successor] {
                seen[*successor] = true;
                frontier.push(*successor);
            }
        }
    }
    false
}

/// Proves the published outputs are the semantic interface, in its order.
///
/// Coverage alone would leave the declared order of the published records
/// observable through
/// [`VerifiedKernelProgram::outputs`](super::VerifiedKernelProgram::outputs) and
/// decided by nothing. A builder is opened against an unforgeable semantic
/// subject, and the ordered output interface belongs to that subject: a program
/// realizes it rather than restating it, so a publication that permutes it is a
/// disagreement with the program's own subject and is refused here. Pinning the
/// order is what lets identity fold the published list in declaration order
/// instead of sorting it, and what makes a consumer's ordered output interface a
/// fact it reads rather than one it re-derives by key.
fn verify_outputs(
    data: &KernelProgramData,
    subject: &SemanticSubject,
) -> Result<(), KernelProgramDiagnostic> {
    let mut published = 0_usize;
    for (key, _, value_type) in &subject.outputs {
        let start = published;
        while data
            .outputs
            .get(published)
            .is_some_and(|output| &output.key == key)
        {
            published = published.saturating_add(1);
        }
        if published == start {
            // The key is either published somewhere other than the interface
            // position it holds, or not published at all.
            if data.outputs.iter().any(|output| &output.key == key) {
                return Err(KernelProgramDiagnostic::MisorderedNamedOutput { position: start });
            }
            return Err(KernelProgramDiagnostic::MissingNamedOutput);
        }
        verify_component_order(
            value_type,
            start,
            data.outputs[start..published]
                .iter()
                .map(|output| data.values[position(output.value)].component_role),
        )?;
    }
    if published < data.outputs.len() {
        return Err(KernelProgramDiagnostic::MisorderedNamedOutput {
            position: published,
        });
    }
    for (index, value) in data.values.iter().enumerate() {
        if value.role != ValueRole::Output {
            continue;
        }
        if !data
            .outputs
            .iter()
            .any(|output| output.value == ordinal(index))
        {
            return Err(KernelProgramDiagnostic::UnboundOutputValue);
        }
    }
    Ok(())
}

/// Proves one key's published records follow its semantic component order.
///
/// A record inside one key's run is named by its component *role* and not by a
/// position the caller counts, so the order that binds it is the encoded
/// contract's own declared component order rather than a producer choice. This
/// is a subsequence walk rather than an equality: a run that omits a component
/// is left to [`verify_components`], which reports the incomplete set against
/// the whole interface instead of the first record that skipped one.
///
/// `roles` are the run's published component roles in declared order and
/// `start` is the declared position of its first record, which is what the
/// diagnostic reports. The run is taken as roles rather than as program data
/// because no kernel in this profile writes a compound *output* yet, so this is
/// the boundary at which the rule can be exercised against a contract directly.
fn verify_component_order(
    value_type: &crate::semantic::ResolvedValueType,
    start: usize,
    roles: impl IntoIterator<Item = Option<crate::semantic::EncodedComponentRole>>,
) -> Result<(), KernelProgramDiagnostic> {
    let Some((_, contract)) = value_type.encoded_numeric_parts() else {
        // A plain interface type has one component and no role, and insertion
        // rejects a second publication of one key and role, so the run is one
        // record and carries no order to violate.
        return Ok(());
    };
    let mut components = contract.components().iter();
    for (offset, role) in roles.into_iter().enumerate() {
        if !components.any(|component| Some(component.role()) == role) {
            return Err(KernelProgramDiagnostic::MisorderedNamedOutput {
                position: start.saturating_add(offset),
            });
        }
    }
    Ok(())
}

/// Proves every logical interface value carries exactly its semantic role set.
fn verify_components(
    data: &KernelProgramData,
    subject: &SemanticSubject,
) -> Result<(), KernelProgramDiagnostic> {
    for (key, _, value_type) in &subject.inputs {
        let expected = expected_roles(value_type)?;
        let actual: BTreeSet<_> = data
            .values
            .iter()
            .filter_map(|value| match &value.origin {
                super::model::MaterializedOrigin::ProgramInput { key: bound } if bound == key => {
                    Some(value.component_role)
                }
                super::model::MaterializedOrigin::ProgramInput { .. }
                | super::model::MaterializedOrigin::Internal => None,
            })
            .collect();
        if actual != expected {
            return Err(KernelProgramDiagnostic::IncompleteComponentSet);
        }
    }
    for (key, _, value_type) in &subject.outputs {
        let expected = expected_roles(value_type)?;
        let actual: BTreeSet<_> = data
            .outputs
            .iter()
            .filter(|output| &output.key == key)
            .map(|output| data.values[position(output.value)].component_role)
            .collect();
        if actual != expected {
            return Err(KernelProgramDiagnostic::IncompleteComponentSet);
        }
    }
    Ok(())
}

fn expected_roles(
    value_type: &crate::semantic::ResolvedValueType,
) -> Result<BTreeSet<Option<crate::semantic::EncodedComponentRole>>, KernelProgramDiagnostic> {
    match value_type.encoded_numeric_parts() {
        None => Ok([None].into_iter().collect()),
        Some((_, contract)) if contract.components().is_empty() => {
            Err(KernelProgramDiagnostic::EmptyEncodedComponentSet)
        }
        Some((_, contract)) => Ok(contract
            .components()
            .iter()
            .map(|component| Some(component.role()))
            .collect()),
    }
}

#[cfg(test)]
mod component_tests {
    use super::*;
    use crate::semantic::{
        AttributeFieldId, CanonicalField, CanonicalValue, EncodedComponentDeclaration,
        EncodedComponentRole, EncodedComponentShape, EncodedNumericContract, QuantSchemeKey,
        ResolvedValueType, TypeKey,
    };

    /// An encoded type whose declared component order is not its role order.
    ///
    /// Deliberately descending, because the two orders coincide for the
    /// governed strict-affine contract (codes, scale, zero point are roles 1,
    /// 2, 3) and a fixture where they agree cannot tell the rule this layer
    /// applies — the contract's own declared order — from an incidental sort by
    /// role identifier.
    fn descending_component_type() -> ResolvedValueType {
        let component = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
        ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("test", "descending-components", 1).unwrap(),
            EncodedNumericContract::with_components(
                [CanonicalField::new(
                    AttributeFieldId::new(1),
                    CanonicalValue::boolean(true),
                )],
                [
                    EncodedComponentDeclaration::new(
                        EncodedComponentRole::new(3),
                        component.clone(),
                        EncodedComponentShape::LogicalValue,
                    ),
                    EncodedComponentDeclaration::new(
                        EncodedComponentRole::new(1),
                        component,
                        EncodedComponentShape::LogicalValue,
                    ),
                ],
            )
            .unwrap(),
        )
        .unwrap()
    }

    /// One key's records follow the contract's declared component order.
    ///
    /// The omission case is here because it is the one the rule must *not*
    /// claim: a run that skips a component is a valid subsequence, and naming
    /// it here would take the report away from `verify_components`, which sees
    /// the whole interface and says the set is incomplete.
    #[test]
    fn published_components_follow_the_contracts_declared_order() {
        let value_type = descending_component_type();
        let role = |id| Some(EncodedComponentRole::new(id));
        assert_eq!(
            verify_component_order(&value_type, 7, [role(3), role(1)]),
            Ok(())
        );
        assert_eq!(verify_component_order(&value_type, 7, [role(3)]), Ok(()));
        assert_eq!(verify_component_order(&value_type, 7, [role(1)]), Ok(()));
        assert_eq!(
            verify_component_order(&value_type, 7, [role(1), role(3)]),
            Err(KernelProgramDiagnostic::MisorderedNamedOutput { position: 8 }),
        );
        // A plain interface type has no component order to violate.
        assert_eq!(
            verify_component_order(
                &ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap()),
                0,
                [None]
            ),
            Ok(())
        );
    }

    #[test]
    fn an_encoded_type_without_components_reaches_typed_rejection() {
        let value_type = ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("test", "empty-components", 1).unwrap(),
            EncodedNumericContract::new([CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::boolean(true),
            )])
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            expected_roles(&value_type),
            Err(KernelProgramDiagnostic::EmptyEncodedComponentSet)
        );
    }
}
