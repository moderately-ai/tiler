//! A consumer reaches `tensor!` through the facade alone, in both import
//! forms and from a module that imports nothing.
//!
//! The nested module is the part that tests the expansion rather than the
//! re-export: generated tokens spell a leading-`::` absolute path, so they
//! must resolve in a scope where nothing named `tiler` is in scope locally.

use tiler::tensor;

mod nested {
    pub fn region() -> impl core::fmt::Debug {
        tiler::tensor!()
    }
}

fn main() {
    let imported = tensor!();
    let qualified = tiler::tensor!();

    assert_eq!(imported, qualified);
    assert_eq!(format!("{imported:?}"), format!("{:?}", nested::region()));
}
