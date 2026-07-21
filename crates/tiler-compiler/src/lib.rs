//! Target-independent optimization, scheduling, and structured lowering.
//!
//! This crate owns compiler decisions and may construct artifact plans. It must
//! not depend on Metal emission, live runtime APIs, Candle, or frontend syntax.

#[cfg(test)]
mod fusion;
#[cfg(test)]
mod physical;
#[cfg(test)]
mod pipeline;
#[cfg(test)]
mod program;
#[cfg(test)]
mod request;
