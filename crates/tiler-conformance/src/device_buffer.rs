//! The only module in this crate that may contain `unsafe`, and the complete
//! population is **two sites**: [`write_bytes`] and [`read_bytes`].
//!
//! # Why the sites exist at all
//!
//! Metal exposes no C API for buffer storage: `MTLBuffer`'s bytes are reached
//! through the raw pointer `metal::Buffer::contents` returns, and no binding in
//! this workspace exposes them safely. Moving a conformance run's operands onto
//! a device and its results back off therefore cannot be spelled without
//! dereferencing that pointer. Device creation, library loading, pipeline
//! construction, encoding, submission, and every comparison the run makes are
//! safe calls and live in [`crate::dispatch`] and [`crate::bf16_vertical`].
//!
//! That is the whole of the admitted justification, decided by Tom on
//! 2026-08-07 and recorded on
//! `decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`:
//! FFI memory management with Metal, where the foreign API leaves no safe
//! route. Not convenience, not performance, not a tidier shape.
//!
//! # The population, named and counted
//!
//! **Two.** [`write_bytes`] copies a host byte run into a buffer, and
//! [`read_bytes`] copies a buffer's byte run back out. Nothing else in this
//! crate contains `unsafe`. `crates/tiler/tests/workspace_unsafe_sites.rs`
//! cross-checks Cargo's actual packages against the explicit workspace list,
//! enumerates every target and supported source-loading edge, and pins this
//! pair's paths, complete item signatures, and exact reasons beside the two
//! prototype sites, so an addition or move is not absorbed into this module.
//!
//! # Why the interface is bytes rather than BF16 elements
//!
//! So that this module knows nothing about the format under test. Element
//! width, stride, packing order, and element count are all *derived* quantities
//! whose derivation is exactly what an end-to-end conformance run exists to
//! check — and a derivation performed inside an unsafe site could not be
//! perturbed without perturbing the unsafe site with it. Keeping the boundary
//! at `&[u8]` leaves every one of them in safe code, which is what lets
//! `crate::bf16_vertical` watch a wrongly derived width fail.
//!
//! `u8` also removes the only alignment question there could be: a byte copy
//! into or out of `contents()` has no alignment requirement and no invalid bit
//! pattern, so each site's obligation reduces to the length bound it asserts.
//!
//! # The invariant both rely on
//!
//! A buffer's length in bytes is fixed at allocation, and
//! `metal::Buffer::contents` returns a pointer valid for that whole length
//! while the buffer is alive. Each function asserts the byte length it is about
//! to touch against the buffer's own reported length, so a caller that
//! mis-derived one gets an attributable panic before any pointer is
//! dereferenced rather than a silent out-of-bounds copy.

use metal::Buffer;

/// Writes `bytes` into the front of `buffer`'s storage.
///
/// # Panics
///
/// Panics when `buffer` is shorter than `bytes`. The check is deliberate: it
/// converts a caller's allocation mistake into an immediate, attributable
/// failure instead of a write past the mapping.
#[allow(
    unsafe_code,
    reason = "MTLBuffer storage is reachable only through the raw pointer `Buffer::contents` returns; no Metal binding in this workspace exposes it safely, and a conformance run must place its operands there. The write is bounded by an asserted length check against the buffer's own byte length, copies bytes — a type with no alignment requirement, no invalid bit pattern, and no destructor — and retains no borrow."
)]
pub(crate) fn write_bytes(buffer: &Buffer, bytes: &[u8]) {
    let required = u64::try_from(bytes.len()).expect("a slice length fits a u64");
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the write needs {required}",
        buffer.length(),
    );
    // SAFETY: `contents()` returns a pointer valid for `buffer.length()` bytes
    // for as long as `buffer` is alive, and `buffer` is borrowed for this call.
    // The assertion above proves the destination spans at least `required`
    // bytes. `u8` has no alignment requirement, no invalid bit pattern, and no
    // destructor, so a byte copy into uninitialized Metal storage is well
    // defined. Source and destination are distinct allocations — the source is
    // a host slice and the destination is Metal's mapping — so they cannot
    // overlap.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.contents().cast::<u8>(), bytes.len());
    }
}

/// Reads the first `len` bytes of `buffer`'s storage.
///
/// # Panics
///
/// Panics when `buffer` is shorter than `len`, for the same reason
/// [`write_bytes`] does.
#[allow(
    unsafe_code,
    reason = "the read half of the same constraint: MTLBuffer storage is reachable only through `Buffer::contents`, and a conformance run must observe what the device wrote. Bounded by an asserted length check, reads bytes, and copies out rather than retaining a borrow of device memory."
)]
#[must_use]
pub(crate) fn read_bytes(buffer: &Buffer, len: usize) -> Vec<u8> {
    let required = u64::try_from(len).expect("a byte length fits a u64");
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the read needs {required}",
        buffer.length(),
    );
    let mut bytes = vec![0_u8; len];
    // SAFETY: as in `write_bytes`, with the direction reversed. The source spans
    // at least `required` bytes by the assertion, the destination is a freshly
    // allocated `Vec` of exactly `len` elements, and the two are distinct
    // allocations. The GPU write that produced these bytes is ordered before
    // this read by the caller's `wait_until_completed`, which
    // `crate::dispatch::submit` performs before it reaches a readback.
    unsafe {
        std::ptr::copy_nonoverlapping(buffer.contents().cast::<u8>(), bytes.as_mut_ptr(), len);
    }
    bytes
}
