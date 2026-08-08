// Reserved subject 3 of 5: the region subject's semantic members.
//
// A member is a graph-local *authoring* coordinate — two spellings of one
// program number the same occurrence differently — so a provider that branched
// on one would put an authoring accident into its decision and, through a
// decline cause, into the explain trace. The count is a property of the region
// and does carry, which is why `covered_occurrences` is public and this is not.

use tiler_compiler::physical_provider::FrontierRegionSubject;

fn read(subject: &FrontierRegionSubject) {
    let _ = subject.semantic_members();
}

fn main() {
    let _: fn(&FrontierRegionSubject) = read;
}
