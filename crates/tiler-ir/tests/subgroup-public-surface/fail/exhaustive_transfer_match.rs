use tiler_ir::schedule::SubgroupTransfer;

fn classify(transfer: SubgroupTransfer) -> &'static str {
    match transfer {
        SubgroupTransfer::InRangeXorShuffle => "in-range-xor-shuffle",
    }
}

fn main() {
    let _ = classify;
}
