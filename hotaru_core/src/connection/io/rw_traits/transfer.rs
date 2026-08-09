/// Reason a transfer stopped.
#[av::ver(
    unstable,
    since = "0.8.5",
    note = "Typed completion state for protocol-neutral capped IO",
    date = "2026-08-09"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferTermination {
    /// The operation reached its logical completion condition.
    Complete,
    /// The input source ended before the operation completed.
    SourceEnded,
    /// The configured cap was reached before the operation completed.
    CapReached,
}

/// Progress and completion state returned by a transfer.
///
/// Backend failures remain in the enclosing `Result::Err`; this value only
/// describes how a successful IO operation stopped. `transferred` counts bytes
/// consumed from a reader or accepted for writing during the current call.
#[av::ver(
    unstable,
    since = "0.8.5",
    note = "Typed progress report for protocol-neutral capped IO",
    date = "2026-08-09"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferOutcome {
    /// Bytes transferred during this call.
    pub transferred: usize,
    /// Reason the operation stopped.
    pub termination: TransferTermination,
}

impl TransferOutcome {
    pub(crate) const fn new(transferred: usize, termination: TransferTermination) -> Self {
        Self {
            transferred,
            termination,
        }
    }
}
