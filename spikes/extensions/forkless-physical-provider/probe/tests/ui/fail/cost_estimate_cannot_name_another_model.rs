// Reserved subject 2 of 5: cost-model attribution.
//
// `PhysicalCostEstimate::structural` is the only constructor an out-of-tree
// provider can reach, and it already attributes to the one governed key. The
// constructor that takes a key is private, so attributing an estimate to a
// model of the provider's own has no spelling at all — which is what stops two
// incomparable numbers being ranked against each other and a plan being
// selected on a comparison that measured nothing.
//
// The key itself is readable, as `GOVERNED_PHYSICAL_COST_MODEL_KEY`; reading it
// and writing it are different rights and only the first is granted.

use tiler_compiler::physical_provider::PhysicalCostEstimate;

fn main() {
    let _ = PhysicalCostEstimate::new("acme.cost.my-own.v1", 1, 4, 0);
}
