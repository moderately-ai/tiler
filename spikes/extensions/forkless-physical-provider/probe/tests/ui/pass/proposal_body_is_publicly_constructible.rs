// The compiling contrast for `fail/frontier_provider_vocabulary_is_private.rs`.
//
// What `ProposalBody::ScheduledKernel` wraps is a `tiler_ir::schedule::
// ScheduledRegion`, and that is fully public and fully constructible from
// outside the workspace — so the private vocabulary is a wrapper problem, not a
// missing capability. A third party can build a correct body today; it has
// nowhere to hand it to.
//
// This matters for the composition contract: the seam to be designed is
// registration and re-verification, not a new way to express an implementation.

use acme_provider::PointwiseSubject;
use tiler_ir::schedule::ScheduledRegionBuilder;

fn main() {
    let region = acme_provider::specialized_region(PointwiseSubject::spike_default());
    let verified = ScheduledRegionBuilder::from_region(region)
        .build()
        .expect("the specialized body verifies");
    assert_eq!(
        verified.region().schedule.threads_per_workgroup,
        acme_provider::SPECIALIZED_THREADS_PER_WORKGROUP,
    );
}
