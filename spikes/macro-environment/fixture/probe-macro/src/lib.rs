use proc_macro::{Literal, TokenStream, TokenTree};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

const OBSERVED: &[&str] = &[
    "HOST",
    "TARGET",
    "CARGO_BUILD_TARGET",
    "CARGO_CFG_TARGET_ARCH",
    "CARGO_CFG_TARGET_OS",
    "CARGO_CFG_TARGET_ENV",
    "CARGO_CFG_TARGET_FAMILY",
    "CARGO_MANIFEST_DIR",
    "CARGO_PKG_NAME",
    "OUT_DIR",
    "PROFILE",
    "OPT_LEVEL",
    "DEBUG",
    "RUSTC",
    "SDKROOT",
    "MACOSX_DEPLOYMENT_TARGET",
    "IPHONEOS_DEPLOYMENT_TARGET",
];

#[proc_macro]
pub fn probe(input: TokenStream) -> TokenStream {
    let selection = input.to_string();
    let fingerprint =
        std::env::var("TILER_TOOLCHAIN_FINGERPRINT").unwrap_or_else(|_| "unset".into());
    let cache_root = std::env::var_os("TILER_PROBE_CACHE").map(PathBuf::from);
    let cache_state = cache_root
        .as_ref()
        .map(|root| {
            fs::create_dir_all(root).unwrap();
            let key = root.join(format!(
                "{}-{}",
                sanitize(&selection),
                sanitize(&fingerprint)
            ));
            if key.exists() {
                "hit"
            } else {
                fs::write(key, b"complete").unwrap();
                "miss"
            }
        })
        .unwrap_or("disabled");

    if let Some(path) = std::env::var_os("TILER_TRACE_PATH") {
        let mut trace = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        write!(
            trace,
            "version=1\tselection_hex={}\tfingerprint_hex={}\tcache={cache_state}",
            hex(&selection),
            hex(&fingerprint)
        )
        .unwrap();
        for name in OBSERVED {
            match std::env::var(name) {
                Ok(value) => write!(trace, "\tenv.{name}.hex={}", hex(&value)).unwrap(),
                Err(std::env::VarError::NotPresent) => {
                    write!(trace, "\tenv.{name}.absent=1").unwrap()
                }
                Err(std::env::VarError::NotUnicode(_)) => {
                    write!(trace, "\tenv.{name}.nonunicode=1").unwrap()
                }
            }
        }
        writeln!(trace).unwrap();
    }

    TokenStream::from(TokenTree::Literal(Literal::string(&selection)))
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn hex(value: &str) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(value.len() * 2);
    for byte in value.as_bytes() {
        result.push(DIGITS[(byte >> 4) as usize] as char);
        result.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    result
}
