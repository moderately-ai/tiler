// Reserved subject 5 of 5: three of the four proposal bodies.
//
// `ProposalBody` is an additive sum over four variants and stays crate-private,
// and `ImplementationProposal::new` — the constructor that would take one — is
// private with it. An installed provider proposes through
// `ImplementationProposal::scheduled_kernel` and by no other route, so the three
// bodies it may not propose are refused by having no spelling rather than by a
// runtime rejection it could mistake for a target verdict.
//
// This is the one of the five reserved subjects that no compile-fail doctest in
// `crates/tiler-compiler/src/physical_provider.rs` pins. The restriction is
// stated in prose on `scheduled_kernel`, and until this fixture existed nothing
// checked it. The compiling contrast is
// `pass/scheduled_kernel_is_publicly_proposable.rs`.

use tiler_compiler::frontier::ProposalBody;
use tiler_compiler::physical_provider::ImplementationProposal;

fn main() {
    let _ = std::any::type_name::<ProposalBody>();
    let _ = ImplementationProposal::new;
}
