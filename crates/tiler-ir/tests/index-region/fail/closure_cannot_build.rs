use tiler_ir::index::{FrozenScalarRegistry, IndexBuildError, IndexRegionBuilder};

// The `build_with` closure only borrows the draft, so it cannot consume it
// through `build()` to obtain the opaque verified product. The verified region
// is reachable solely through the checked convenience return value.
fn smuggle(registry: FrozenScalarRegistry) {
    let _ = IndexRegionBuilder::build_with(registry, |builder| -> Result<(), IndexBuildError> {
        let _verified = builder.build();
        Ok(())
    });
}

fn main() {
    let _ = smuggle;
}
