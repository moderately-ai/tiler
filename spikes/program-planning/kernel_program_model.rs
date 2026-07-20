//! Compile-checking spike for the KernelProgram and conservative BufferPlan.
//!
//! Run with:
//! `rustc --edition 2021 --test spikes/program-planning/kernel_program_model.rs -o /tmp/tiler-program-plan-spike && /tmp/tiler-program-plan-spike`

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StageId(u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ValueId(u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AllocationId(u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ViewId(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValueRole {
    Input,
    Output,
    Temporary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StageKind {
    ScheduledKernel,
    MaterializeCopy,
    Validation,
    OpaqueCall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DependencyReason {
    Data(ValueId),
    Validation,
    Effect,
    StorageHandoff(AllocationId),
}

#[derive(Clone, Debug)]
struct Dependency {
    predecessor: StageId,
    reason: DependencyReason,
}

#[derive(Clone, Debug)]
struct Stage {
    id: StageId,
    kind: StageKind,
    dependencies: Vec<Dependency>,
    reads: Vec<ValueId>,
    writes: Vec<ValueId>,
}

#[derive(Clone, Debug)]
struct Value {
    id: ValueId,
    role: ValueRole,
    allocation: AllocationId,
    required_bytes: u64,
    required_alignment: u64,
    memory_space: &'static str,
}

#[derive(Clone, Debug)]
struct Allocation {
    id: AllocationId,
    capacity_bytes: u64,
    alignment: u64,
    memory_space: &'static str,
    external: bool,
}

#[derive(Clone, Debug)]
struct ProgramOutput {
    key: &'static str,
    value: ValueId,
}

#[derive(Clone, Debug)]
struct View {
    id: ViewId,
    base: ValueId,
    byte_offset: u64,
    extent_bytes: u64,
    users: Vec<StageId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Action {
    Preflight,
    RoutingCommit,
    AcquireStorage,
    Encode(StageId),
    Fallback,
    PublishOutputs,
}

#[derive(Clone, Debug)]
struct Program {
    stages: Vec<Stage>,
    values: Vec<Value>,
    allocations: Vec<Allocation>,
    views: Vec<View>,
    outputs: Vec<ProgramOutput>,
    actions: Vec<Action>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Error {
    DuplicateStage,
    DuplicateValue,
    DuplicateAllocation,
    DuplicateView,
    UnknownReference,
    DependencyCycle,
    MissingWriter(ValueId),
    MultipleWriters(ValueId),
    MissingDataDependency { reader: StageId, value: ValueId },
    DuplicateOutputKey,
    InvalidOutput(ValueId),
    AllocationTooSmall(ValueId),
    AlignmentMismatch(ValueId),
    MemorySpaceMismatch(ValueId),
    ForbiddenAlias(AllocationId),
    ViewOutOfRange(ViewId),
    ReuseLifetimeOverlap(AllocationId),
    ReuseMissingHandoff(AllocationId),
    FallbackAfterCommit,
    WorkBeforeCommit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Lifetime {
    definition: usize,
    last_use: usize,
}

impl Program {
    fn verify(&self) -> Result<BTreeMap<ValueId, Lifetime>, Error> {
        let stage_ids: BTreeSet<_> = self.stages.iter().map(|stage| stage.id).collect();
        if stage_ids.len() != self.stages.len() {
            return Err(Error::DuplicateStage);
        }
        let value_ids: BTreeSet<_> = self.values.iter().map(|value| value.id).collect();
        if value_ids.len() != self.values.len() {
            return Err(Error::DuplicateValue);
        }
        let allocation_ids: BTreeSet<_> = self
            .allocations
            .iter()
            .map(|allocation| allocation.id)
            .collect();
        if allocation_ids.len() != self.allocations.len() {
            return Err(Error::DuplicateAllocation);
        }
        let view_ids: BTreeSet<_> = self.views.iter().map(|view| view.id).collect();
        if view_ids.len() != self.views.len() {
            return Err(Error::DuplicateView);
        }

        let order = self.topological_order()?;
        let stage_pos: BTreeMap<_, _> = order.iter().enumerate().map(|(i, id)| (*id, i)).collect();
        let stages: BTreeMap<_, _> = self.stages.iter().map(|s| (s.id, s)).collect();
        let values: BTreeMap<_, _> = self.values.iter().map(|v| (v.id, v)).collect();
        let allocations: BTreeMap<_, _> = self.allocations.iter().map(|a| (a.id, a)).collect();

        let mut writers: BTreeMap<ValueId, Vec<StageId>> = BTreeMap::new();
        for stage in &self.stages {
            for value in stage.reads.iter().chain(&stage.writes) {
                if !values.contains_key(value) {
                    return Err(Error::UnknownReference);
                }
            }
            for &value in &stage.writes {
                writers.entry(value).or_default().push(stage.id);
            }
        }

        let mut view_users: BTreeMap<ValueId, Vec<StageId>> = BTreeMap::new();
        for view in &self.views {
            let base = values.get(&view.base).ok_or(Error::UnknownReference)?;
            let end = view
                .byte_offset
                .checked_add(view.extent_bytes)
                .ok_or(Error::ViewOutOfRange(view.id))?;
            if end > base.required_bytes {
                return Err(Error::ViewOutOfRange(view.id));
            }
            for &user in &view.users {
                if !stages.contains_key(&user) {
                    return Err(Error::UnknownReference);
                }
                if base.role != ValueRole::Input {
                    let writer = writers[&base.id][0];
                    if !self.reachable(writer, user) {
                        return Err(Error::MissingDataDependency {
                            reader: user,
                            value: base.id,
                        });
                    }
                }
                view_users.entry(base.id).or_default().push(user);
            }
        }

        for value in &self.values {
            let count = writers.get(&value.id).map_or(0, Vec::len);
            match (value.role, count) {
                (ValueRole::Input, 0) => {}
                (ValueRole::Input, _) => return Err(Error::MultipleWriters(value.id)),
                (_, 0) => return Err(Error::MissingWriter(value.id)),
                (_, 1) => {}
                (_, _) => return Err(Error::MultipleWriters(value.id)),
            }

            let allocation = allocations
                .get(&value.allocation)
                .ok_or(Error::UnknownReference)?;
            if allocation.capacity_bytes < value.required_bytes {
                return Err(Error::AllocationTooSmall(value.id));
            }
            if value.required_alignment == 0
                || allocation.alignment < value.required_alignment
                || allocation.alignment % value.required_alignment != 0
            {
                return Err(Error::AlignmentMismatch(value.id));
            }
            if allocation.memory_space != value.memory_space {
                return Err(Error::MemorySpaceMismatch(value.id));
            }
            if (value.role == ValueRole::Input) != allocation.external {
                return Err(Error::ForbiddenAlias(value.allocation));
            }
        }

        for stage in &self.stages {
            for &read in &stage.reads {
                let value = values[&read];
                if value.role != ValueRole::Input {
                    let writer = writers[&read][0];
                    if !self.reachable(writer, stage.id) {
                        return Err(Error::MissingDataDependency {
                            reader: stage.id,
                            value: read,
                        });
                    }
                }
            }
        }

        let mut keys = BTreeSet::new();
        let mut output_values = BTreeSet::new();
        for output in &self.outputs {
            if !keys.insert(output.key) {
                return Err(Error::DuplicateOutputKey);
            }
            let value = values.get(&output.value).ok_or(Error::UnknownReference)?;
            if value.role != ValueRole::Output {
                return Err(Error::InvalidOutput(output.value));
            }
            if !output_values.insert(output.value) {
                return Err(Error::InvalidOutput(output.value));
            }
        }
        for value in &self.values {
            if value.role == ValueRole::Output && !output_values.contains(&value.id) {
                return Err(Error::InvalidOutput(value.id));
            }
        }

        let mut lifetimes = BTreeMap::new();
        for value in &self.values {
            if value.role == ValueRole::Input {
                continue;
            }
            let writer = writers[&value.id][0];
            let definition = stage_pos[&writer];
            let last_use = self
                .stages
                .iter()
                .filter(|stage| stage.reads.contains(&value.id))
                .map(|stage| stage_pos[&stage.id])
                .chain(
                    view_users
                        .get(&value.id)
                        .into_iter()
                        .flatten()
                        .map(|stage| stage_pos[stage]),
                )
                .max()
                .unwrap_or(definition);
            lifetimes.insert(
                value.id,
                Lifetime {
                    definition,
                    last_use,
                },
            );
        }

        let mut by_allocation: BTreeMap<AllocationId, Vec<&Value>> = BTreeMap::new();
        for value in &self.values {
            by_allocation
                .entry(value.allocation)
                .or_default()
                .push(value);
        }
        for (allocation, mut bound) in by_allocation {
            if bound.len() < 2 {
                continue;
            }
            // Inputs may alias one another. All other shared storage is either a
            // forbidden public alias or a proved temporary-to-temporary handoff.
            if bound.iter().all(|value| value.role == ValueRole::Input) {
                continue;
            }
            if !bound.iter().all(|value| value.role == ValueRole::Temporary) {
                return Err(Error::ForbiddenAlias(allocation));
            }
            bound.sort_by_key(|value| lifetimes[&value.id].definition);
            for pair in bound.windows(2) {
                let old = pair[0];
                let new = pair[1];
                let old_life = lifetimes[&old.id];
                let new_life = lifetimes[&new.id];
                if old_life.last_use >= new_life.definition {
                    return Err(Error::ReuseLifetimeOverlap(allocation));
                }
                let old_last_stage = order[old_life.last_use];
                let new_writer = writers[&new.id][0];
                let handoff = stages[&new_writer].dependencies.iter().any(|dep| {
                    dep.reason == DependencyReason::StorageHandoff(allocation)
                        && self.reachable(old_last_stage, dep.predecessor)
                });
                if !handoff {
                    return Err(Error::ReuseMissingHandoff(allocation));
                }
            }
        }

        let mut committed = false;
        for action in &self.actions {
            match action {
                Action::RoutingCommit => committed = true,
                Action::AcquireStorage | Action::Encode(_) if !committed => {
                    return Err(Error::WorkBeforeCommit)
                }
                Action::Fallback if committed => return Err(Error::FallbackAfterCommit),
                _ => {}
            }
        }

        Ok(lifetimes)
    }

    fn topological_order(&self) -> Result<Vec<StageId>, Error> {
        let ids: BTreeSet<_> = self.stages.iter().map(|stage| stage.id).collect();
        let mut indegree: BTreeMap<_, usize> = ids.iter().map(|id| (*id, 0)).collect();
        let mut successors: BTreeMap<StageId, Vec<StageId>> = BTreeMap::new();
        for stage in &self.stages {
            for dependency in &stage.dependencies {
                if !ids.contains(&dependency.predecessor) {
                    return Err(Error::UnknownReference);
                }
                *indegree.get_mut(&stage.id).ok_or(Error::UnknownReference)? += 1;
                successors
                    .entry(dependency.predecessor)
                    .or_default()
                    .push(stage.id);
            }
        }
        let mut ready: BTreeSet<_> = indegree
            .iter()
            .filter_map(|(id, degree)| (*degree == 0).then_some(*id))
            .collect();
        let mut order = Vec::new();
        while let Some(id) = ready.pop_first() {
            order.push(id);
            for successor in successors.get(&id).into_iter().flatten() {
                let degree = indegree.get_mut(successor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(*successor);
                }
            }
        }
        if order.len() != self.stages.len() {
            return Err(Error::DependencyCycle);
        }
        Ok(order)
    }

    fn reachable(&self, from: StageId, to: StageId) -> bool {
        if from == to {
            return true;
        }
        let mut seen = BTreeSet::new();
        let mut frontier = vec![to];
        while let Some(current) = frontier.pop() {
            if !seen.insert(current) {
                continue;
            }
            let Some(stage) = self.stages.iter().find(|stage| stage.id == current) else {
                return false;
            };
            for dependency in &stage.dependencies {
                if dependency.predecessor == from {
                    return true;
                }
                frontier.push(dependency.predecessor);
            }
        }
        false
    }
}

fn dependency(predecessor: u16, reason: DependencyReason) -> Dependency {
    Dependency {
        predecessor: StageId(predecessor),
        reason,
    }
}

fn stage(id: u16, dependencies: Vec<Dependency>, reads: &[u16], writes: &[u16]) -> Stage {
    Stage {
        id: StageId(id),
        kind: StageKind::ScheduledKernel,
        dependencies,
        reads: reads.iter().copied().map(ValueId).collect(),
        writes: writes.iter().copied().map(ValueId).collect(),
    }
}

fn value(id: u16, role: ValueRole, allocation: u16, bytes: u64) -> Value {
    Value {
        id: ValueId(id),
        role,
        allocation: AllocationId(allocation),
        required_bytes: bytes,
        required_alignment: 16,
        memory_space: "device",
    }
}

fn allocation(id: u16, bytes: u64, external: bool) -> Allocation {
    Allocation {
        id: AllocationId(id),
        capacity_bytes: bytes,
        alignment: 16,
        memory_space: "device",
        external,
    }
}

fn committed_actions(stages: &[u16]) -> Vec<Action> {
    let mut actions = vec![
        Action::Preflight,
        Action::RoutingCommit,
        Action::AcquireStorage,
    ];
    actions.extend(stages.iter().copied().map(|id| Action::Encode(StageId(id))));
    actions.push(Action::PublishOutputs);
    actions
}

#[test]
fn fanout_extends_temporary_to_last_consumer_and_supports_named_outputs() {
    let program = Program {
        stages: vec![
            stage(0, vec![], &[0], &[1]),
            stage(
                1,
                vec![dependency(0, DependencyReason::Data(ValueId(1)))],
                &[1],
                &[2],
            ),
            stage(
                2,
                vec![dependency(0, DependencyReason::Data(ValueId(1)))],
                &[1],
                &[3],
            ),
        ],
        values: vec![
            value(0, ValueRole::Input, 0, 64),
            value(1, ValueRole::Temporary, 1, 64),
            value(2, ValueRole::Output, 2, 64),
            value(3, ValueRole::Output, 3, 4),
        ],
        allocations: vec![
            allocation(0, 64, true),
            allocation(1, 64, false),
            allocation(2, 64, false),
            allocation(3, 4, false),
        ],
        views: vec![],
        outputs: vec![
            ProgramOutput {
                key: "scores",
                value: ValueId(2),
            },
            ProgramOutput {
                key: "summary",
                value: ValueId(3),
            },
        ],
        actions: committed_actions(&[0, 1, 2]),
    };

    let lifetimes = program.verify().unwrap();
    assert_eq!(
        lifetimes[&ValueId(1)],
        Lifetime {
            definition: 0,
            last_use: 2
        }
    );
    assert_eq!(
        program.topological_order().unwrap(),
        vec![StageId(0), StageId(1), StageId(2)]
    );
}

#[test]
fn multi_dispatch_reduction_scratch_has_explicit_lifetime() {
    let program = Program {
        stages: vec![
            stage(0, vec![], &[0], &[1]),
            stage(
                1,
                vec![dependency(0, DependencyReason::Data(ValueId(1)))],
                &[1],
                &[2],
            ),
        ],
        values: vec![
            value(0, ValueRole::Input, 0, 4096),
            value(1, ValueRole::Temporary, 1, 128),
            value(2, ValueRole::Output, 2, 4),
        ],
        allocations: vec![
            allocation(0, 4096, true),
            allocation(1, 128, false),
            allocation(2, 4, false),
        ],
        views: vec![],
        outputs: vec![ProgramOutput {
            key: "sum",
            value: ValueId(2),
        }],
        actions: committed_actions(&[0, 1]),
    };
    assert_eq!(
        program.verify().unwrap()[&ValueId(1)],
        Lifetime {
            definition: 0,
            last_use: 1
        }
    );
}

#[test]
fn temporary_reuse_requires_explicit_handoff() {
    let mut program = Program {
        stages: vec![
            stage(0, vec![], &[0], &[1]),
            stage(
                1,
                vec![dependency(0, DependencyReason::Data(ValueId(1)))],
                &[1],
                &[2],
            ),
            stage(2, vec![dependency(1, DependencyReason::Effect)], &[0], &[3]),
            stage(
                3,
                vec![dependency(2, DependencyReason::Data(ValueId(3)))],
                &[3],
                &[4],
            ),
        ],
        values: vec![
            value(0, ValueRole::Input, 0, 64),
            value(1, ValueRole::Temporary, 1, 64),
            value(2, ValueRole::Output, 2, 64),
            value(3, ValueRole::Temporary, 1, 32),
            value(4, ValueRole::Output, 3, 32),
        ],
        allocations: vec![
            allocation(0, 64, true),
            allocation(1, 64, false),
            allocation(2, 64, false),
            allocation(3, 32, false),
        ],
        views: vec![],
        outputs: vec![
            ProgramOutput {
                key: "a",
                value: ValueId(2),
            },
            ProgramOutput {
                key: "b",
                value: ValueId(4),
            },
        ],
        actions: committed_actions(&[0, 1, 2, 3]),
    };
    assert_eq!(
        program.verify(),
        Err(Error::ReuseMissingHandoff(AllocationId(1)))
    );

    program.stages[2].dependencies.push(dependency(
        1,
        DependencyReason::StorageHandoff(AllocationId(1)),
    ));
    assert!(program.verify().is_ok());
}

#[test]
fn overlapping_fanout_scratch_cannot_be_reused() {
    let program = Program {
        stages: vec![
            stage(0, vec![], &[0], &[1]),
            stage(
                1,
                vec![dependency(0, DependencyReason::Data(ValueId(1)))],
                &[1],
                &[2],
            ),
            stage(
                2,
                vec![
                    dependency(0, DependencyReason::Data(ValueId(1))),
                    dependency(1, DependencyReason::StorageHandoff(AllocationId(1))),
                ],
                &[1],
                &[3],
            ),
        ],
        values: vec![
            value(0, ValueRole::Input, 0, 64),
            value(1, ValueRole::Temporary, 1, 64),
            value(2, ValueRole::Output, 2, 64),
            value(3, ValueRole::Temporary, 1, 64),
        ],
        allocations: vec![
            allocation(0, 64, true),
            allocation(1, 64, false),
            allocation(2, 64, false),
        ],
        views: vec![],
        outputs: vec![ProgramOutput {
            key: "result",
            value: ValueId(2),
        }],
        actions: committed_actions(&[0, 1, 2]),
    };
    assert_eq!(
        program.verify(),
        Err(Error::ReuseLifetimeOverlap(AllocationId(1)))
    );
}

#[test]
fn public_values_must_not_alias() {
    let program = Program {
        stages: vec![stage(0, vec![], &[0], &[1])],
        values: vec![
            value(0, ValueRole::Input, 0, 64),
            value(1, ValueRole::Output, 0, 64),
        ],
        allocations: vec![allocation(0, 64, true)],
        views: vec![],
        outputs: vec![ProgramOutput {
            key: "result",
            value: ValueId(1),
        }],
        actions: committed_actions(&[0]),
    };
    assert_eq!(
        program.verify(),
        Err(Error::ForbiddenAlias(AllocationId(0)))
    );
}

#[test]
fn inputs_may_alias_without_granting_noalias() {
    let program = Program {
        stages: vec![stage(0, vec![], &[0, 1], &[2])],
        values: vec![
            value(0, ValueRole::Input, 0, 64),
            value(1, ValueRole::Input, 0, 64),
            value(2, ValueRole::Output, 1, 64),
        ],
        allocations: vec![allocation(0, 64, true), allocation(1, 64, false)],
        views: vec![],
        outputs: vec![ProgramOutput {
            key: "result",
            value: ValueId(2),
        }],
        actions: committed_actions(&[0]),
    };
    assert!(program.verify().is_ok());
}

#[test]
fn view_use_extends_base_lifetime_and_blocks_early_reuse() {
    let program = Program {
        stages: vec![
            stage(0, vec![], &[0], &[1]),
            stage(
                1,
                vec![dependency(0, DependencyReason::Data(ValueId(1)))],
                &[1],
                &[2],
            ),
            stage(
                2,
                vec![
                    dependency(0, DependencyReason::Data(ValueId(1))),
                    dependency(1, DependencyReason::StorageHandoff(AllocationId(1))),
                ],
                &[],
                &[3],
            ),
        ],
        values: vec![
            value(0, ValueRole::Input, 0, 64),
            value(1, ValueRole::Temporary, 1, 64),
            value(2, ValueRole::Output, 2, 16),
            value(3, ValueRole::Temporary, 1, 64),
        ],
        allocations: vec![
            allocation(0, 64, true),
            allocation(1, 64, false),
            allocation(2, 16, false),
        ],
        views: vec![View {
            id: ViewId(0),
            base: ValueId(1),
            byte_offset: 16,
            extent_bytes: 16,
            users: vec![StageId(2)],
        }],
        outputs: vec![ProgramOutput {
            key: "result",
            value: ValueId(2),
        }],
        actions: committed_actions(&[0, 1, 2]),
    };

    assert_eq!(
        program.verify(),
        Err(Error::ReuseLifetimeOverlap(AllocationId(1)))
    );
}

#[test]
fn data_use_requires_dependency_not_just_list_order() {
    let program = Program {
        stages: vec![stage(0, vec![], &[0], &[1]), stage(1, vec![], &[1], &[2])],
        values: vec![
            value(0, ValueRole::Input, 0, 64),
            value(1, ValueRole::Temporary, 1, 64),
            value(2, ValueRole::Output, 2, 64),
        ],
        allocations: vec![
            allocation(0, 64, true),
            allocation(1, 64, false),
            allocation(2, 64, false),
        ],
        views: vec![],
        outputs: vec![ProgramOutput {
            key: "result",
            value: ValueId(2),
        }],
        actions: committed_actions(&[0, 1]),
    };
    assert_eq!(
        program.verify(),
        Err(Error::MissingDataDependency {
            reader: StageId(1),
            value: ValueId(1)
        })
    );
}

#[test]
fn fallback_is_only_legal_before_routing_commit() {
    let mut program = Program {
        stages: vec![stage(0, vec![], &[0], &[1])],
        values: vec![
            value(0, ValueRole::Input, 0, 64),
            value(1, ValueRole::Output, 1, 64),
        ],
        allocations: vec![allocation(0, 64, true), allocation(1, 64, false)],
        views: vec![],
        outputs: vec![ProgramOutput {
            key: "result",
            value: ValueId(1),
        }],
        actions: vec![Action::Preflight, Action::Fallback],
    };
    assert!(program.verify().is_ok());

    program.actions = vec![Action::Preflight, Action::RoutingCommit, Action::Fallback];
    assert_eq!(program.verify(), Err(Error::FallbackAfterCommit));
}
