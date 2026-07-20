use proc_macro::{Literal, TokenStream, TokenTree};
use visibility_api::Provider;

fn string_literal(value: &str) -> TokenStream {
    TokenStream::from(TokenTree::Literal(Literal::string(value)))
}

#[proc_macro]
pub fn builtin_provider_key(_input: TokenStream) -> TokenStream {
    string_literal(visibility_builtin_provider::BuiltinAdd::key())
}

#[cfg(feature = "external-provider")]
#[proc_macro]
pub fn predeclared_external_provider_key(_input: TokenStream) -> TokenStream {
    string_literal(visibility_external_provider::PredeclaredExternalOp::key())
}

#[proc_macro]
pub fn macro_package(_input: TokenStream) -> TokenStream {
    // Evaluated while compiling the host proc-macro crate.
    string_literal(env!("CARGO_PKG_NAME"))
}

#[proc_macro]
pub fn caller_package(_input: TokenStream) -> TokenStream {
    // Emitted tokens are evaluated later in the consuming target crate.
    "env!(\"CARGO_PKG_NAME\")".parse().unwrap()
}

#[proc_macro]
pub fn visible_consumer_tokens(input: TokenStream) -> TokenStream {
    // The macro can inspect spelling, but cannot resolve this path to a
    // consumer-local Rust type or invoke its Provider implementation.
    string_literal(&input.to_string())
}
