use tiler_ir::semantic::{
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer,
};

struct Probe;

impl OperationInferencer for Probe {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        _outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let _ = request.extent_sources();
        Ok(())
    }
}

fn main() {}
