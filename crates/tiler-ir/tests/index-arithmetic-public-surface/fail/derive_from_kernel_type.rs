use tiler_ir::kernel::KernelType;
use tiler_ir::schedule::IndexArithmetic;

fn main() {
    let _ = IndexArithmetic::of(KernelType::Index);
}
