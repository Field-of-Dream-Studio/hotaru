//! Reader and writer traits for Hotaru async IO.
//!
//! `std` is not an IO backend here: blocking `std::io::Read` / `Write`
//! does not fit this async trait surface. Concrete backend adapters live in
//! sibling crates such as `hotaru_io_tokio`, `hotaru_io_futures`, and
//! `hotaru_io_embedded`.
//!
//! # Read/write contract map
//!
//! The two sides are symmetric by transfer role, not by identical method
//! names. A reader pulls from an externally-sized source into caller-owned
//! storage; a writer pushes a caller-owned, already-sized source into an
//! externally-behaving sink.
//!
//! | Read side | Write side | Shared contract |
//! | --- | --- | --- |
//! | [`HotaruRead::into_buf`] | [`HotaruWrite::into_buf_write`] | Consume an adapter and produce its buffered form. |
//! | [`HotaruRead::read`] | [`HotaruWrite::write`] | Perform one partial transfer and return the byte count. |
//! | [`HotaruRead::read_exact`] | [`HotaruWrite::write_all`] | Strict completion: fill the entire destination or drain the entire source; inability to complete is `Self::Error`. |
//! | [`HotaruRead::read_to_end`] | [`HotaruWrite::write_exact`] | Outcome-driven count boundary: distinguish source exhaustion from reaching the caller's configured boundary. |
//! | [`HotaruBufRead::read_until`] | [`HotaruWrite::write_until`] | Transfer through an inclusive delimiter, subject to a cap. |
//! | [`HotaruBufRead::read_until_unbounded`] | [`HotaruWrite::write_until_unbounded`] | Explicitly opt out of the delimiter-transfer cap. |
//! | [`HotaruBufRead::read_line`] | [`HotaruWrite::write_line`] | Transfer through an existing newline, subject to a cap. |
//! | [`HotaruBufRead::read_line_unbounded`] | [`HotaruWrite::write_line_unbounded`] | Explicitly opt out of the line-transfer cap. |
//!
//! [`HotaruRead::read_to_end_unbounded`] has no separate write-side method:
//! a write already receives a finite source slice, and
//! [`HotaruWrite::write_all`] drains that slice. Conversely,
//! [`HotaruWrite::write_all_capped`] has no exact read-side counterpart. A
//! writer can inspect the complete source length before performing IO and
//! reject an oversized source without emitting a partial frame; a reader
//! cannot know an external source's final length in advance.
//!
//! [`HotaruBufRead::fill_buf`] and [`HotaruBufRead::consume`] are intentionally
//! read-only lookahead controls. [`HotaruWrite::flush`] and
//! [`HotaruWrite::shutdown`] are intentionally write-only sink lifecycle
//! controls. These are directional capabilities, not missing symmetric
//! transfer helpers.
//!
//! # Why some helpers require buffering
//!
//! [`HotaruRead`] contains operations that can be implemented without
//! lookahead. Delimiter and line reads live on [`HotaruBufRead`] because a raw
//! `read` may return bytes beyond the delimiter. [`HotaruBufRead::fill_buf`]
//! lets the default implementation inspect available bytes, while
//! [`HotaruBufRead::consume`] advances through only the selected prefix and
//! preserves the suffix for the next operation.
//!
//! Delimiter and line writes remain on [`HotaruWrite`]. Their source is an
//! already-available byte slice or string, so the implementation can find the
//! boundary before touching the sink and pass only the selected prefix to
//! [`HotaruWrite::write_all`]. No sink-side lookahead or buffering capability
//! is required. [`HotaruBufWrite`] is therefore currently a buffered-form
//! marker over [`HotaruWrite`], rather than the owner of separate record-write
//! helpers.
//!
//! # Failures, outcomes, and intentional asymmetry
//!
//! Backend failures remain `Result::Err`. This includes early EOF for the
//! strict [`HotaruRead::read_exact`] contract and zero write progress for the
//! strict [`HotaruWrite::write_all`] contract. [`TransferOutcome`] describes
//! a non-backend stopping event such as source exhaustion, a logical condition,
//! or a configured cap.
//!
//! Capped reads and writes intentionally need not mutate symmetrically. A
//! bounded read may consume the prefix that fits and leave later bytes
//! buffered. A bounded record write can preflight its known source and write
//! nothing when the selected prefix exceeds the cap, avoiding a partial
//! protocol frame.

/// Framework-owned IO errors used by concrete core implementations.
pub mod error;
/// Async read-side Hotaru IO traits.
pub mod read;
#[cfg(test)]
mod test;
/// Progress and termination outcomes shared by bounded and unbounded transfers.
pub mod transfer;
/// Async write-side Hotaru IO traits.
pub mod write;

pub use error::HotaruIOError;
pub use read::{HotaruBufRead, HotaruRead};
pub use transfer::{TransferOutcome, TransferTermination};
pub use write::{HotaruBufWrite, HotaruWrite};
