//! Body buffer reading and writing with transfer framing.

use hotaru_core::connection::{HotaruBufRead, HotaruWrite, TransferTermination};

use crate::util::content_type::HttpContentType;
use crate::message::meta::{HttpMeta, MetaError};
use crate::protocol::{HttpError, HttpSafety};
use crate::util::encoding::TransferCodings;

use super::{BodyError, ChunkingError, HttpBody};

impl HttpBody {
    /// Converts stream bytes into a buffer.
    ///
    /// The reverse operation is [`Self::write_buffer`].
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Read body buffers with transfer framing",
        date = "2026-11-13"
    )]
    pub async fn read_buffer<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        header: &mut HttpMeta,
        parse_config: &HttpSafety,
    ) -> Result<Self, HttpError> {
        Ok(Self::Buffer {
            data: Self::read_binary_info(buf_reader, header, parse_config).await?,
            content_type: header
                .get_content_type()
                .unwrap_or(HttpContentType::from_str("")),
            content_coding: header.get_encoding()?.content().clone(),
        })
    }

    /// Parse the HTTP body directly from a TCP Stream
    pub async fn direct_parse<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        header: &mut HttpMeta,
        parse_config: &HttpSafety,
    ) -> Result<Self, crate::protocol::HttpError> {
        let buffer = Self::read_buffer(buf_reader, header, parse_config).await?;
        Ok(buffer.decode(parse_config)?)
    }

    pub async fn read_binary_info<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        header: &mut HttpMeta,
        parse_config: &HttpSafety,
    ) -> Result<Vec<u8>, HttpError> {
        /// Reads body with Content-Length
        async fn read_content_length_body<
            R: HotaruBufRead<Error = std::io::Error> + Unpin + Send,
        >(
            buf_reader: &mut R,
            safety_setting: &HttpSafety,
            content_length: usize,
        ) -> Result<Vec<u8>, HttpError> {
            // Security: reject before reading. Truncating (min(cl, cap)) would leave
            // the excess body bytes on the wire and let the keep-alive loop parse
            // them as the next request — CL desync / request smuggling.
            if !safety_setting.check_body_size(content_length) {
                return Err(HttpError::Body(BodyError::TooLarge));
            }
            let mut body_buffer = vec![0; content_length];
            buf_reader.read_exact(&mut body_buffer).await?;
            Ok(body_buffer)
        }

        /// Reads chunked transfer encoding body
        ///
        /// # Security Philosophy: Efficient Validation Through Size Limits
        ///
        /// This parser follows a pragmatic security approach: **we only validate data size limits,
        /// not every possible malformed input**. This philosophy provides:
        ///
        /// 1. **Performance**: Fast parsing without exhaustive validation of every byte
        /// 2. **Energy Efficiency**: Minimal CPU cycles spent on validation overhead
        /// 3. **Equivalent Safety**: Size limits prevent all critical attacks (DoS, memory exhaustion)
        /// 4. **Simplicity**: Clear, maintainable code with focused security checks
        ///
        /// ## What We Check (Critical)
        /// - [x] Cumulative size limits (prevents DoS)
        /// - [x] Invalid hex chunk sizes (prevents crashes)
        /// - [x] CRLF terminators (prevents protocol confusion)
        ///
        /// ## What We Don't Check (Non-Critical)
        /// - [ ] Chunk extension validity (doesn't affect security if size is validated)
        /// - [ ] Duplicate zero chunks (harmless, just ends parsing)
        /// - [ ] Chunk data content validation (application layer concern)
        ///
        /// **Rationale**: If data doesn't overflow the upper size limit, it's safe to process.
        /// Malformed but size-compliant data will be caught at the application layer or cause
        /// predictable failures without security impact. This saves energy while maintaining
        /// equivalent security to exhaustive validation.
        async fn read_chunked_body<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
            buf_reader: &mut R,
            header: &mut HttpMeta,
            safety_setting: &HttpSafety,
        ) -> Result<Vec<u8>, HttpError> {
            let mut body_buffer = Vec::new();
            let mut current_size = 0;

            loop {
                // Read chunk size line
                let mut size_line = String::new();
                let outcome = buf_reader
                    .read_line(&mut size_line, safety_setting.effective_line_length())
                    .await?;
                if outcome.termination == TransferTermination::CapReached {
                    return Err(HttpError::Body(BodyError::Chunking(
                        ChunkingError::LineTooLong,
                    )));
                }
                let chunk_size_str = size_line.trim_end_matches(|c| c == '\r' || c == '\n');
                // RFC 9112 §7.1.1: strip chunk extension (";ext=...") before hex parse
                let chunk_size_str =
                    chunk_size_str.split(';').next().unwrap_or("").trim_end();

                // Parse chunk size (validates hex format - critical for preventing crashes)
                let chunk_size = usize::from_str_radix(chunk_size_str, 16)
                    .map_err(|_| HttpError::Body(BodyError::Chunking(ChunkingError::InvalidSize)))?;

                if chunk_size == 0 {
                    break; // End of chunks
                }

                // Cumulative size validation — see PR #26 for the checked-addition rationale.
                current_size = match safety_setting.check_body_size_delta(current_size, chunk_size) {
                    Some(new_total) => new_total,
                    None => return Err(HttpError::Body(BodyError::TooLarge)),
                };

                // Read chunk data (only reached if validation passed)
                let mut chunk_data = vec![0; chunk_size];
                buf_reader.read_exact(&mut chunk_data).await?;
                body_buffer.extend_from_slice(&chunk_data);

                // Read trailing CRLF
                let mut crlf = [0; 2];
                buf_reader.read_exact(&mut crlf).await?;
                if crlf != [b'\r', b'\n'] {
                    return Err(HttpError::Body(BodyError::Chunking(
                        ChunkingError::InvalidTerminator,
                    )));
                }
            }

            // Read trailing headers (if any) — bare header block, no start line.
            header
                .append_headers_from_stream(buf_reader, safety_setting, false)
                .await?;

            Ok(body_buffer)
        }

        // Validate Content-Length before selecting the framing mode. This also
        // protects callers that construct HttpMeta directly instead of using
        // HttpMeta::from_stream.
        let content_length = header.get_content_length()?;

        if content_length.is_some() && header.header.contains_key("transfer-encoding") {
            return Err(HttpError::Meta(MetaError::ConflictingFraming));
        }

        // Read raw body data
        let encoding = header.get_encoding()?;
        let raw_data = if encoding.transfer().is_chunked() {
            read_chunked_body(buf_reader, header, parse_config).await?
        } else {
            let content_length = usize::try_from(content_length.unwrap_or(0))
                .map_err(|_| HttpError::Body(BodyError::TooLarge))?;
            read_content_length_body(buf_reader, parse_config, content_length).await?
        };

        Ok(raw_data)
    }

    /// Converts a buffer into stream bytes.
    ///
    /// The reverse operation is [`Self::read_buffer`].
    #[av::ver(
        unstable,
        since = "0.8.6",
        note = "Write prepared body buffers with transfer framing",
        date = "2026-11-13"
    )]
    pub async fn write_buffer<W: HotaruWrite<Error = std::io::Error> + Unpin + Send>(
        self,
        writer: &mut W,
        transfer_coding: &TransferCodings,
    ) -> Result<(), HttpError> {
        let data = match self {
            Self::Buffer { data, .. } => data,
            _ => return Err(BodyError::NotBuffered.into()),
        };

        if transfer_coding.is_chunked() {
            if !data.is_empty() {
                let size = format!("{:x}\r\n", data.len());
                writer.write_all(size.as_bytes()).await?;
                writer.write_all(&data).await?;
                writer.write_all(b"\r\n").await?;
            }
            writer.write_all(b"0\r\n\r\n").await?;
        } else {
            writer.write_all(&data).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Error, ErrorKind};

    use hotaru_core::connection::HotaruRead;
    use hotaru_io_tokio::TokioIo;

    use crate::util::content_type::HttpContentType;
    use crate::message::meta::HttpMeta;
    use crate::protocol::HttpSafety;
    use crate::util::encoding::HttpEncoding;

    use super::*;

    #[tokio::test]
    async fn write_and_read_preserve_payload_and_message_boundaries() {
        for transfer in [None, Some("chunked".to_string())] {
            let encoding = HttpEncoding::from_headers(transfer, Some("gzip".to_string())).unwrap();
            for payload in [Vec::new(), b"hello\0\xffabc".to_vec()] {
                let body = HttpBody::Buffer {
                    data: payload.clone(),
                    content_type: HttpContentType::ApplicationOctetStream(),
                    content_coding: encoding.content().clone(),
                };
                let mut writer = TokioIo::new(Vec::new());
                body.write_buffer(&mut writer, encoding.transfer())
                    .await
                    .unwrap();
                let mut wire = writer.into_inner();

                let expected = if encoding.transfer().is_chunked() {
                    if payload.is_empty() {
                        b"0\r\n\r\n".to_vec()
                    } else {
                        b"a\r\nhello\0\xffabc\r\n0\r\n\r\n".to_vec()
                    }
                } else {
                    payload.clone()
                };
                assert_eq!(wire, expected);

                wire.extend_from_slice(b"next");
                let mut reader = TokioIo::new(tokio::io::BufReader::new(Cursor::new(wire)));
                let mut meta = HttpMeta::default();
                meta.set_content_type(HttpContentType::ApplicationOctetStream());
                meta.set_encoding(Some(encoding.clone()));
                if encoding.transfer().is_identity() {
                    meta.set_content_length(payload.len());
                }
                let body = HttpBody::read_buffer(&mut reader, &mut meta, &HttpSafety::default())
                    .await
                    .unwrap();
                let HttpBody::Buffer {
                    data,
                    content_coding,
                    ..
                } = body
                else {
                    panic!("expected a body buffer");
                };
                assert_eq!(data, payload);
                assert_eq!(&content_coding, encoding.content());
                let mut remaining = [0; 4];
                reader.read_exact(&mut remaining).await.unwrap();
                assert_eq!(&remaining, b"next");
            }
        }
    }

    #[tokio::test]
    async fn write_buffer_propagates_mid_payload_io_failure() {
        let stream = tokio_test::io::Builder::new()
            .write(b"5\r\n")
            .write(b"he")
            .write_error(Error::from(ErrorKind::BrokenPipe))
            .build();
        let mut writer = TokioIo::new(stream);
        let encoding = HttpEncoding::from_headers(Some("chunked".to_string()), None).unwrap();
        let result = HttpBody::Text("hello".to_string())
            .encode(Default::default())
            .unwrap()
            .write_buffer(&mut writer, encoding.transfer())
            .await;
        assert!(
            matches!(result, Err(HttpError::Io(error)) if error.kind() == ErrorKind::BrokenPipe)
        );
    }

    #[tokio::test]
    async fn write_buffer_rejects_unprepared_bodies_before_writing() {
        for body in [
            HttpBody::Text("hello".to_string()),
            HttpBody::Binary(vec![0xff]),
            HttpBody::Empty,
            HttpBody::Unparsed,
        ] {
            let mut writer = TokioIo::new(Vec::new());
            let result = body
                .write_buffer(&mut writer, &TransferCodings::new())
                .await;
            assert!(matches!(
                result,
                Err(HttpError::Body(BodyError::NotBuffered))
            ));
            assert!(writer.into_inner().is_empty());
        }
    }
}
