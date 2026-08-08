// Reserved subject 4 of 5: the enumeration itself.
//
// Installing a provider is not the same right as running the frontier. A caller
// that could call `enumerate_frontier` directly would admit a body outside a
// compilation's own authorities — outside its verified request, its target
// profile, and its numerical contract — and the re-verification the seam is
// built on happens inside that call rather than around it.
//
// `mod frontier` stays private for exactly this reason even though the
// vocabulary it defines is publicly re-exported through `physical_provider`, so
// this diagnostic is a module gate rather than an item one. The contrast is
// `pass/provider_vocabulary_is_publicly_reachable.rs`, which names every item a
// provider implementation needs and compiles.

use tiler_compiler::frontier::enumerate_frontier;

fn main() {
    let _ = enumerate_frontier;
}
