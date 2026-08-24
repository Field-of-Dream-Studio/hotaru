use crate::message::body::{BodyError, ChunkingError};
use crate::message::meta::MetaError;
use crate::protocol::HttpError;
use crate::security::safety::HttpSafety;
use crate::util::encoding::ContentCodings;

use crate::message::http_value::*;
use crate::message::meta::HttpMeta;
use crate::util::form::*;
use akari::Value;
use hotaru_core::connection::{HotaruBufRead, TransferTermination};

#[derive(Debug, Clone)]
pub enum HttpBody {
    Text(String),
    Binary(Vec<u8>),
    Form(UrlEncodedForm),
    Files(MultiForm),
    Json(Value),
    Empty,
    Unparsed,

    Buffer {
        data: Vec<u8>,
        content_type: HttpContentType,
        content_coding: ContentCodings,
    },
}

impl HttpBody {
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
        Ok(buffer.parse_buffer(parse_config)?)
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
}

impl Default for HttpBody {
    fn default() -> Self {
        Self::Unparsed
    }
}
