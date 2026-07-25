//! The non-published offline producer for the serial-Sum vertical slice.
//!
//! It exists to drive the already-implemented component capabilities — semantic
//! construction, compilation, MSL emission, offline Metal compilation, and
//! artifact packaging — through one path and prove the offline half of the
//! slice end to end. It implements no component capability of its own; the one
//! thing it owns is the orchestration those components cannot own individually,
//! which today is [`target`], the translation between the emitter's and the
//! driver's target vocabularies.

mod target;

fn main() {
    println!("serial-Sum compile prototype is not implemented yet");
}
