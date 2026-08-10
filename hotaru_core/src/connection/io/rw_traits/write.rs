use core::future::Future;

use super::super::MaybeSend;
use super::transfer::{TransferOutcome, TransferTermination};

/// Async byte writer.
pub trait HotaruWrite {
    type Error: core::error::Error + Send + Sync + 'static;

    type Buffered: HotaruBufWrite<Error = Self::Error> + Unpin + MaybeSend + 'static;

    /// Consumes this writer and returns its buffered form.
    fn into_buf_write(self) -> Self::Buffered
    where
        Self: Sized;

    fn write<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend + 'a;

    fn flush(&mut self) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend + '_;

    /// Default no-op for backends that rely on drop semantics
    /// (embedded-io-async); tokio blanket overrides this to send TCP FIN.
    fn shutdown(&mut self) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend + '_ {
        async { Ok(()) }
    }

    /// Writes the entire buffer, looping until all bytes are consumed.
    /// Implementors signal "writer accepted 0 bytes" through their own
    /// `Self::Error`. Required for the same reason as `read_exact`.
    fn write_all<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend;

    /// Writes up to `exact` bytes from the source buffer.
    ///
    /// `SourceEnded` means the source buffer was exhausted at or before
    /// `exact`. `ConditionReached` means `exact` bytes were written while the
    /// source still contained more bytes. `exact` is a logical condition, not
    /// a transfer cap, so this method does not return `CapReached`. Backend
    /// failures, including a writer accepting zero bytes before the selected
    /// prefix is exhausted, remain in `Self::Error` through
    /// [`HotaruWrite::write_all`].
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Exact-count write with typed termination",
        date = "2026-08-09"
    )]
    fn write_exact<'a>(
        &'a mut self,
        buf: &'a [u8],
        exact: usize,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        async move {
            let transferred = buf.len().min(exact);
            self.write_all(&buf[..transferred]).await?;

            let termination = if buf.len() <= exact {
                TransferTermination::SourceEnded
            } else {
                TransferTermination::ConditionReached
            };

            Ok(TransferOutcome::new(transferred, termination))
        }
    }

    /// Writes all of `buf` only when it fits within `cap`.
    ///
    /// Inputs larger than `cap` are rejected before any bytes are written and
    /// return `CapReached` with `transferred == 0`. This preflight behavior
    /// prevents a capped write from emitting a partial protocol frame.
    /// `Self::Error` is reserved for failures reported by the underlying IO
    /// backend while writing an accepted buffer.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Preflight-capped complete write",
        date = "2026-08-09"
    )]
    fn write_all_capped<'a>(
        &'a mut self,
        buf: &'a [u8],
        cap: usize,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        async move {
            if buf.len() > cap {
                return Ok(TransferOutcome::new(0, TransferTermination::CapReached));
            }

            self.write_all(buf).await?;
            Ok(TransferOutcome::new(
                buf.len(),
                TransferTermination::SourceEnded,
            ))
        }
    }

    /// Writes through the first `delimiter`, including it.
    ///
    /// `ConditionReached` means the delimiter was written. `SourceEnded`
    /// means the entire source buffer was written without finding it.
    /// `CapReached` means the intended prefix exceeded `cap`; in that case the
    /// method writes nothing so it cannot emit a partial protocol frame.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Preflight-bounded delimiter write",
        date = "2026-08-09"
    )]
    fn write_until<'a>(
        &'a mut self,
        delimiter: u8,
        buf: &'a [u8],
        cap: usize,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        async move {
            let condition_end = buf
                .iter()
                .position(|candidate| *candidate == delimiter)
                .map(|index| index + 1);
            let intended = condition_end.unwrap_or(buf.len());

            if intended > cap {
                return Ok(TransferOutcome::new(0, TransferTermination::CapReached));
            }

            self.write_all(&buf[..intended]).await?;
            let termination = condition_end.map_or(TransferTermination::SourceEnded, |_| {
                TransferTermination::ConditionReached
            });
            Ok(TransferOutcome::new(intended, termination))
        }
    }

    /// Writes through the first `delimiter` without an application-level cap.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Explicitly unbounded delimiter write",
        date = "2026-08-09"
    )]
    fn write_until_unbounded<'a>(
        &'a mut self,
        delimiter: u8,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        self.write_until(delimiter, buf, usize::MAX)
    }

    /// Writes one line from `line`, through and including its first `\n`.
    ///
    /// This method does not append a newline. If `line` contains no newline,
    /// the entire string is written and the termination is `SourceEnded`.
    /// `cap` is measured in UTF-8 bytes. As with [`HotaruWrite::write_until`],
    /// an over-cap line is rejected before any bytes are written.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Preflight-bounded line write",
        date = "2026-08-09"
    )]
    fn write_line<'a>(
        &'a mut self,
        line: &'a str,
        cap: usize,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        self.write_until(b'\n', line.as_bytes(), cap)
    }

    /// Writes one line without an application-level cap.
    ///
    /// This method writes through an existing newline and does not append one.
    #[av::ver(
        unstable,
        since = "0.8.5",
        note = "Explicitly unbounded line write",
        date = "2026-08-09"
    )]
    fn write_line_unbounded<'a>(
        &'a mut self,
        line: &'a str,
    ) -> impl Future<Output = Result<TransferOutcome, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        self.write_line(line, usize::MAX)
    }
}

pub trait HotaruBufWrite: HotaruWrite {}
