//! What the retained record can be held to without a device.
//!
//! Every test here is device-free by construction: the record is a checked-in
//! file and the comparison is a pure function of it and one observed row, so a
//! host that can never measure still holds the reader, the filter, and every
//! field of the row check.

use super::{RECORD_DIRECTORY, RowComparison, RowField, compare, direct_digests, environment_row};
use crate::measurement::MeasurementBoundary;

/// A boundary equal to the retained record's own row, field for field.
///
/// Built by hand rather than observed, because the point of the perturbations
/// below is that each one is the *only* difference; a row read off this host
/// would already differ in two fields and no perturbation could then be isolated.
fn the_records_own_row() -> MeasurementBoundary {
    MeasurementBoundary {
        os_family: "macos".to_owned(),
        os_version: "27.0".to_owned(),
        os_build: "26A5388g".to_owned(),
        architecture: "arm64".to_owned(),
        device_name: "Apple M4 Max".to_owned(),
        gpu_family: "Apple9".to_owned(),
        metal_compiler: "Apple metal version 32023.883 (metalfe-32023.883)".to_owned(),
        metallib_linker: "AIR-LLD 32023.883 (metalfe-32023.883)".to_owned(),
        sdk: "macosx 26.5 build 25F70".to_owned(),
        profile_key: "tiler.metal.macos-apple9.msl4-0.f32-bf16.v1".to_owned(),
        aot_target: "air64-apple-macos26.0".to_owned(),
        language_standard: "metal4.0".to_owned(),
        metallib_bytes: 0,
    }
}

/// The record reader opens both files and finds the shape it needs.
///
/// **The population is counted rather than sampled.** A reader whose filter had
/// stopped matching would return an empty vector, and every equality below would
/// then hold vacuously — the failure mode this repository has recorded more than
/// once. `direct_digests` refuses an empty result for the same reason, and this
/// states the number a reader can check against the record by eye.
#[test]
fn the_retained_record_reads_and_carries_the_six_correctness_cells() {
    assert!(
        RECORD_DIRECTORY
            .ends_with("2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883"),
        "the record directory names the correctness run rather than the timing one: \
         {RECORD_DIRECTORY}",
    );

    let row = environment_row().expect("the retained environment row reads");
    assert!(
        row.len() >= 20,
        "the retained environment row carries {} field(s), which is fewer than the record has; a \
         parse that stopped early would report a truncated row as an intact one",
        row.len(),
    );

    let direct = direct_digests().expect("the retained direct rows read");
    assert_eq!(
        direct.len(),
        6,
        "the retained record states six correctness cells and this reader found {}",
        direct.len(),
    );
    for row in &direct {
        assert_eq!(
            row.result_sha256.len(),
            64,
            "{}: a retained digest is 64 hexadecimal characters and this one is {:?}",
            row.id,
            row.result_sha256,
        );
        assert!(
            row.m > 0 && row.n > 0 && row.k > 0,
            "{}: a cell with a zero extent contracts nothing",
            row.id,
        );
    }

    // The digests are distinct, which is what makes the per-cell comparison a
    // per-cell claim: six members compared against one repeated value would agree
    // for a reason that says nothing about five of them.
    let mut digests: Vec<&str> = direct
        .iter()
        .map(|row| row.result_sha256.as_str())
        .collect();
    digests.sort_unstable();
    digests.dedup();
    assert_eq!(
        digests.len(),
        6,
        "two correctness cells carry the same retained digest, so one of them is not the claim it \
         appears to be",
    );
}

/// The record's own row compares equal, field for field.
///
/// The neighbour every perturbation below is paired against. Without it a
/// comparison that reported *everything* as differing would satisfy each of them
/// while measuring nothing.
#[test]
fn the_records_own_row_agrees_in_every_compared_field() {
    let comparison = compare(&the_records_own_row()).expect("the retained row reads");
    assert_eq!(
        comparison.fields.len(),
        6,
        "six fields are compared; the record's `xcode` is deliberately not one of them because \
         nothing observes it",
    );
    assert!(
        comparison.differences().is_empty(),
        "the record's own row must agree with itself: {}",
        comparison.render(),
    );
    assert!(comparison.hardware_differences().is_empty());
    assert!(
        comparison
            .render()
            .contains("on the retained record's own row"),
        "an agreeing comparison must say so: {}",
        comparison.render(),
    );
}

/// Every compared field is watched refusing, one perturbation at a time.
///
/// **This is the check that makes the row comparison a check.** Six equalities
/// against six strings pass trivially if a field is read from the wrong key, is
/// compared against itself, or is never compared at all — and every one of those
/// defects leaves the agreeing case above green. Each row below perturbs exactly
/// one observed field and requires that field, and only that field, to be named.
///
/// The `hardware` classification is asserted beside the difference because it is
/// what `crate::envelope` acts on: a device difference declines the retained
/// comparison and a toolchain difference does not, so a field that drifted into
/// the wrong class would silently change which runs compare.
/// One perturbation: the field it must name, its class, and the change itself.
///
/// A named type rather than an inline tuple because the function pointer makes
/// the triple wide enough that a reader has to count parentheses to see which
/// element is which.
type Perturbation = (&'static str, bool, fn(&mut MeasurementBoundary));

#[test]
fn each_compared_field_is_watched_refusing_on_its_own() {
    let perturbations: [Perturbation; 6] = [
        ("device", true, |row| {
            row.device_name = "Apple M3 Pro".to_owned();
        }),
        ("gpu-family", true, |row| {
            row.gpu_family = "Apple8".to_owned();
        }),
        ("architecture", false, |row| {
            row.architecture = "x86_64".to_owned();
        }),
        ("os", false, |row| {
            row.os_build = "26A5388h".to_owned();
        }),
        ("offline-compiler", false, |row| {
            row.metal_compiler = "Apple metal version 32023.921 (metalfe-32023.921)".to_owned();
        }),
        ("sdk", false, |row| {
            row.sdk = "macosx 27.0 build 26A5388f".to_owned();
        }),
    ];

    for (name, hardware, perturb) in perturbations {
        let mut row = the_records_own_row();
        perturb(&mut row);
        let comparison = compare(&row).expect("the retained row reads");
        let differences = comparison.differences();
        assert_eq!(
            differences.len(),
            1,
            "perturbing {name} must move exactly that field: {}",
            comparison.render(),
        );
        assert_eq!(differences[0].name, name);
        assert_eq!(
            differences[0].hardware, hardware,
            "{name} changed which class it is in, and the class is what decides whether a run \
             declines the retained comparison",
        );
        assert_eq!(
            comparison.hardware_differences().len(),
            usize::from(hardware),
            "{name}: a hardware difference declines the retained comparison and a toolchain \
             difference does not",
        );
        assert!(
            comparison.render().contains(name),
            "a differing field must be named in the rendered sentence: {}",
            comparison.render(),
        );
    }
}

/// A comparison over no fields cannot report agreement.
///
/// The degenerate case the sentence above would otherwise render as a pass: an
/// empty field list has no differences, so `render` would say every field agreed
/// while comparing none. Stated as a property of the rendering rather than left
/// to the six-field assertion, because that assertion is about today's field set
/// and this is about what the sentence means.
#[test]
fn an_empty_comparison_renders_the_count_it_actually_made() {
    let empty = RowComparison { fields: Vec::new() };
    assert!(
        empty.render().contains("all 0 compared field(s)"),
        "an empty comparison must state that it compared nothing: {}",
        empty.render(),
    );
    let one = RowComparison {
        fields: vec![RowField {
            name: "device",
            retained: "Apple M4 Max".to_owned(),
            observed: "Apple M4 Max".to_owned(),
            hardware: true,
        }],
    };
    assert!(one.render().contains("all 1 compared field(s)"));
}
