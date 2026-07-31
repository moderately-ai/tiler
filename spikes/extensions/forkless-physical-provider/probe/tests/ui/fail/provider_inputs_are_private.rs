// Making `frontier` public would not be sufficient, so its blockage is not the
// whole finding. A provider must also *read* its context and *build* a body
// that binds to the region subject, and each of those lives behind its own
// private module:
//
//   request::VerifiedTargetRequest  what `ImplementationContext::request` returns
//   region::SemanticMemberId        what `FrontierRegionSubject` is made of
//   physical::pointwise_region      how the governed provider builds a body
//   pipeline::compile               the internal path that installs providers
//
// Four separate module gates, listed together because they are four instances
// of one fact: the provider seam's transitive closure is private, not just its
// entry point.

use tiler_compiler::physical::pointwise_region;
use tiler_compiler::pipeline::compile;
use tiler_compiler::region::SemanticMemberId;
use tiler_compiler::request::VerifiedTargetRequest;

fn main() {
    let _ = std::any::type_name::<SemanticMemberId>();
    let _ = std::any::type_name::<VerifiedTargetRequest>();
    let _ = pointwise_region;
    let _ = compile;
}
