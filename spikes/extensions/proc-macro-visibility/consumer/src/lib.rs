use visibility_api::Provider;

pub struct ConsumerLocalOp;

impl Provider for ConsumerLocalOp {
    fn key() -> &'static str {
        "consumer::local@1"
    }
}

pub const BUILTIN_KEY: &str = visibility_macro::builtin_provider_key!();
pub const PREDECLARED_EXTERNAL_KEY: &str = visibility_macro::predeclared_external_provider_key!();
pub const MACRO_PACKAGE: &str = visibility_macro::macro_package!();
pub const CALLER_PACKAGE: &str = visibility_macro::caller_package!();
pub const CONSUMER_TOKENS: &str = visibility_macro::visible_consumer_tokens!(ConsumerLocalOp);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_macro_sees_only_its_linked_provider_graph() {
        assert_eq!(BUILTIN_KEY, "tiler::add@1");
        assert_eq!(PREDECLARED_EXTERNAL_KEY, "example::predeclared@1");
        assert_eq!(MACRO_PACKAGE, "visibility-macro");
        assert_eq!(CALLER_PACKAGE, "visibility-consumer");
        assert_eq!(CONSUMER_TOKENS, "ConsumerLocalOp");
        assert_eq!(ConsumerLocalOp::key(), "consumer::local@1");
    }
}
