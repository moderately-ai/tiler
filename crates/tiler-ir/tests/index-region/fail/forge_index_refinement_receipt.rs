use tiler_ir::index::{IndexRefinementReceipt, IndexRefinementReceiptIdentity};

fn main() {
    let _forged_identity = IndexRefinementReceiptIdentity(Vec::new().into_boxed_slice());
    let _forged_receipt = IndexRefinementReceipt {};
}
