//! Out-of-crate evidence for the growing subgroup-transfer vocabulary.

use tiler_ir::schedule::SubgroupTransfer;

/// An external consumer may classify the transfer it knows while retaining the
/// wildcard `#[non_exhaustive]` requires for future variants. The companion UI
/// test proves an exhaustive external match is refused, while the defining
/// crate's private exhaustive matches continue to guard the identity tag and
/// subject-construction rule independently.
#[test]
fn known_transfer_is_constructible_and_partial_classification_is_future_proof() {
    let transfer = SubgroupTransfer::InRangeXorShuffle;
    let key = match transfer {
        SubgroupTransfer::InRangeXorShuffle => transfer.key(),
        _ => "future-subgroup-transfer",
    };
    assert_eq!(key, "in-range-xor-shuffle");
}
