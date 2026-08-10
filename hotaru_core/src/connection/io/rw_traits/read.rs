use core::future::Future;

use super::super::MaybeSend;
use super::transfer::{TransferOutcome, TransferTermination};

/// Async byte reader.
pub trait HotaruRead {
    /// Concrete error returned by this reader. A backend picks `HotaruIOError`
    /// or its own type (e.g. tokio uses `std::io::Error`).
    type Error: core::error::Error + Send + Sync + 'static;

    type Buffered: HotaruBufRead<Error = Self::Error> + Unpin + MaybeSend + 'static;

    /// Consumes this reader and returns its buffered form.  
    fn into_buf(self) -> Self::Buffered
    where
        Self: Sized;

    /// Reads bytes into `buf`, returning the number written.
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend + 'a;

    /// Reads exactly `buf.len()` bytes. Implementors signal "EOF before the
    /// buffer was filled" through their own `Self::Error` (concrete impls use
    /// `HotaruIOError::UnexpectedEof`; tokio uses `ErrorKind::UnexpectedEof`).
    /// Required — the trait definition stays in terms of `Self::Error` only,
    /// so the sentinel construction lives in the concrete impl.
    fn read_exact<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend;

    /// Reads into `buf` until the source ends or `cap` is reached.
    ///
    /// `cap` bounds the final length of `buf`, including bytes already in it.
    /// `SourceEnded` means the reader returned EOF, while `CapReached` means
    /// the cap filled first. When both could coincide, the method reports
    /// `CapReached` without performing an additional read to probe for EOF.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Bounded read-to-end with typed termination",
        date = "2026-08-09"
    )]
    fn read_to_end<'a>(
        &'a mut self,
        buf: &'a mut alloc::vec::Vec<u8>,
        cap: usize,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        async move {
            let mut transferred = 0;
            let mut chunk = [0_u8; 1024];

            if buf.len() >= cap {
                return Ok(TransferOutcome::new(
                    transferred,
                    TransferTermination::CapReached,
                ));
            }

            loop {
                let remaining = cap - buf.len();
                let request = remaining.min(chunk.len());
                let read = self.read(&mut chunk[..request]).await?;

                if read == 0 {
                    return Ok(TransferOutcome::new(
                        transferred,
                        TransferTermination::SourceEnded,
                    ));
                }

                buf.extend_from_slice(&chunk[..read]);
                transferred += read;

                if buf.len() == cap {
                    return Ok(TransferOutcome::new(
                        transferred,
                        TransferTermination::CapReached,
                    ));
                }
            }
        }
    }

    /// Reads into `buf` until the source ends without an application-level
    /// cap.
    ///
    /// This is an explicit opt-out from bounded accumulation. Prefer
    /// [`HotaruRead::read_to_end`] for untrusted or otherwise size-unknown
    /// input. `CapReached` is only theoretically possible at the address-space
    /// limit.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Explicitly unbounded read-to-end",
        date = "2026-08-09"
    )]
    fn read_to_end_unbounded<'a>(
        &'a mut self,
        buf: &'a mut alloc::vec::Vec<u8>,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        self.read_to_end(buf, usize::MAX)
    }
}

/// Buffered async byte reader. Carries protocol-detection peeked bytes
/// through `Protocol::open_channel` without leaking `tokio::io::BufReader`.
pub trait HotaruBufRead: HotaruRead {
    /// Returns a slice of the currently buffered bytes, filling the buffer
    /// from the underlying reader if it's empty.
    fn fill_buf<'a>(
        &'a mut self,
    ) -> impl Future<Output = Result<&'a [u8], Self::Error>> + MaybeSend + 'a;

    /// Marks the first `amt` bytes of the internal buffer as consumed so
    /// the next `fill_buf` skips them.
    fn consume(&mut self, amt: usize);

    /// Reads bytes into `buf` until `delimiter` is encountered, inclusive.
    ///
    /// `TransferTermination::ConditionReached` means the delimiter was
    /// consumed, `SourceEnded` means EOF arrived first, and `CapReached` means
    /// `cap` was filled before the delimiter was found. `Self::Error` is
    /// reserved for failures reported by the underlying IO backend.
    ///
    /// `cap` bounds the final length of `buf`, including any bytes already in
    /// it. On `CapReached`, the method consumes and appends exactly the prefix
    /// that fits; bytes beyond the cap remain unread. Callers must decide
    /// whether an incomplete logical record can be resumed or the connection
    /// must be discarded.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Bounded delimiter read with typed termination",
        date = "2026-08-09"
    )]
    fn read_until<'a>(
        &'a mut self,
        delimiter: u8,
        buf: &'a mut alloc::vec::Vec<u8>,
        cap: usize,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        async move {
            let mut transferred = 0;

            if buf.len() >= cap {
                return Ok(TransferOutcome::new(
                    transferred,
                    TransferTermination::CapReached,
                ));
            }

            loop {
                let (termination, used) = {
                    let available = self.fill_buf().await?;
                    if available.is_empty() {
                        return Ok(TransferOutcome::new(
                            transferred,
                            TransferTermination::SourceEnded,
                        ));
                    }

                    let remaining = cap - buf.len();
                    let available_within_cap = &available[..available.len().min(remaining)];

                    if let Some(i) = available_within_cap
                        .iter()
                        .position(|candidate| *candidate == delimiter)
                    {
                        buf.extend_from_slice(&available[..=i]);
                        (Some(TransferTermination::ConditionReached), i + 1)
                    } else {
                        buf.extend_from_slice(available_within_cap);
                        let termination = (available_within_cap.len() == remaining)
                            .then_some(TransferTermination::CapReached);
                        (termination, available_within_cap.len())
                    }
                };

                self.consume(used);
                transferred += used;

                if let Some(termination) = termination {
                    return Ok(TransferOutcome::new(transferred, termination));
                }
            }
        }
    }

    /// Reads through `delimiter` without imposing an application-level cap.
    ///
    /// This is an explicit opt-out from bounded accumulation. Prefer
    /// [`HotaruBufRead::read_until`] for protocol data or any other untrusted
    /// input. The returned [`TransferOutcome`] still distinguishes a delimiter
    /// match (`ConditionReached`) from EOF before the delimiter
    /// (`SourceEnded`). `CapReached` is only theoretically possible at the
    /// address-space limit.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Explicitly unbounded delimiter read",
        date = "2026-08-09"
    )]
    fn read_until_unbounded<'a>(
        &'a mut self,
        delimiter: u8,
        buf: &'a mut alloc::vec::Vec<u8>,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        self.read_until(delimiter, buf, usize::MAX)
    }

    /// Reads a line into `buf` (up to and including the next `\n`).
    ///
    /// The returned termination has the same meaning as
    /// [`HotaruBufRead::read_until`]. On `CapReached`, `buf` receives
    /// the prefix that fit within `cap`.
    ///
    /// `cap` applies to the newly-read line, not to text already present in
    /// `buf`.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Bounded line read with typed termination",
        date = "2026-08-09"
    )]
    fn read_line<'a>(
        &'a mut self,
        buf: &'a mut alloc::string::String,
        cap: usize,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        async move {
            let mut bytes = alloc::vec::Vec::new();
            let outcome = self.read_until(b'\n', &mut bytes, cap).await?;
            buf.push_str(&alloc::string::String::from_utf8_lossy(&bytes));
            Ok(outcome)
        }
    }

    /// Reads a line without imposing an application-level cap.
    ///
    /// This is an explicit opt-out from bounded accumulation. Prefer
    /// [`HotaruBufRead::read_line`] for protocol data or any other untrusted
    /// input. The returned [`TransferOutcome`] still distinguishes a complete
    /// line (`ConditionReached`) from EOF before the newline (`SourceEnded`).
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Explicitly unbounded line read",
        date = "2026-08-09"
    )]
    fn read_line_unbounded<'a>(
        &'a mut self,
        buf: &'a mut alloc::string::String,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        self.read_line(buf, usize::MAX)
    }
}
