//! Downstream compile contract for the facade's macro re-export.
//!
//! These cases compile as a separate out-of-tree crate, which is what makes
//! them evidence: an in-crate test resolves `crate::__private` and so cannot
//! tell a working expansion from one whose absolute path is wrong.
//!
//! What that separate crate does *not* isolate is the manifest. `trybuild`
//! copies the crate under test's `[dependencies]` into the generated project,
//! so `tiler-macros` is declared there too (inspect
//! `target/tests/trybuild/tiler/Cargo.toml` after a run). No `trybuild` case
//! can remove it, because the facade genuinely depends on it. The fixtures
//! therefore prove what they can — that nothing a consumer *writes* or a macro
//! *emits* names anything but `tiler` — while the resolved-graph invariant is
//! `dependency_direction`'s job.

//! The `bind_*` cases carry a second claim beyond compiling: each defines its
//! own adapter over its own value type, in a crate that depends on `tiler`
//! alone. That is what "an arbitrary external consumer supplies the adapter
//! without a facade change or a global registration" means, checked rather than
//! asserted. Their `FACTS` constants are byte-identical to what
//! `tiler_macros::binding` emits, and the macro crate's tests read these files
//! to keep the two ends from drifting apart.

use std::path::Path;
use std::process::{Command, Output};

#[test]
fn facade_reexport_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/facade/pass/*.rs");
    cases.compile_fail("tests/facade/fail/*.rs");
}

/// A consumer Cargo binding named `core` cannot retarget Tiler's diagnostics.
///
/// The first compile proves the adversarial binding is live: its no-op
/// `compile_error` procedural macro consumes the exact direct path the producer
/// used before the workspace unsafe-site inventory closed this identity hole.
/// The matching and nonmatching compiles then prove the facade-owned builtin
/// retains ADR 0053's target-gated behavior under that same binding, and the
/// final compile reaches the real producer's unconditional refusal.
#[test]
fn a_consumer_core_dependency_cannot_replace_the_facade_diagnostic_builtin() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the facade crate sits two levels below the workspace root");
    let root =
        std::env::temp_dir().join(format!("tiler-facade-core-binding-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let evil = root.join("evil-core");
    let consumer = root.join("consumer");
    std::fs::create_dir_all(evil.join("src")).expect("the evil proc-macro source is creatable");
    std::fs::create_dir_all(consumer.join("src")).expect("the consumer source is creatable");
    std::fs::write(
        evil.join("Cargo.toml"),
        "[package]\nname = \"evil-core\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
         [lib]\nproc-macro = true\n",
    )
    .expect("the evil proc-macro manifest is writable");
    std::fs::write(
        evil.join("src/lib.rs"),
        "extern crate proc_macro;\nuse proc_macro::TokenStream;\n\
         #[proc_macro]\npub fn compile_error(_: TokenStream) -> TokenStream {\n\
             \"()\".parse().expect(\"unit is valid Rust\")\n}\n",
    )
    .expect("the evil proc-macro source is writable");
    std::fs::write(
        consumer.join("Cargo.toml"),
        format!(
            "[package]\nname = \"tiler-core-binding-consumer\"\nversion = \"0.0.0\"\n\
             edition = \"2024\"\n[dependencies]\n\
             tiler = {{ path = {:?} }}\n\
             core = {{ package = \"evil-core\", path = {:?} }}\n",
            Path::new(env!("CARGO_MANIFEST_DIR")),
            evil,
        ),
    )
    .expect("the consumer manifest is writable");

    let check = |source: &str| -> Output {
        std::fs::write(consumer.join("src/main.rs"), source)
            .expect("the consumer subject is writable");
        Command::new("cargo")
            .args(["check", "--offline", "--manifest-path"])
            .arg(consumer.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", root.join("target"))
            .current_dir(workspace)
            .output()
            .expect("the downstream Cargo check runs")
    };

    let direct = check("fn main() { ::core::compile_error!(\"swallowed direct path\"); }\n");
    assert!(
        direct.status.success(),
        "the renamed dependency must establish the old bypass subject:\n{}",
        String::from_utf8_lossy(&direct.stderr),
    );

    let matching = check(concat!(
        "fn main() {\n",
        "    #[cfg(target_os = \"macos\")]\n",
        "    ::tiler::__private::__tiler_compile_error!(\"retained diagnostic survives consumer core rebinding\");\n",
        "}\n",
    ));
    let matching_stderr = String::from_utf8_lossy(&matching.stderr);
    assert!(
        !matching.status.success(),
        "the matching diagnostic compiled"
    );
    assert!(
        matching_stderr.contains("retained diagnostic survives consumer core rebinding"),
        "the matching build failed on something other than the retained diagnostic:\n{matching_stderr}",
    );

    let nonmatching = check(concat!(
        "fn main() {\n",
        "    #[cfg(target_os = \"ios\")]\n",
        "    ::tiler::__private::__tiler_compile_error!(\"nonmatching diagnostic\");\n",
        "}\n",
    ));
    assert!(
        nonmatching.status.success(),
        "a nonmatching diagnostic must be removed by cfg:\n{}",
        String::from_utf8_lossy(&nonmatching.stderr),
    );

    let unconditional = check("fn main() { let _ = tiler::tensor! {}; }\n");
    let unconditional_stderr = String::from_utf8_lossy(&unconditional.stderr);
    assert!(
        !unconditional.status.success(),
        "the malformed region compiled"
    );
    assert!(
        unconditional_stderr.contains("`tiler::tensor!` was given no region"),
        "the producer refusal was replaced by the consumer's `core` binding:\n{unconditional_stderr}",
    );

    std::fs::remove_dir_all(&root).expect("the private downstream fixture is removable");
}
