/// Reason a transfer stopped.
#[av::ver(
    unstable,
    since = "0.8.5",
    note = "Typed stopping event for protocol-neutral IO",
    date = "2026-08-09"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferTermination {
    /// The operation reached its configured logical condition, such as a
    /// delimiter or line ending.
    ConditionReached,
    /// No input remained in the operation's source.
    ///
    /// This can be successful completion when exhausting the source is the
    /// operation's goal, as with `read_to_end` or `write_all`. For a
    /// delimiter-oriented read it means EOF arrived before the delimiter.
    SourceEnded,
    /// The configured transfer cap was reached before another stop condition.
    CapReached,
}

/// Progress and stopping state returned by a transfer.
///
/// Backend failures remain in the enclosing `Result::Err`; this value describes
/// an operation that stopped without a backend failure. A termination variant
/// is not a universal success/failure classification: callers interpret it in
/// the context of the requested operation. `transferred` counts bytes consumed
/// from a reader or accepted for writing during the current call.
#[av::ver(
    unstable,
    since = "0.8.5",
    note = "Typed progress report for protocol-neutral IO",
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
    /// Creates a transfer outcome from its byte count and stopping event.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Constructor for protocol-neutral transfer outcomes",
        date = "2026-08-09"
    )]
    pub const fn new(transferred: usize, termination: TransferTermination) -> Self {
        Self {
            transferred,
            termination,
        }
    }
}
