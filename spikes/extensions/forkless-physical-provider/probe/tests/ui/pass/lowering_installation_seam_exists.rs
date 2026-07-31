// The compiling contrast for `fail/no_physical_provider_installation_seam.rs`.
//
// A caller-supplied *lowering* authority installs through a public method on
// the same request type, so "no installation path exists" is false in general
// and true only of physical providers. Without this file the failing case would
// be consistent with a compile boundary that installs nothing at all, which is
// a different — and much weaker — finding.

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
        NumericalContract::FlushSubnormalsToZeroF32,
        targets,
    )
    .with_capabilities(InstalledCapabilities::governed());
}
