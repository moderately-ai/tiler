use visibility_api::Provider;

pub struct BuiltinAdd;

impl Provider for BuiltinAdd {
    fn key() -> &'static str {
        "tiler::add@1"
    }
}
