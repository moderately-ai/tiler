// The complete vocabulary a `PhysicalImplementationProvider` implementation
// must name: the trait itself, the argument and return types of its two
// methods, the constructors those return types need, and the enumeration
// entry point that would consume it.
//
// Every one of them is `pub(crate)` inside `mod frontier`, which `lib.rs`
// declares private, so the module gate fires before item visibility is even
// consulted. This one file is the whole blocked list rather than nine files,
// because the blocker is a single module boundary and nine diagnostics naming
// it would say the same thing nine times.

use tiler_compiler::frontier::{
    FrontierRegionSubject, ImplementationContext, ImplementationProposal, PhysicalCostEstimate,
    PhysicalImplementationProvider, PhysicalProviderProvenance, ProposalBody, TargetApplicability,
    enumerate_frontier,
};

fn main() {
    let _ = (
        std::any::type_name::<FrontierRegionSubject>(),
        std::any::type_name::<ImplementationProposal>(),
        std::any::type_name::<PhysicalCostEstimate>(),
        std::any::type_name::<PhysicalProviderProvenance>(),
        std::any::type_name::<ProposalBody>(),
        std::any::type_name::<TargetApplicability>(),
    );
    let _: Option<&dyn PhysicalImplementationProvider> = None;
    let _: Option<&ImplementationContext<'_>> = None;
    let _ = enumerate_frontier;
}
