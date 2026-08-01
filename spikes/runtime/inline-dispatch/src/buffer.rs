//! The one place this spike needs `unsafe`, and nothing else.
//!
//! # Why the site exists at all
//!
//! `metal` 0.33.0 publishes exactly one storage accessor for an `MTLBuffer`:
//! `Buffer::contents(&self) -> *mut std::ffi::c_void`
//! (`metal-0.33.0/src/buffer.rs:24`). There is no slice accessor, no `read_to`,
//! and no typed view, so **there is no safe route through the foreign API** —
//! reading what a kernel wrote means dereferencing that pointer. Everything else
//! this spike does to a device — creating it, loading a library, building a
//! pipeline, allocating, encoding, dispatching, submitting, waiting, and reading
//! the terminal status — is a safe call and lives in [`crate::adapter`].
//!
//! Uploading operands needs no site of its own: `Device::new_buffer_with_data`
//! (`metal-0.33.0/src/device.rs:1956`) is a safe function that copies from a
//! pointer the caller may hand it safely, so the write half of the round trip is
//! outside this module by construction rather than by exemption.
//!
//! # The four conditions ADR 0079 requires, all visible below
//!
//! 1. no safe route through the foreign API — stated above, with the exact line;
//! 2. an `#[allow(unsafe_code, reason = …)]`, because the crate denies rather
//!    than forbids and every site opts in by name;
//! 3. an assertion bounding the block against **the foreign object's own
//!    report** — `Buffer::length()`, not a number this crate computed twice;
//! 4. a `SAFETY` comment naming the invariant the block relies on.
//!
//! The assertion is the one that carries the weight, and it is deliberately
//! against the buffer's report rather than against the length the route
//! declared: a route that asked for one length and an allocator that returned
//! another is exactly the disagreement a read past the mapping is made of, and
//! comparing the route against itself would not see it.

use metal::Buffer;

/// Copies `destination.len()` bytes out of `buffer`'s storage.
///
/// The destination is the region result's own storage, borrowed from the seam,
/// so the bytes a consumer receives are the ones this copy wrote and not a
/// second buffer that agrees with them.
///
/// # Panics
///
/// Panics when `buffer` reports fewer bytes than `destination` asks for. The
/// check converts a planning mistake into an immediate, attributable failure
/// instead of a read past the mapping, and it is unreachable on the route this
/// spike takes: the allocation is requested at the byte reach the artifact
/// declares, and `adapter::allocation_holds` already refused an allocator that
/// returned less — before the routing commit.
#[allow(
    unsafe_code,
    reason = "MTLBuffer storage is reachable only through the raw pointer `Buffer::contents` returns; `metal` 0.33.0 publishes no slice, typed view, or copy-out accessor. The read is bounded by an asserted length check against the buffer's own reported `length()`, copies plain-old-data bytes with no destructor, and copies out rather than retaining a borrow of device memory."
)]
pub fn read_into(buffer: &Buffer, destination: &mut [u8]) {
    let required = u64::try_from(destination.len()).expect("a slice length fits a u64");
    assert!(
        buffer.length() >= required,
        "buffer holds {} byte(s), the read needs {required}",
        buffer.length(),
    );
    // SAFETY: `contents()` returns a pointer valid for `buffer.length()` bytes
    // for as long as `buffer` is alive, and `buffer` is borrowed for this call.
    // The assertion above proves the source spans at least `destination.len()`
    // bytes. `u8` has no invalid bit patterns and no destructor, so a byte copy
    // out of Metal storage is well defined for any content. The source is device
    // memory and the destination is the caller's slice, so the two are distinct
    // allocations and cannot overlap. The GPU write that produced these bytes is
    // ordered before this read by the caller's `wait_until_completed` and its
    // observed `Completed` terminal status, both of which precede every call to
    // this function.
    unsafe {
        std::ptr::copy_nonoverlapping(
            buffer.contents().cast::<u8>(),
            destination.as_mut_ptr(),
            destination.len(),
        );
    }
}
