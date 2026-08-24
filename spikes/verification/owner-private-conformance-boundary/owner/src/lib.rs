#[cfg(not(feature = "conditional-subject"))]
const PRIVATE_SUBJECTS: [&str; 2] = ["owner.subject.alpha@1", "owner.subject.beta@1"];

#[cfg(feature = "conditional-subject")]
const PRIVATE_SUBJECTS: [&str; 3] = [
    "owner.subject.alpha@1",
    "owner.subject.beta@1",
    "owner.subject.conditional@1",
];

#[cfg(feature = "conditional-subject")]
#[allow(
    dead_code,
    reason = "the coupled implementation/declaration feature is the configuration-shrink negative-control subject"
)]
fn conditional_implementation() -> bool {
    true
}

#[allow(
    dead_code,
    reason = "the inaccessible function is the private-boundary negative-control subject"
)]
pub(crate) const fn private_inventory() -> &'static [&'static str] {
    &PRIVATE_SUBJECTS
}

#[cfg(test)]
pub fn test_only_inventory() -> &'static [&'static str] {
    private_inventory()
}

#[cfg(feature = "conformance-internal")]
pub const fn feature_inventory() -> &'static [&'static str] {
    &PRIVATE_SUBJECTS
}

#[cfg(test)]
mod tests {
    use super::private_inventory;

    #[test]
    fn emit_private_inventory() {
        let subjects = private_inventory();
        #[cfg(not(feature = "conditional-subject"))]
        assert_eq!(subjects.len(), 2);
        #[cfg(feature = "conditional-subject")]
        {
            assert!(super::conditional_implementation());
            assert_eq!(subjects.len(), 3);
        }
        if let Ok(path) = std::env::var("OWNER_MANIFEST_OUT") {
            let mut bytes = subjects.join("\n");
            bytes.push('\n');
            std::fs::write(path, bytes).expect("write private owner inventory");
        }
    }
}
