//! Reader and writer traits for Hotaru async IO.
//!
//! `std` is not an IO backend here: blocking `std::io::Read` / `Write`
//! does not fit this async trait surface. Concrete backend adapters live in
//! sibling crates such as `hotaru_io_tokio`, `hotaru_io_futures`, and
//! `hotaru_io_embedded`.

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
