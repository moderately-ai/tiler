// Claim 2, inertness in its natural form: the sibling fail case forces the
// unknown lint to an error with `#![deny(unknown_lints)]`; this one does not.
// It compiles even though `Growing::B` is omitted, which is what "the attribute
// is inert and yields only an `unknown_lints` warning" means. Without the
// repository gate's warning-free requirement, a consumer that spelled the
// attribute and forgot the feature gate would get no signal at all.

use non_exhaustive_defining::Growing;

fn classify(value: &Growing) -> &'static str {
    #[deny(non_exhaustive_omitted_patterns)]
    match value {
        Growing::A => "a",
        _ => "unsupported",
    }
}

fn main() {
    assert_eq!(classify(&Growing::A), "a");
    assert_eq!(classify(&Growing::B), "unsupported");
}
