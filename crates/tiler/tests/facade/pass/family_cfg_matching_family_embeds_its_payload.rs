//! A consumer target matching a selected artifact family selects that family's
//! payload, and never another family's.
//!
//! The plan this stands in for selects macOS and the iOS device family, both of
//! which built, so the one envelope carries two payloads — the shape Tom decided
//! on 2026-07-25: one artifact identity per compilation, with the consumer's
//! `#[cfg]` selecting a payload *within* an artifact it already holds.
//!
//! On the macOS host that compiles this file the macOS arm holds, so the selector
//! resolves to payload 1. Payload 0 is the iOS device's, and the assertion that
//! macOS did **not** select it is the point: canonical family order puts
//! `ios-device` first, so a selector that ignored the family and took the first
//! payload would be indistinguishable from a correct one on a single-family plan.
//!
//! The artifact is all 256 byte values, which is what makes this file evidence
//! about the emitter's byte-string escaping rather than about the printable
//! subset: a literal that mis-escaped `"`, `\`, or any non-printable byte would
//! either fail to compile here or round-trip to different bytes.
//!
//! The delivery items are at column zero because generated tokens carry no
//! indentation, and they are **byte-identical** to what
//! `tiler_macros::delivery::DeliveryPlan::items_source` emits for that plan;
//! `the_matching_fixture_compiles_what_this_emitter_produces` in the macro crate
//! reads this file and asserts it.

fn main() {
    let (artifact, selected) = {
const __TILER_ARTIFACT: &[u8] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\x7f\x80\x81\x82\x83\x84\x85\x86\x87\x88\x89\x8a\x8b\x8c\x8d\x8e\x8f\x90\x91\x92\x93\x94\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f\xa0\xa1\xa2\xa3\xa4\xa5\xa6\xa7\xa8\xa9\xaa\xab\xac\xad\xae\xaf\xb0\xb1\xb2\xb3\xb4\xb5\xb6\xb7\xb8\xb9\xba\xbb\xbc\xbd\xbe\xbf\xc0\xc1\xc2\xc3\xc4\xc5\xc6\xc7\xc8\xc9\xca\xcb\xcc\xcd\xce\xcf\xd0\xd1\xd2\xd3\xd4\xd5\xd6\xd7\xd8\xd9\xda\xdb\xdc\xdd\xde\xdf\xe0\xe1\xe2\xe3\xe4\xe5\xe6\xe7\xe8\xe9\xea\xeb\xec\xed\xee\xef\xf0\xf1\xf2\xf3\xf4\xf5\xf6\xf7\xf8\xf9\xfa\xfb\xfc\xfd\xfe\xff";
#[cfg(all(target_os = "ios", target_abi = ""))]
const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = ::core::option::Option::Some(0usize);
#[cfg(all(target_os = "macos", target_abi = ""))]
const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = ::core::option::Option::Some(1usize);
#[cfg(not(any(all(target_os = "ios", target_abi = ""), all(target_os = "macos", target_abi = ""))))]
const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = ::core::option::Option::None;
        (__TILER_ARTIFACT, __TILER_SELECTED_PAYLOAD)
    };

    let expected: Vec<u8> = (0..=u8::MAX).collect();
    assert_eq!(
        artifact, expected,
        "every byte of the embedded envelope must survive its literal unchanged",
    );
    assert_eq!(
        selected,
        Some(1usize),
        "a macOS consumer selects the macOS payload",
    );
    assert_ne!(
        selected,
        Some(0usize),
        "payload 0 is the iOS device family's, and a macOS consumer must never receive it",
    );
}
