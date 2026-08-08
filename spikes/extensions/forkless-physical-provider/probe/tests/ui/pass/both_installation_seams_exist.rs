// The two installation seams, on one request, spelled side by side.
//
// This file replaces the fixture that recorded their asymmetry as an *absence*:
// until 2026-08-08 a lowering authority installed through a public method and a
// physical provider had no method at all, and the missing one was the spike's
// second blocker. Both exist now, and the asymmetry that remains is a rule
// rather than a gap — an installed lowering registry *replaces* the governed
// one, because exactly one authority may say what an occurrence means, while an
// installed physical provider is *added to* the governed one, because several
// correct implementations of one verified region are alternatives. Neither rule
// generalizes to the other seam.
//
// Kept as a compiling contrast rather than deleted, because the retired fixture
// is what the `no method named with_physical_providers` measurement in ADR 0090
// and in this spike's earlier results rests on, and a reader arriving from
// either needs the file that says what replaced it.

use tiler_compiler::physical_provider::InstalledPhysicalProviders;
use tiler_compiler::session::{CompileRequest, InstalledCapabilities, NumericalContract};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{F32, InputKey, OutputKey, SemanticProgramBuilder};
use tiler_ir::shape::Shape;

fn main() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4]))
        .unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), input)
        .unwrap();
    let program = builder.build().unwrap();
    let targets = TargetRequest::new([TargetProfile::governed()]).unwrap();

    let _ = CompileRequest::new(
        &program,
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
        targets,
    )
    .with_capabilities(InstalledCapabilities::governed())
    .with_physical_providers(InstalledPhysicalProviders::governed());
}
