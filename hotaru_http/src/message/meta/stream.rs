use super::HttpMeta;
use super::error::{MetaError, StreamedMetaError};
use crate::message::header::{HeaderLine, HeaderMap, HeaderValue};
use crate::message::start_line::HttpStartLine;
use crate::security::safety::HttpSafety;
use crate::start_line::StartLineError;
use crate::util::streamed::Streamed;
use hotaru_core::connection::{HotaruBufRead, TransferTermination};
use std::collections::HashMap;

impl HttpMeta {
    pub async fn from_stream<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
        is_request: bool,
    ) -> Result<HttpMeta, StreamedMetaError> {
        let mut headers = Self::header_lines_raw_from_stream(buf_reader, config, print_raw).await?;

        if headers.is_empty() {
            return Err(Streamed::Err(MetaError::from(
                crate::message::start_line::StartLineError::Empty,
            )));
        }

        // Parse the start line according to whether it's a request or response
        let start_line = Self::parse_start_line(&headers.remove(0), is_request)?;

        // Parse headers with special handling for specific header names
        let header = Self::parse_headers(headers, is_request)?;

        if print_raw {
            println!("Parsed headers: {:?}", header);
            println!("Parsed start line: {:?}", start_line);
        }

        let mut meta = HttpMeta::new(start_line, header);

        if meta.header.contains_key("content-length")
            && meta.header.contains_key("transfer-encoding")
        {
            return Err(Streamed::Err(MetaError::ConflictingFraming));
        }

        meta.parse_content_length().map_err(Streamed::Err)?;
        meta.parse_connection()
            .map_err(MetaError::from)
            .map_err(Streamed::Err)?;

        Ok(meta)
    }

    async fn header_lines_raw_from_stream<
        R: HotaruBufRead<Error = std::io::Error> + Unpin + Send,
    >(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<Vec<String>, StreamedMetaError> {
        let mut headers = Vec::new();
        let mut total_header_size = 0;

        // Try to fill the buffer with a single read first
        let buffer = buf_reader.fill_buf().await?;

        // Fast path: Check if we got all headers in one go
        let fast_path_result = Self::extract_headers_from_buffer(buffer, config);

        if let Some((header_lines, headers_end)) = fast_path_result {
            // We found the complete headers in the buffer
            if print_raw {
                println!("Fast path: got all headers in single read");
            }

            // Process headers from buffer
            for line in header_lines {
                if !config.check_line_length(line.len()) {
                    return Err(Streamed::Err(MetaError::HeaderLineTooLong));
                }

                total_header_size += line.len() + 2; // +2 for CRLF

                if !config.check_header_size(total_header_size) {
                    return Err(Streamed::Err(MetaError::HeadersTooLarge));
                }

                if !config.check_headers_count(headers.len()) {
                    return Err(Streamed::Err(MetaError::TooManyHeaders));
                }

                headers.push(line.to_string());
            }

            // Consume the processed data from the buffer
            buf_reader.consume(headers_end);
        } else {
            // Slow path: read headers line by line
            if print_raw {
                println!("Slow path: reading headers line by line");
            }

            loop {
                let mut line = String::new();
                let outcome = buf_reader
                    .read_line(&mut line, config.effective_line_length())
                    .await?;
                if outcome.termination == TransferTermination::CapReached {
                    return Err(Streamed::Err(MetaError::HeaderLineTooLong));
                }
                let bytes_read = outcome.transferred;
                if print_raw {
                    println!("Read line: {}, buffer: {}", line, bytes_read);
                }

                if bytes_read == 0 {
                    break;
                }

                let line = line.strip_suffix('\n').unwrap_or(&line);
                let line = line.strip_suffix('\r').unwrap_or(line);

                if line.is_empty() {
                    break; // End of headers
                }

                total_header_size += outcome.transferred;

                // Enforce max header size limit
                if !config.check_header_size(total_header_size) {
                    return Err(Streamed::Err(MetaError::HeadersTooLarge));
                }

                // Enforce max number of headers
                if !config.check_headers_count(headers.len()) {
                    return Err(Streamed::Err(MetaError::TooManyHeaders));
                }

                headers.push(line.to_string());
            }
        }

        Ok(headers)
    }

    // Helper function to parse the start line
    pub fn parse_start_line_or_default(line: &str, is_request: bool) -> HttpStartLine {
        if is_request {
            HttpStartLine::parse_request_or_default(line)
        } else {
            HttpStartLine::parse_response_or_default(line)
        }
    }

    pub fn parse_start_line(line: &str, is_request: bool) -> Result<HttpStartLine, StartLineError> {
        if is_request {
            HttpStartLine::parse_request(line)
        } else {
            HttpStartLine::parse_response(line)
        }
    }

    // Helper function to parse headers with special handling for specific header types
    fn parse_headers(
        header_lines: Vec<String>,
        _is_response: bool,
    ) -> Result<HeaderMap, MetaError> {
        let mut headers: HashMap<String, HeaderValue> = HashMap::new();

        for line in header_lines {
            let (header_name, header_value) = HeaderLine::parse(&line)?.into_parts();

            match headers.get_mut(&header_name) {
                Some(existing_value) => {
                    existing_value.add_without_combining(header_value);
                }
                None => {
                    // First occurrence of this header
                    headers.insert(header_name, HeaderValue::new(header_value));
                }
            }
        }

        Ok(headers.into())
    }

    // Expose the specific methods that call the shared implementation
    pub async fn from_request_stream<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<HttpMeta, StreamedMetaError> {
        Self::from_stream(buf_reader, config, print_raw, true).await
    }

    pub async fn from_response_stream<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<HttpMeta, StreamedMetaError> {
        Self::from_stream(buf_reader, config, print_raw, false).await
    }

    /// Reads a bare header block and merges it into `self.header`.
    /// No start line is parsed — every line is treated as a header.
    /// Used for chunked trailers and any other header-only append.
    pub async fn append_headers_from_stream<
        R: HotaruBufRead<Error = std::io::Error> + Unpin + Send,
    >(
        &mut self,
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<(), StreamedMetaError> {
        let headers = Self::header_lines_raw_from_stream(buf_reader, config, print_raw).await?;

        if headers.is_empty() {
            return Ok(());
        }

        let header = Self::parse_headers(headers, true)?;

        if print_raw {
            println!("Parsed trailers: {:?}", header);
        }

        if header.contains_key("connection") {
            self.connection = None;
        }
        self.header.extend(header);
        Ok(())
    }

    /// Helper function to extract complete headers from a buffer if possible
    fn extract_headers_from_buffer<'a>(
        buffer: &'a [u8],
        config: &HttpSafety,
    ) -> Option<(Vec<&'a str>, usize)> {
        // Look for the end of headers marker (double CRLF)
        let mut i = 0;
        while i + 3 < buffer.len() {
            if buffer[i] == b'\r'
                && buffer[i + 1] == b'\n'
                && buffer[i + 2] == b'\r'
                && buffer[i + 3] == b'\n'
            {
                // Found end of headers
                let headers_section = std::str::from_utf8(&buffer[..i + 2]).ok()?;

                // Split into lines
                let lines: Vec<&str> = headers_section
                    .split("\r\n")
                    .filter(|s| !s.is_empty())
                    .collect();

                if !config.check_headers_count(lines.len()) {
                    return None; // Too many headers, fall back to slow path
                }

                return Some((lines, i + 4)); // +4 to include the final \r\n\r\n
            }
            i += 1;
        }

        None // Didn't find complete headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::http_value::StatusCode;
    use crate::message::start_line::StartLineError;
    use hotaru_io_tokio::TokioIo;
    use std::io::Cursor;
    use tokio::io::BufReader;

    async fn parse_request_head(input: &[u8]) -> Result<HttpMeta, StreamedMetaError> {
        let cursor = Cursor::new(input.to_vec());
        let mut reader = TokioIo::new(BufReader::new(cursor));

        HttpMeta::from_request_stream(&mut reader, &HttpSafety::default(), false).await
    }

    fn assert_invalid_header(result: Result<HttpMeta, StreamedMetaError>) {
        match result {
            Err(Streamed::Err(error @ MetaError::HeaderLine(_))) => {
                assert_eq!(StatusCode::from(&error), StatusCode::BAD_REQUEST);
                assert!(!error.can_continue());
            }
            other => panic!("expected invalid header error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn malformed_request_line_is_not_defaulted_to_root_get() {
        let cases: &[&[u8]] = &[
            b"GE T / HTTP/1.1\r\nHost: example.test\r\n\r\n",
            b" / HTTP/1.1\r\nHost: example.test\r\n\r\n",
            b"GET /\r\nHost: example.test\r\n\r\n",
        ];

        for case in cases {
            let result = parse_request_head(case).await;

            assert!(matches!(
                result,
                Err(Streamed::Err(MetaError::StartLine(
                    StartLineError::Unrecognised
                )))
            ));
        }
    }

    #[tokio::test]
    async fn empty_request_head_returns_start_line_error() {
        let result = parse_request_head(b"\r\n").await;

        assert!(matches!(
            result,
            Err(Streamed::Err(MetaError::StartLine(StartLineError::Empty)))
        ));
    }

    #[tokio::test]
    async fn header_value_with_cr_is_invalid() {
        let result =
            parse_request_head(b"GET / HTTP/1.1\r\nHost: example.test\r\nX-Test: a\rb\r\n\r\n")
                .await;

        assert_invalid_header(result);
    }

    #[tokio::test]
    async fn header_value_with_nul_is_invalid() {
        let result =
            parse_request_head(b"GET / HTTP/1.1\r\nHost: example.test\r\nX-Test: a\0b\r\n\r\n")
                .await;

        assert_invalid_header(result);
    }

    #[tokio::test]
    async fn whitespace_before_colon_is_invalid() {
        let result =
            parse_request_head(b"GET / HTTP/1.1\r\nHost: example.test\r\nX : v\r\n\r\n").await;

        assert_invalid_header(result);
    }

    #[tokio::test]
    async fn obfuscated_transfer_encoding_is_invalid_before_framing() {
        let result = parse_request_head(
            b"POST / HTTP/1.1\r\nHost: example.test\r\nTransfer-Encoding : chunked\r\n\r\n0\r\n\r\n",
        )
        .await;

        assert_invalid_header(result);
    }

    #[tokio::test]
    async fn valid_header_is_parsed_normally() {
        let meta = parse_request_head(b"GET / HTTP/1.1\r\nHost: example.test\r\nX-Test: v\r\n\r\n")
            .await
            .unwrap();

        assert_eq!(meta.header.get("x-test").unwrap().first(), "v");
    }
}
