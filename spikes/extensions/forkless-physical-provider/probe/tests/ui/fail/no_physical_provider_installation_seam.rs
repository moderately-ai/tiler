// The second, independent blocker: even a fully public
// `PhysicalImplementationProvider` would not compose, because nothing installs
// one.
//
// ADR 0078 item 4 named exactly this asymmetry for *lowering* providers —
// everything needed to build a registry was public and nothing could install
// one — and closed it with `CompileRequest::with_capabilities`. The same
// asymmetry is still open for physical providers: the provider array is a
// hardcoded one-element literal at
// `crates/tiler-compiler/src/pipeline/planning.rs:171`, and the internal
// `CompilationRequest` (`request.rs:542`) carries no provider field for a
// public method to write into.
//
// The compiling contrast is `pass/lowering_installation_seam_exists.rs`; the
// two files differ in one method name.

use tiler_compiler::session::{CompileRequest, NumericalContract};
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
    .with_physical_providers([&acme_provider::identity()]);
}
