//! Stable procedural-macro generation probe for the selected nightly type.

use proc_macro::TokenStream;

/// Expands comma-separated integer extents into the canonical evidence type.
///
/// # Panics
///
/// Panics only if the macro's internally generated, fixed Rust token grammar
/// cannot be parsed. Invalid caller input is emitted as `compile_error!`.
#[proc_macro]
pub fn static_shape(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let extents = if source.trim().is_empty() {
        Vec::new()
    } else {
        let parsed = source
            .split(',')
            .map(str::trim)
            .map(|extent| {
                extent
                    .parse::<u64>()
                    .map_err(|_| format!("expected an unsigned integer extent, got `{extent}`"))
            })
            .collect::<Result<Vec<_>, _>>();
        match parsed {
            Ok(extents) => extents,
            Err(message) => {
                return format!("compile_error!({message:?});")
                    .parse()
                    .expect("fixed compile_error token grammar is valid");
            }
        }
    };
    let rank = extents.len();
    let extents = extents
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("::nightly_shape_api::StaticShape<{rank}, {{ [{extents}] }}>")
        .parse()
        .expect("generated static shape is valid Rust syntax")
}
