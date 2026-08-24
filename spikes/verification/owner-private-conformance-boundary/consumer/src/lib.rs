#[cfg(feature = "probe-test-only")]
pub fn dependency_test_cfg_is_not_visible() -> &'static [&'static str] {
    owner::test_only_inventory()
}

#[cfg(feature = "probe-private")]
pub fn dependency_private_item_is_not_visible() -> &'static [&'static str] {
    owner::private_inventory()
}

#[cfg(feature = "probe-feature")]
pub fn dependency_feature_surface_is_public() -> &'static [&'static str] {
    owner::feature_inventory()
}
