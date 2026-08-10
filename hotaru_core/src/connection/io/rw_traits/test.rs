use super::{
    HotaruBufRead, HotaruBufWrite, HotaruRead, HotaruWrite, TransferOutcome, TransferTermination,
};

struct SliceReader {
    bytes: alloc::vec::Vec<u8>,
    position: usize,
}

impl SliceReader {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
            position: 0,
        }
    }
}

impl HotaruRead for SliceReader {
    type Error = std::io::Error;
    type Buffered = Self;

    fn into_buf(self) -> Self::Buffered {
        self
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let available = &self.bytes[self.position..];
        let read = available.len().min(buf.len());
        buf[..read].copy_from_slice(&available[..read]);
        self.position += read;
        Ok(read)
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let read = self.read(buf).await?;
        if read == buf.len() {
            Ok(())
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
        }
    }
}

impl HotaruBufRead for SliceReader {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        Ok(&self.bytes[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        self.position = (self.position + amt).min(self.bytes.len());
    }
}

#[derive(Default)]
struct VecWriter {
    bytes: alloc::vec::Vec<u8>,
}

impl HotaruWrite for VecWriter {
    type Error = std::io::Error;
    type Buffered = Self;

    fn into_buf_write(self) -> Self::Buffered {
        self
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.bytes.extend_from_slice(buf);
        Ok(())
    }
}

impl HotaruBufWrite for VecWriter {}

struct WriteZeroWriter;

impl HotaruWrite for WriteZeroWriter {
    type Error = std::io::Error;
    type Buffered = Self;

    fn into_buf_write(self) -> Self::Buffered {
        self
    }

    async fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(0)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write_all(&mut self, _buf: &[u8]) -> Result<(), Self::Error> {
        Err(std::io::Error::from(std::io::ErrorKind::WriteZero))
    }
}

impl HotaruBufWrite for WriteZeroWriter {}

#[tokio::test]
async fn delimiter_inside_cap_wins_over_larger_available_buffer() {
    let mut reader = SliceReader::new(b"ok\nremaining");
    let mut output = alloc::vec::Vec::new();

    let outcome = reader.read_until(b'\n', &mut output, 3).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(3, TransferTermination::ConditionReached)
    );
    assert_eq!(output, b"ok\n");
    assert_eq!(reader.fill_buf().await.unwrap(), b"remaining");
}

#[tokio::test]
async fn cap_reached_consumes_only_the_prefix_that_fits() {
    let mut reader = SliceReader::new(b"abcdef");
    let mut output = alloc::vec::Vec::new();

    let outcome = reader.read_until(b'\n', &mut output, 4).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(4, TransferTermination::CapReached)
    );
    assert_eq!(output, b"abcd");
    assert_eq!(reader.fill_buf().await.unwrap(), b"ef");
}

#[tokio::test]
async fn eof_before_delimiter_is_reported_as_source_ended() {
    let mut reader = SliceReader::new(b"partial");
    let mut output = alloc::vec::Vec::new();

    let outcome = reader.read_until(b'\n', &mut output, 16).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(7, TransferTermination::SourceEnded)
    );
    assert_eq!(output, b"partial");
}

#[tokio::test]
async fn existing_buffer_length_counts_toward_cap() {
    let mut reader = SliceReader::new(b"cdef");
    let mut output = b"ab".to_vec();

    let outcome = reader.read_until(b'\n', &mut output, 4).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(2, TransferTermination::CapReached)
    );
    assert_eq!(output, b"abcd");
    assert_eq!(reader.fill_buf().await.unwrap(), b"ef");
}

#[tokio::test]
async fn unbounded_delimiter_read_reuses_bounded_termination_semantics() {
    let mut reader = SliceReader::new(b"line\nremaining");
    let mut output = alloc::vec::Vec::new();

    let outcome = reader
        .read_until_unbounded(b'\n', &mut output)
        .await
        .unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(5, TransferTermination::ConditionReached)
    );
    assert_eq!(output, b"line\n");
    assert_eq!(reader.fill_buf().await.unwrap(), b"remaining");
}

#[tokio::test]
async fn unbounded_delimiter_read_reports_source_ended_before_delimiter() {
    let mut reader = SliceReader::new(b"partial");
    let mut output = alloc::vec::Vec::new();

    let outcome = reader
        .read_until_unbounded(b'\n', &mut output)
        .await
        .unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(7, TransferTermination::SourceEnded)
    );
    assert_eq!(output, b"partial");
}

#[tokio::test]
async fn unbounded_line_read_reuses_bounded_implementation() {
    let mut reader = SliceReader::new(b"line\nremaining");
    let mut output = alloc::string::String::new();

    let outcome = reader.read_line_unbounded(&mut output).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(5, TransferTermination::ConditionReached)
    );
    assert_eq!(output, "line\n");
    assert_eq!(reader.fill_buf().await.unwrap(), b"remaining");
}

#[tokio::test]
async fn unbounded_line_read_reports_source_ended_before_newline() {
    let mut reader = SliceReader::new(b"partial");
    let mut output = alloc::string::String::new();

    let outcome = reader.read_line_unbounded(&mut output).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(7, TransferTermination::SourceEnded)
    );
    assert_eq!(output, "partial");
}

#[tokio::test]
async fn read_to_end_reports_source_ended_after_exhausting_input() {
    let mut reader = SliceReader::new(b"complete source");
    let mut output = alloc::vec::Vec::new();

    let outcome = reader.read_to_end(&mut output, 32).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(15, TransferTermination::SourceEnded)
    );
    assert_eq!(output, b"complete source");
}

#[tokio::test]
async fn read_to_end_stops_at_cap_without_overconsuming() {
    let mut reader = SliceReader::new(b"abcdef");
    let mut output = alloc::vec::Vec::new();

    let outcome = reader.read_to_end(&mut output, 4).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(4, TransferTermination::CapReached)
    );
    assert_eq!(output, b"abcd");
    assert_eq!(
        reader
            .read_to_end_unbounded(&mut output)
            .await
            .unwrap()
            .termination,
        TransferTermination::SourceEnded
    );
    assert_eq!(output, b"abcdef");
}

#[tokio::test]
async fn write_all_writes_the_entire_buffer() {
    let mut writer = VecWriter::default();

    writer.write_all(b"hello").await.unwrap();

    assert_eq!(writer.bytes, b"hello");
}

#[tokio::test]
async fn write_exact_reports_condition_reached_at_the_requested_count() {
    let mut writer = VecWriter::default();

    let outcome = writer.write_exact(b"hello", 3).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(3, TransferTermination::ConditionReached)
    );
    assert_eq!(writer.bytes, b"hel");
}

#[tokio::test]
async fn write_exact_reports_source_ended_when_the_buffer_is_shorter() {
    let mut writer = VecWriter::default();

    let outcome = writer.write_exact(b"hello", 8).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(5, TransferTermination::SourceEnded)
    );
    assert_eq!(writer.bytes, b"hello");
}

#[tokio::test]
async fn write_exact_zero_reaches_the_condition_without_writing() {
    let mut writer = VecWriter::default();

    let outcome = writer.write_exact(b"hello", 0).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(0, TransferTermination::ConditionReached)
    );
    assert!(writer.bytes.is_empty());
}

#[tokio::test]
async fn write_exact_propagates_write_all_errors() {
    let error = WriteZeroWriter.write_exact(b"hello", 3).await.unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::WriteZero);
}

#[tokio::test]
async fn capped_write_reports_source_ended_after_writing_the_full_buffer() {
    let mut writer = VecWriter::default();

    let outcome = writer.write_all_capped(b"hello", 5).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(5, TransferTermination::SourceEnded)
    );
    assert_eq!(writer.bytes, b"hello");
}

#[tokio::test]
async fn capped_write_rejects_an_oversized_buffer_before_writing() {
    let mut writer = VecWriter::default();

    let outcome = writer.write_all_capped(b"hello", 4).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(0, TransferTermination::CapReached)
    );
    assert!(writer.bytes.is_empty());
}

#[tokio::test]
async fn delimiter_write_reports_condition_and_ignores_remaining_source() {
    let mut writer = VecWriter::default();

    let outcome = writer
        .write_until(b'\n', b"line\nremaining", 8)
        .await
        .unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(5, TransferTermination::ConditionReached)
    );
    assert_eq!(writer.bytes, b"line\n");
}

#[tokio::test]
async fn delimiter_write_reports_source_ended_when_delimiter_is_absent() {
    let mut writer = VecWriter::default();

    let outcome = writer
        .write_until_unbounded(b'\n', b"partial")
        .await
        .unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(7, TransferTermination::SourceEnded)
    );
    assert_eq!(writer.bytes, b"partial");
}

#[tokio::test]
async fn delimiter_write_rejects_over_cap_prefix_before_writing() {
    let mut writer = VecWriter::default();

    let outcome = writer.write_until(b'\n', b"line\n", 4).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(0, TransferTermination::CapReached)
    );
    assert!(writer.bytes.is_empty());
}

#[tokio::test]
async fn line_write_uses_existing_newline_without_appending_one() {
    let mut writer = VecWriter::default();

    let outcome = writer
        .write_line_unbounded("line\nremaining")
        .await
        .unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(5, TransferTermination::ConditionReached)
    );
    assert_eq!(writer.bytes, b"line\n");
}
