pub const SELECTED: &str =
    tiler_environment_probe_macro::probe!(targets = [macos, ios_device, ios_simulator]);

pub fn unrelated() -> u32 {
    7
}

#[cfg(test)]
mod tests {
    #[test]
    fn selection_is_visible_in_macro_tokens() {
        assert_eq!(
            super::SELECTED,
            "targets = [macos, ios_device, ios_simulator]"
        );
    }
}
