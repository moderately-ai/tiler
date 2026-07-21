//! Direct rustc probes for the selected and rejected feature combinations.

use std::path::{Path, PathBuf};
use std::process::Command;

fn compile_probe(name: &str) -> std::process::Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("conformance crate is inside the spike workspace");
    let output = std::env::temp_dir().join(format!(
        "tiler-nightly-shape-probe-{}-{name}.rlib",
        std::process::id()
    ));
    Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("--crate-name")
        .arg(format!("probe_{name}"))
        .arg("--crate-type=lib")
        .arg(root.join("probes").join(format!("{name}.rs")))
        .arg("-o")
        .arg(output)
        .output()
        .expect("rustc probe must launch")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn selected_form_needs_exactly_the_two_governed_features() {
    let selected = compile_probe("selected");
    assert!(selected.status.success(), "{}", stderr(&selected));

    let without_min_adt = compile_probe("without_min_adt");
    assert!(!without_min_adt.status.success());
    assert!(stderr(&without_min_adt).contains("adt_const_params"));

    let without_dependent = compile_probe("without_dependent_types");
    assert!(!without_dependent.status.success());
    let dependent_error = stderr(&without_dependent);
    assert!(
        dependent_error.contains("E0770")
            && dependent_error.contains("must not depend on other generic parameters")
    );
}

#[test]
fn borrowed_slice_remains_an_isolated_nonselected_comparison() {
    let borrowed = compile_probe("borrowed_slice");
    assert!(borrowed.status.success(), "{}", stderr(&borrowed));

    let api = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("api/src/lib.rs");
    let source = std::fs::read_to_string(api).unwrap();
    assert!(!source.contains("unsized_const_params"));
    assert!(!source.contains("#![feature(adt_const_params)]"));
    assert!(!source.contains("generic_const_args"));
    assert!(!source.contains("generic_const_exprs"));
}
