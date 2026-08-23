use crate::security::safety::HttpSafety;
use crate::util::encoding::ContentCodings;

use crate::message::http_value::*;
use crate::message::meta::HttpMeta;
use crate::util::form::*;
use akari::Value;
use hotaru_core::connection::{HotaruBufRead, TransferTermination};

use super::BodyError;

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
    ) -> std::io::Result<Self> {
        Ok(Self::Buffer {
            data: Self::read_binary_info(buf_reader, header, parse_config).await?,
            content_type: header
                .get_content_type()
                .unwrap_or(HttpContentType::from_str("")),
            content_coding: header
                .get_encoding()
                .map(|e| e.content().clone())
                .map_err(|error| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
                })?,
        })
    }

    /// Parse the HTTP body directly from a TCP Stream
    pub async fn direct_parse<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        header: &mut HttpMeta,
        parse_config: &HttpSafety,
    ) -> Result<Self, BodyError> {
        let buffer = Self::read_buffer(buf_reader, header, parse_config).await?;
        buffer.parse_buffer(parse_config)
    }

    pub async fn read_binary_info<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        header: &mut HttpMeta,
        parse_config: &HttpSafety,
    ) -> std::io::Result<Vec<u8>> {
        /// Reads body with Content-Length
        async fn read_content_length_body<
            R: HotaruBufRead<Error = std::io::Error> + Unpin + Send,
        >(
            buf_reader: &mut R,
            safety_setting: &HttpSafety,
            content_length: usize,
        ) -> std::io::Result<Vec<u8>> {
            // Security: reject before reading. Truncating (min(cl, cap)) would leave
            // the excess body bytes on the wire and let the keep-alive loop parse
            // them as the next request — CL desync / request smuggling.
            if !safety_setting.check_body_size(content_length) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Content-Length exceeds maximum body size",
                ));
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
        ) -> std::io::Result<Vec<u8>> {
            let mut body_buffer = Vec::new();
            let mut current_size = 0;

            loop {
                // Read chunk size line
                let mut size_line = String::new();
                let outcome = buf_reader
                    .read_line(&mut size_line, safety_setting.effective_line_length())
                    .await?;
                if outcome.termination == TransferTermination::CapReached {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Chunk size line exceeds maximum length",
                    ));
                }
                let chunk_size_str = size_line.trim_end_matches(|c| c == '\r' || c == '\n');

                // Parse chunk size (validates hex format - critical for preventing crashes)
                let chunk_size = usize::from_str_radix(chunk_size_str, 16).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid chunk size")
                })?;

                if chunk_size == 0 {
                    break; // End of chunks
                }

                // Security: Cumulative size validation prevents chunked encoding DoS attacks.
                // This is the CORE security mechanism - validating size limits, not every byte.
                //
                // Protects against:
                // 1. Single giant chunk: e.g., chunk_size = 1GB rejected immediately.
                // 2. Multiple chunks exceeding limit: e.g., 9 bytes + 9 bytes when limit is 10.
                // 3. Death by a thousand cuts: many small chunks accumulating beyond limit.
                // 4. Integer overflow: attacker crafts chunk sizes whose sum wraps `usize`,
                //    which would bypass a plain `+=` guard (PR #26). `check_body_size_delta`
                //    uses checked addition and rejects both overflow and over-cap.
                //
                // Validation happens BEFORE memory allocation, so attacker cannot force
                // excessive allocation by sending large chunk size declarations.
                current_size = match safety_setting.check_body_size_delta(current_size, chunk_size) {
                    Some(new_total) => new_total,
                    None => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Chunked body exceeds maximum size",
                        ));
                    }
                };

                // Read chunk data (only reached if validation passed)
                let mut chunk_data = vec![0; chunk_size];
                buf_reader.read_exact(&mut chunk_data).await?;
                body_buffer.extend_from_slice(&chunk_data);

                // Read trailing CRLF
                let mut crlf = [0; 2];
                buf_reader.read_exact(&mut crlf).await?;
                if crlf != [b'\r', b'\n'] {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid chunk terminator",
                    ));
                }
            }

            // Read trailing headers (if any)
            header
                .append_from_request_stream(buf_reader, safety_setting, false)
                .await
                .map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::NetworkUnreachable,
                        "Error parsing headers",
                    )
                })?;

            Ok(body_buffer)
        }

        // Validate Content-Length before selecting the framing mode. This also
        // protects callers that construct HttpMeta directly instead of using
        // HttpMeta::from_stream.
        let content_length = header
            .get_content_length()
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;

        if content_length.is_some() && header.header.contains_key("transfer-encoding") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Content-Length cannot be combined with Transfer-Encoding",
            ));
        }

        // Read raw body data
        let encoding = header
            .get_encoding()
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let raw_data = if encoding.transfer().is_chunked() {
            read_chunked_body(buf_reader, header, parse_config).await?
        } else {
            let content_length = usize::try_from(content_length.unwrap_or(0)).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Content-Length exceeds the platform limit",
                )
            })?;
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
