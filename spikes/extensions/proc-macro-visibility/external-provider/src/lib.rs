use visibility_api::Provider;

pub struct PredeclaredExternalOp;

impl Provider for PredeclaredExternalOp {
    fn key() -> &'static str {
        "example::predeclared@1"
    }
}
