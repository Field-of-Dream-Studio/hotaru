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

#[tokio::test]
async fn delimiter_inside_cap_wins_over_larger_available_buffer() {
    let mut reader = SliceReader::new(b"ok\nremaining");
    let mut output = alloc::vec::Vec::new();

    let outcome = reader.read_until(b'\n', &mut output, 3).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(3, TransferTermination::Complete)
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
        TransferOutcome::new(5, TransferTermination::Complete)
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
        TransferOutcome::new(5, TransferTermination::Complete)
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
async fn capped_write_reports_complete_after_writing_the_full_buffer() {
    let mut writer = VecWriter::default();

    let outcome = writer.write_all_capped(b"hello", 5).await.unwrap();

    assert_eq!(
        outcome,
        TransferOutcome::new(5, TransferTermination::Complete)
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
