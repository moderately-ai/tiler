use super::support::{region_attributions, semantic};
use super::*;

/// **Obligation 4 of the minimum correct physical realization profile, end to
/// end: no region a legal cover placed is answered with silence.**
///
/// Every region subject the compile path enumerates either admits at least one
/// implementation or carries a typed decline naming which region-vocabulary
/// wall it hit. Before the provider read the cover region subject, fourteen of
/// this program's seventeen subjects reached the final `else` of a member-set
/// comparison and returned an empty offer, so complete-plan selection saw an
/// unimplemented region with nothing said about it.
///
/// The implication is what this asserts, and it is a check that can say no:
/// deleting the `Err(wall)` arm of `GovernedPhysicalProvider::propose` restores
/// the empty offer and fourteen subjects fail it at once.
#[test]
fn every_cover_region_receives_a_proposal_or_a_typed_decline() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let attributions = region_attributions(&product.targets[0].explain);

    assert_eq!(
        attributions.len(),
        17,
        "the governed five-operation program covers seventeen distinct region subjects",
    );
    for (key, attribution) in &attributions {
        assert!(
            attribution.admitted > 0 || attribution.declined_baseline.is_some(),
            "region {key} ({}) was answered with silence",
            attribution.role,
        );
    }
    // The three the vocabulary spells are answered with implementations, and
    // every other one with a wall. Asserting both halves is what stops a
    // regression that declined *everything* from passing the implication above.
    let mut answered: Vec<&str> = attributions
        .values()
        .filter(|attribution| attribution.admitted > 0)
        .map(|attribution| attribution.role.as_str())
        .collect();
    answered.sort_unstable();
    assert_eq!(answered, ["pointwise", "reduction", "whole-program"]);
    let walls: BTreeMap<&str, usize> = attributions
        .values()
        .filter_map(|attribution| attribution.declined_baseline.as_deref())
        .fold(BTreeMap::new(), |mut counts, reason| {
            *counts.entry(reason).or_insert(0) += 1;
            counts
        });
    assert_eq!(
        walls,
        BTreeMap::from([
            // Five regions covering the reduction together with part, but not
            // all, of its four-occurrence prologue.
            ("region-partial-fused-program", 5),
            // Nine regions covering a proper part of that prologue.
            ("region-partial-coverage", 9),
        ]),
        "the fourteen declines no longer name the walls they hit",
    );
}
