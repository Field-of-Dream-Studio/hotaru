//! #27 无上限行读取验证测试
//!
//! 验证 `read_until` / `read_line` 在 `max_size` 限制下不会无限分配内存。

use hotaru_core::connection::{HotaruBufRead, HotaruRead};

/// 从字节切片读取的简单 `HotaruRead` 实现。
struct SliceRead {
    data: Vec<u8>,
    pos: usize,
}

impl SliceRead {
    fn new(data: &[u8]) -> Self {
        Self { data: data.to_vec(), pos: 0 }
    }
}

impl HotaruRead for SliceRead {
    type Error = std::io::Error;
    type Buffered = Self;

    fn into_buf(self) -> Self::Buffered {
        self
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let avail = (self.data.len() - self.pos).min(buf.len());
        buf[..avail].copy_from_slice(&self.data[self.pos..self.pos + avail]);
        self.pos += avail;
        Ok(avail)
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let n = self.read(buf).await?;
        if n < buf.len() {
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF",
            ))
        } else {
            Ok(())
        }
    }
}

impl HotaruBufRead for SliceRead {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        Ok(&self.data[self.pos..])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = (self.pos + amt).min(self.data.len());
    }
}

// ── read_until ────────────────────────────────────────────────────

#[tokio::test]
async fn read_until_normal_within_limit() {
    let mut reader = SliceRead::new(b"hello\nworld");
    let mut buf = Vec::new();
    let n = reader.read_until(b'\n', &mut buf, 1024).await.unwrap();
    assert_eq!(n, 6);
    assert_eq!(&buf, b"hello\n");
}

#[tokio::test]
async fn read_until_exact_limit_passes() {
    let mut reader = SliceRead::new(b"abc\n");
    let mut buf = Vec::new();
    let n = reader.read_until(b'\n', &mut buf, 4).await.unwrap();
    assert_eq!(n, 4);
    assert_eq!(&buf, b"abc\n");
}

#[tokio::test]
async fn read_until_stops_at_max_size() {
    // 发送一个很长的无换行数据，max_size 限制分配
    let data = vec![b'a'; 100_000];
    let mut reader = SliceRead::new(&data);
    let mut buf = Vec::new();
    let n = reader.read_until(b'\n', &mut buf, 1024).await.unwrap();
    // 应该截停在 max_size 以内，不会分配 100KB
    assert!(buf.len() <= 1024);
    assert!(n <= 1024);
}

#[tokio::test]
async fn read_until_no_delimiter_truncates() {
    // 无分隔符，读完或超过 max_size 都应该停止
    let mut reader = SliceRead::new(b"no_newline_here");
    let mut buf = Vec::new();
    let n = reader.read_until(b'\n', &mut buf, 10).await.unwrap();
    assert!(n <= 10);
}

// ── read_line ─────────────────────────────────────────────────────

#[tokio::test]
async fn read_line_normal_within_limit() {
    let mut reader = SliceRead::new(b"hello world\nnext line\n");
    let mut line = String::new();
    let n = reader.read_line(&mut line, 1024).await.unwrap();
    assert_eq!(n, 12);
    assert_eq!(line, "hello world\n");
}

#[tokio::test]
async fn read_line_stops_at_max_size() {
    let data = vec![b'a'; 100_000];
    let mut reader = SliceRead::new(&data);
    let mut line = String::new();
    let n = reader.read_line(&mut line, 1024).await.unwrap();
    assert!(line.len() <= 1024);
    assert!(n <= 1024);
}

#[tokio::test]
async fn read_line_empty_input() {
    let mut reader = SliceRead::new(b"");
    let mut line = String::new();
    let n = reader.read_line(&mut line, 1024).await.unwrap();
    assert_eq!(n, 0);
    assert!(line.is_empty());
}
