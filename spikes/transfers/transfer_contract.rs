//! Bounded verifier and transition model for transfer/lifetime contracts.
//!
//! This spike does not call a device API, measure transfer performance, or
//! model distributed scheduling. It makes the consumer-neutral invariants
//! executable with dependency-free tests.
//!
//! Run with:
//! `rustc --edition 2021 --test spikes/transfers/transfer_contract.rs -o /tmp/tiler-transfer-tests && /tmp/tiler-transfer-tests`

#![allow(dead_code)]

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SymbolicAffinity(&'static str);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AllocationRole(&'static str);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DependencyToken(&'static str);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ByteRange {
    start: u64,
    length: u64,
}

impl ByteRange {
    fn end(self) -> Option<u64> {
        self.start.checked_add(self.length)
    }

    fn contains(self, other: Self) -> bool {
        matches!((self.end(), other.end()), (Some(a), Some(b)) if self.start <= other.start && b <= a)
    }

    fn overlaps(self, other: Self) -> bool {
        matches!((self.end(), other.end()), (Some(a), Some(b)) if self.start < b && other.start < a)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AccessMode {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Endpoint {
    affinity: SymbolicAffinity,
    allocation: AllocationRole,
    allocation_bytes: u64,
    /// The allocation-relative byte range reachable through the logical view.
    view_range: ByteRange,
    /// The exact or conservative allocation-relative bytes touched by this stage.
    touched_range: ByteRange,
    encoding: &'static str,
    value_version: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CopyRoute {
    CpuToAccelerator,
    AcceleratorToCpu,
    PeerDirect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AliasProof {
    same_backing: bool,
    view_refines_semantics: bool,
    destination_accessible: bool,
    visibility_forwarded: bool,
    ownership_imported: bool,
}

impl AliasProof {
    fn complete(self) -> bool {
        self.same_backing
            && self.view_refines_semantics
            && self.destination_accessible
            && self.visibility_forwarded
            && self.ownership_imported
    }
}

/// The four conditions that discharge a recomputation's value preservation.
///
/// Each is decidable from stated identities and stated target facts; none is an
/// estimate and none needs a value model. The pair of identities is deliberately
/// `RegionImplementation` and not `ScheduledRegion`, because the latter excludes
/// the selected realization/provider that decides the delivered bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RecomputeProof {
    /// Equals the identity that produced, or would have produced, the reference.
    same_implementation_identity: bool,
    /// The two sites' stated delivered numerical realizations are equal.
    same_delivered_realization: bool,
    /// The selected plan carries the plan-deterministic guarantee at this scope.
    plan_deterministic: bool,
    /// Every operand carries a closure entry naming the same authoritative version.
    operands_closed: bool,
}

impl RecomputeProof {
    fn complete(self) -> bool {
        self.same_implementation_identity
            && self.same_delivered_realization
            && self.plan_deterministic
            && self.operands_closed
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mechanism {
    BitPreservingCopy(CopyRoute),
    LogicalMaterialize,
    AliasImport(AliasProof),
    PeerAccess(AliasProof),
    ManagedMigration,
    HostStaged {
        staging: AllocationRole,
        first_leg_complete: DependencyToken,
    },
    /// Re-derives the version rather than moving it, so it reads
    /// `TransferPlan::recompute_operands` and never `TransferPlan::source`.
    Recompute(RecomputeProof),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum RetainedRole {
    SourceAllocation,
    DestinationAllocation,
    SourceView,
    DestinationView,
    StagingAllocation,
    CommandObject,
    SynchronizationObject,
    ImportedBackingOwner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ConcurrentAccess {
    allocation: AllocationRole,
    range: ByteRange,
    mode: AccessMode,
    ordered_after_completion: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TransferPlan {
    /// For every mechanism but `Recompute`, the authoritative version being
    /// moved. For `Recompute` there is no source version: this endpoint
    /// describes the reference placement the re-derived value must agree with,
    /// which a plan that kept the producer's result would have delivered.
    source: Endpoint,
    destination: Endpoint,
    /// Read by a `Recompute` in place of a source; empty for every other mechanism.
    recompute_operands: Vec<Endpoint>,
    mechanism: Mechanism,
    source_ready: DependencyToken,
    waits_on_source: Vec<DependencyToken>,
    internal_waits: Vec<DependencyToken>,
    completion: DependencyToken,
    destination_waits: Vec<DependencyToken>,
    retained: BTreeSet<RetainedRole>,
    concurrent_accesses: Vec<ConcurrentAccess>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VerifyError {
    RangeOverflow,
    ViewOutsideAllocation,
    TouchOutsideView,
    EncodingConversionHiddenInTransfer,
    ValueVersionMismatch,
    MissingSourceDependency,
    MissingDestinationDependency,
    MissingStagedLegDependency,
    MissingRetention(RetainedRole),
    CopyAliasesBacking,
    IncompleteAliasProof,
    AliasUsesDistinctBacking,
    UnorderedHazard,
    RecomputeValuePreservationUnproved,
    RecomputeWithoutOperandClosure,
    RecomputeOverwritesItsOwnOperand,
}

fn required_retention(mechanism: Mechanism) -> BTreeSet<RetainedRole> {
    let mut required = BTreeSet::from([
        RetainedRole::SourceAllocation,
        RetainedRole::DestinationAllocation,
        RetainedRole::SourceView,
        RetainedRole::DestinationView,
        RetainedRole::SynchronizationObject,
    ]);
    match mechanism {
        Mechanism::HostStaged { .. } => {
            required.insert(RetainedRole::CommandObject);
            required.insert(RetainedRole::StagingAllocation);
        }
        Mechanism::AliasImport(_) | Mechanism::PeerAccess(_) => {
            required.insert(RetainedRole::ImportedBackingOwner);
        }
        // `SourceAllocation` and `SourceView` denote the recomputed region's
        // operands here; a recomputation retains no staging and no imported owner.
        _ => {
            required.insert(RetainedRole::CommandObject);
        }
    }
    required
}

fn verify_endpoint(endpoint: Endpoint) -> Result<(), VerifyError> {
    let allocation = ByteRange {
        start: 0,
        length: endpoint.allocation_bytes,
    };
    if endpoint.view_range.end().is_none() || endpoint.touched_range.end().is_none() {
        return Err(VerifyError::RangeOverflow);
    }
    if !allocation.contains(endpoint.view_range) {
        return Err(VerifyError::ViewOutsideAllocation);
    }
    if !endpoint.view_range.contains(endpoint.touched_range) {
        return Err(VerifyError::TouchOutsideView);
    }
    Ok(())
}

fn conflicts(transfer_mode: AccessMode, access: ConcurrentAccess) -> bool {
    !access.ordered_after_completion
        && (transfer_mode == AccessMode::Write || access.mode == AccessMode::Write)
}

fn verify(plan: &TransferPlan) -> Result<(), VerifyError> {
    verify_endpoint(plan.source)?;
    verify_endpoint(plan.destination)?;
    if plan.source.encoding != plan.destination.encoding {
        return Err(VerifyError::EncodingConversionHiddenInTransfer);
    }
    if plan.source.value_version != plan.destination.value_version {
        return Err(VerifyError::ValueVersionMismatch);
    }
    if !plan.waits_on_source.contains(&plan.source_ready) {
        return Err(VerifyError::MissingSourceDependency);
    }
    if !plan.destination_waits.contains(&plan.completion) {
        return Err(VerifyError::MissingDestinationDependency);
    }

    for role in required_retention(plan.mechanism) {
        if !plan.retained.contains(&role) {
            return Err(VerifyError::MissingRetention(role));
        }
    }

    match plan.mechanism {
        Mechanism::AliasImport(proof) | Mechanism::PeerAccess(proof) => {
            if plan.source.allocation != plan.destination.allocation {
                return Err(VerifyError::AliasUsesDistinctBacking);
            }
            if !proof.complete() {
                return Err(VerifyError::IncompleteAliasProof);
            }
        }
        Mechanism::HostStaged {
            first_leg_complete, ..
        } => {
            if !plan.internal_waits.contains(&first_leg_complete) {
                return Err(VerifyError::MissingStagedLegDependency);
            }
            if plan.source.allocation == plan.destination.allocation
                && plan
                    .source
                    .touched_range
                    .overlaps(plan.destination.touched_range)
            {
                return Err(VerifyError::CopyAliasesBacking);
            }
        }
        Mechanism::BitPreservingCopy(_) | Mechanism::LogicalMaterialize => {
            if plan.source.allocation == plan.destination.allocation
                && plan
                    .source
                    .touched_range
                    .overlaps(plan.destination.touched_range)
            {
                return Err(VerifyError::CopyAliasesBacking);
            }
        }
        Mechanism::Recompute(proof) => {
            if !proof.complete() {
                return Err(VerifyError::RecomputeValuePreservationUnproved);
            }
            if plan.recompute_operands.is_empty() {
                return Err(VerifyError::RecomputeWithoutOperandClosure);
            }
            for operand in &plan.recompute_operands {
                verify_endpoint(*operand)?;
                if operand.allocation == plan.destination.allocation
                    && operand.touched_range.overlaps(plan.destination.touched_range)
                {
                    return Err(VerifyError::RecomputeOverwritesItsOwnOperand);
                }
            }
        }
        Mechanism::ManagedMigration => {}
    }

    let destination_mode = match plan.mechanism {
        Mechanism::AliasImport(_) | Mechanism::PeerAccess(_) => AccessMode::Read,
        _ => AccessMode::Write,
    };
    // A recomputation reads its operands rather than a source version, so its
    // read-side hazards are over those endpoints.
    let read_endpoints: Vec<Endpoint> = match plan.mechanism {
        Mechanism::Recompute(_) => plan.recompute_operands.clone(),
        _ => vec![plan.source],
    };
    for access in &plan.concurrent_accesses {
        let read_hazard = read_endpoints.iter().any(|read| {
            access.allocation == read.allocation
                && access.range.overlaps(read.touched_range)
                && conflicts(AccessMode::Read, *access)
        });
        let destination_hazard = access.allocation == plan.destination.allocation
            && access.range.overlaps(plan.destination.touched_range)
            && conflicts(destination_mode, *access);
        if read_hazard || destination_hazard {
            return Err(VerifyError::UnorderedHazard);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Preflight,
    RoutingCommitted,
    ResourcesAcquired,
    EncodedFirstLeg,
    Submitted,
    TerminalSuccess,
    TerminalFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransitionError {
    IllegalTransition,
    FallbackAfterCommit,
    ReleaseBeforeTerminal,
}

#[derive(Debug)]
struct Execution {
    phase: Phase,
    fallback_authority: bool,
    cancellation_requested: bool,
    resources_retained: bool,
}

impl Execution {
    fn new() -> Self {
        Self {
            phase: Phase::Preflight,
            fallback_authority: true,
            cancellation_requested: false,
            resources_retained: false,
        }
    }

    fn commit(&mut self) -> Result<(), TransitionError> {
        if self.phase != Phase::Preflight || !self.fallback_authority {
            return Err(TransitionError::IllegalTransition);
        }
        self.fallback_authority = false;
        self.phase = Phase::RoutingCommitted;
        Ok(())
    }

    fn acquire(&mut self) -> Result<(), TransitionError> {
        if self.phase != Phase::RoutingCommitted {
            return Err(TransitionError::IllegalTransition);
        }
        self.resources_retained = true;
        self.phase = Phase::ResourcesAcquired;
        Ok(())
    }

    fn encode_first_leg(&mut self) -> Result<(), TransitionError> {
        if self.phase != Phase::ResourcesAcquired {
            return Err(TransitionError::IllegalTransition);
        }
        self.phase = Phase::EncodedFirstLeg;
        Ok(())
    }

    fn submit(&mut self) -> Result<(), TransitionError> {
        if !matches!(
            self.phase,
            Phase::ResourcesAcquired | Phase::EncodedFirstLeg
        ) {
            return Err(TransitionError::IllegalTransition);
        }
        self.phase = Phase::Submitted;
        Ok(())
    }

    fn request_cancellation(&mut self) {
        self.cancellation_requested = true;
    }

    fn try_fallback(&self) -> Result<(), TransitionError> {
        if self.fallback_authority {
            Ok(())
        } else {
            Err(TransitionError::FallbackAfterCommit)
        }
    }

    fn observe_terminal(&mut self, success: bool) -> Result<(), TransitionError> {
        if self.phase != Phase::Submitted {
            return Err(TransitionError::IllegalTransition);
        }
        self.phase = if success {
            Phase::TerminalSuccess
        } else {
            Phase::TerminalFailure
        };
        Ok(())
    }

    fn release(&mut self) -> Result<(), TransitionError> {
        if !matches!(self.phase, Phase::TerminalSuccess | Phase::TerminalFailure) {
            return Err(TransitionError::ReleaseBeforeTerminal);
        }
        self.resources_retained = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(
        affinity: &'static str,
        allocation: &'static str,
        start: u64,
        length: u64,
    ) -> Endpoint {
        Endpoint {
            affinity: SymbolicAffinity(affinity),
            allocation: AllocationRole(allocation),
            allocation_bytes: 4096,
            view_range: ByteRange { start, length },
            touched_range: ByteRange { start, length },
            encoding: "f32-le",
            value_version: 7,
        }
    }

    fn full_retention() -> BTreeSet<RetainedRole> {
        BTreeSet::from([
            RetainedRole::SourceAllocation,
            RetainedRole::DestinationAllocation,
            RetainedRole::SourceView,
            RetainedRole::DestinationView,
            RetainedRole::CommandObject,
            RetainedRole::SynchronizationObject,
        ])
    }

    fn copy_plan(
        route: CopyRoute,
        source_affinity: &'static str,
        destination_affinity: &'static str,
    ) -> TransferPlan {
        TransferPlan {
            source: endpoint(source_affinity, "src", 128, 1024),
            destination: endpoint(destination_affinity, "dst", 256, 1024),
            recompute_operands: vec![],
            mechanism: Mechanism::BitPreservingCopy(route),
            source_ready: DependencyToken("producer"),
            waits_on_source: vec![DependencyToken("producer")],
            internal_waits: vec![],
            completion: DependencyToken("transfer"),
            destination_waits: vec![DependencyToken("transfer")],
            retained: full_retention(),
            concurrent_accesses: vec![],
        }
    }

    #[test]
    fn verifies_cpu_to_accelerator() {
        assert_eq!(
            verify(&copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0")),
            Ok(())
        );
    }

    #[test]
    fn verifies_accelerator_to_cpu() {
        assert_eq!(
            verify(&copy_plan(CopyRoute::AcceleratorToCpu, "gpu0", "cpu")),
            Ok(())
        );
    }

    #[test]
    fn verifies_same_device_materialization() {
        let mut plan = copy_plan(CopyRoute::CpuToAccelerator, "gpu0", "gpu0");
        plan.mechanism = Mechanism::LogicalMaterialize;
        assert_eq!(verify(&plan), Ok(()));
    }

    #[test]
    fn verifies_peer_direct_copy() {
        assert_eq!(
            verify(&copy_plan(CopyRoute::PeerDirect, "gpu0", "gpu1")),
            Ok(())
        );
    }

    #[test]
    fn verifies_shared_backing_alias() {
        let mut plan = copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0");
        plan.destination.allocation = plan.source.allocation;
        plan.destination.view_range = plan.source.view_range;
        plan.destination.touched_range = plan.source.touched_range;
        plan.mechanism = Mechanism::AliasImport(AliasProof {
            same_backing: true,
            view_refines_semantics: true,
            destination_accessible: true,
            visibility_forwarded: true,
            ownership_imported: true,
        });
        plan.retained.insert(RetainedRole::ImportedBackingOwner);
        assert_eq!(verify(&plan), Ok(()));
    }

    #[test]
    fn verifies_peer_access_without_copy() {
        let mut plan = copy_plan(CopyRoute::PeerDirect, "gpu0", "gpu1");
        plan.destination.allocation = plan.source.allocation;
        plan.destination.view_range = plan.source.view_range;
        plan.destination.touched_range = plan.source.touched_range;
        plan.mechanism = Mechanism::PeerAccess(AliasProof {
            same_backing: true,
            view_refines_semantics: true,
            destination_accessible: true,
            visibility_forwarded: true,
            ownership_imported: true,
        });
        plan.retained.insert(RetainedRole::ImportedBackingOwner);
        assert_eq!(verify(&plan), Ok(()));
    }

    #[test]
    fn verifies_managed_migration_without_new_backing() {
        let mut plan = copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0");
        plan.destination.allocation = plan.source.allocation;
        plan.destination.view_range = plan.source.view_range;
        plan.destination.touched_range = plan.source.touched_range;
        plan.mechanism = Mechanism::ManagedMigration;
        assert_eq!(verify(&plan), Ok(()));
    }

    #[test]
    fn verifies_host_staged_copy_with_two_leg_dependency() {
        let mut plan = copy_plan(CopyRoute::PeerDirect, "gpu0", "gpu1");
        plan.mechanism = Mechanism::HostStaged {
            staging: AllocationRole("pinned-host-staging"),
            first_leg_complete: DependencyToken("download-complete"),
        };
        plan.internal_waits
            .push(DependencyToken("download-complete"));
        plan.retained.insert(RetainedRole::StagingAllocation);
        assert_eq!(verify(&plan), Ok(()));
    }

    fn complete_recompute_proof() -> RecomputeProof {
        RecomputeProof {
            same_implementation_identity: true,
            same_delivered_realization: true,
            plan_deterministic: true,
            operands_closed: true,
        }
    }

    /// A recomputation at `gpu0` standing in for a version produced at `cpu`.
    ///
    /// The `source` endpoint is the reference placement, not a version this plan
    /// reads: the stage reads `recompute_operands` instead.
    fn recompute_plan(proof: RecomputeProof) -> TransferPlan {
        let mut plan = copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0");
        plan.mechanism = Mechanism::Recompute(proof);
        plan.recompute_operands = vec![endpoint("gpu0", "operand", 0, 512)];
        plan
    }

    #[test]
    fn verifies_recompute_carrying_a_complete_value_preservation_record() {
        assert_eq!(verify(&recompute_plan(complete_recompute_proof())), Ok(()));
    }

    #[test]
    fn rejects_recompute_missing_any_one_of_the_four_conditions() {
        // Each condition is independently load-bearing, so each is asserted:
        // dropping any one leaves a recomputation whose delivered bits are
        // unproved, and all four must therefore fail closed the same way.
        let mut without_identity = complete_recompute_proof();
        without_identity.same_implementation_identity = false;
        let mut without_realization = complete_recompute_proof();
        without_realization.same_delivered_realization = false;
        let mut without_determinism = complete_recompute_proof();
        without_determinism.plan_deterministic = false;
        let mut without_closure = complete_recompute_proof();
        without_closure.operands_closed = false;

        for proof in [
            without_identity,
            without_realization,
            without_determinism,
            without_closure,
        ] {
            assert_eq!(
                verify(&recompute_plan(proof)),
                Err(VerifyError::RecomputeValuePreservationUnproved)
            );
        }
    }

    #[test]
    fn rejects_recompute_that_names_no_operand_to_close_over() {
        let mut plan = recompute_plan(complete_recompute_proof());
        plan.recompute_operands.clear();
        assert_eq!(
            verify(&plan),
            Err(VerifyError::RecomputeWithoutOperandClosure)
        );
    }

    #[test]
    fn rejects_recompute_whose_destination_overwrites_an_operand_it_reads() {
        let mut plan = recompute_plan(complete_recompute_proof());
        plan.recompute_operands = vec![Endpoint {
            allocation: plan.destination.allocation,
            ..plan.destination
        }];
        assert_eq!(
            verify(&plan),
            Err(VerifyError::RecomputeOverwritesItsOwnOperand)
        );
    }

    #[test]
    fn rejects_hidden_dtype_or_encoding_conversion() {
        let mut plan = copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0");
        plan.destination.encoding = "f16-le";
        assert_eq!(
            verify(&plan),
            Err(VerifyError::EncodingConversionHiddenInTransfer)
        );
    }

    #[test]
    fn rejects_out_of_bounds_logical_view() {
        let mut plan = copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0");
        plan.source.view_range = ByteRange {
            start: 4000,
            length: 200,
        };
        plan.source.touched_range = plan.source.view_range;
        assert_eq!(verify(&plan), Err(VerifyError::ViewOutsideAllocation));
    }

    #[test]
    fn rejects_missing_two_sided_dependencies() {
        let mut source_missing = copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0");
        source_missing.waits_on_source.clear();
        assert_eq!(
            verify(&source_missing),
            Err(VerifyError::MissingSourceDependency)
        );

        let mut consumer_missing = copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0");
        consumer_missing.destination_waits.clear();
        assert_eq!(
            verify(&consumer_missing),
            Err(VerifyError::MissingDestinationDependency)
        );
    }

    #[test]
    fn rejects_incomplete_no_copy_proof() {
        let mut plan = copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0");
        plan.destination.allocation = plan.source.allocation;
        plan.mechanism = Mechanism::AliasImport(AliasProof {
            same_backing: true,
            view_refines_semantics: true,
            destination_accessible: true,
            visibility_forwarded: false,
            ownership_imported: true,
        });
        plan.retained.insert(RetainedRole::ImportedBackingOwner);
        assert_eq!(verify(&plan), Err(VerifyError::IncompleteAliasProof));
    }

    #[test]
    fn rejects_missing_staging_retention_and_leg_order() {
        let mut plan = copy_plan(CopyRoute::PeerDirect, "gpu0", "gpu1");
        plan.mechanism = Mechanism::HostStaged {
            staging: AllocationRole("staging"),
            first_leg_complete: DependencyToken("first-leg"),
        };
        assert_eq!(
            verify(&plan),
            Err(VerifyError::MissingRetention(
                RetainedRole::StagingAllocation
            ))
        );
        plan.retained.insert(RetainedRole::StagingAllocation);
        assert_eq!(verify(&plan), Err(VerifyError::MissingStagedLegDependency));
    }

    #[test]
    fn rejects_unordered_write_hazard() {
        let mut plan = copy_plan(CopyRoute::CpuToAccelerator, "gpu0", "gpu0");
        plan.mechanism = Mechanism::LogicalMaterialize;
        plan.concurrent_accesses.push(ConcurrentAccess {
            allocation: plan.source.allocation,
            range: ByteRange {
                start: 256,
                length: 64,
            },
            mode: AccessMode::Write,
            ordered_after_completion: false,
        });
        assert_eq!(verify(&plan), Err(VerifyError::UnorderedHazard));
    }

    #[test]
    fn rejects_overlapping_copy_without_explicit_semantics() {
        let mut plan = copy_plan(CopyRoute::CpuToAccelerator, "cpu", "gpu0");
        plan.destination.allocation = plan.source.allocation;
        assert_eq!(verify(&plan), Err(VerifyError::CopyAliasesBacking));
    }

    #[test]
    fn cancellation_is_not_completion_or_fallback() {
        let mut execution = Execution::new();
        execution.commit().unwrap();
        execution.acquire().unwrap();
        execution.submit().unwrap();
        execution.request_cancellation();
        assert_eq!(
            execution.try_fallback(),
            Err(TransitionError::FallbackAfterCommit)
        );
        assert_eq!(
            execution.release(),
            Err(TransitionError::ReleaseBeforeTerminal)
        );
        assert!(execution.resources_retained);
        execution.observe_terminal(false).unwrap();
        execution.release().unwrap();
        assert!(!execution.resources_retained);
    }

    #[test]
    fn staged_failure_retains_everything_until_terminal_observation() {
        let mut execution = Execution::new();
        execution.commit().unwrap();
        execution.acquire().unwrap();
        execution.encode_first_leg().unwrap();
        execution.submit().unwrap();
        assert_eq!(
            execution.release(),
            Err(TransitionError::ReleaseBeforeTerminal)
        );
        assert!(execution.resources_retained);
        execution.observe_terminal(false).unwrap();
        execution.release().unwrap();
        assert!(!execution.resources_retained);
    }
}
