// The compiling contrast for
// `fail/scheduled_kernel_is_the_only_proposable_body.rs`.
//
// The body a proposal wraps is a `tiler_ir::schedule::ScheduledRegion`, fully
// public and fully constructible from outside the workspace, and
// `ImplementationProposal::scheduled_kernel` takes one by value. So the private
// `ProposalBody` next door is a restriction on *which* bodies may be proposed,
// not on whether an out-of-tree crate can express one — which is the difference
// between a reserved seam and a missing capability.

use acme_provider::{AcmeProvider, Specialization};
use tiler_compiler::physical_provider::{
    ImplementationProposal, PhysicalCostEstimate, TargetApplicability,
};
use tiler_compiler::target::{TargetProfile, TargetProfileKey};
use tiler_ir::schedule::ScheduledRegion;

fn propose(region: ScheduledRegion, key: TargetProfileKey) -> ImplementationProposal {
    ImplementationProposal::scheduled_kernel(
        region,
        TargetApplicability::for_targets([key]),
        PhysicalCostEstimate::structural(1, 256, 0),
    )
}

fn main() {
    // The provider crate compiles against the same public surface, which is the
    // half of this claim a signature alone cannot make.
    let _ = AcmeProvider::new(Specialization::WideWorkgroup);
    let _: fn(ScheduledRegion, TargetProfileKey) -> ImplementationProposal = propose;
    let _ = TargetProfile::governed().profile_key().clone();
}
