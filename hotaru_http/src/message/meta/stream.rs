use super::HttpMeta;
use crate::connection::error::ConnectionError;
use crate::message::header::{HeaderMap, HeaderValue};
use crate::message::start_line::HttpStartLine;
use crate::security::safety::HttpSafety;
use hotaru_core::connection::{HotaruBufRead, TransferTermination};
use std::collections::HashMap;

impl HttpMeta {
    pub async fn from_stream<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
        is_request: bool,
    ) -> Result<HttpMeta, ConnectionError> {
        let mut headers = Self::header_lines_raw_from_stream(buf_reader, config, print_raw)
            .await
            .map_err(|_| ConnectionError::BadRequest(format!("Failed to read headers")))?;

        if headers.is_empty() {
            return Err(ConnectionError::BadRequest(format!(
                "Empty {}",
                if is_request { "request" } else { "response" }
            )));
        }

        // Parse the start line according to whether it's a request or response
        let start_line = Self::parse_start_line(&headers.remove(0), is_request);

        // Parse headers with special handling for specific header names
        let header = Self::parse_headers(headers, is_request);

        if print_raw {
            println!("Parsed headers: {:?}", header);
            println!("Parsed start line: {:?}", start_line);
        }

        let mut meta = HttpMeta::new(start_line, header);

        if meta.header.contains_key("content-length")
            && meta.header.contains_key("transfer-encoding")
        {
            return Err(ConnectionError::BadRequest(
                "Content-Length cannot be combined with Transfer-Encoding".to_string(),
            ));
        }

        meta.parse_content_length()?;

        Ok(meta)
    }

    async fn header_lines_raw_from_stream<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<Vec<String>, ConnectionError> {
        let mut headers = Vec::new();
        let mut total_header_size = 0;

        // Try to fill the buffer with a single read first
        let buffer = buf_reader
            .fill_buf()
            .await
            .map_err(|_| ConnectionError::InternalServerError(format!("Failed to fill buffer")))?;

        // Fast path: Check if we got all headers in one go
        // Extract result first, then drop buffer borrow before calling consume
        let fast_path_result = Self::extract_headers_from_buffer(buffer, config);

        if let Some((header_lines, headers_end)) = fast_path_result {
            // We found the complete headers in the buffer
            if print_raw {
                println!("Fast path: got all headers in single read");
            }

            // Process headers from buffer
            for line in header_lines {
                if !config.check_line_length(line.len()) {
                    return Err(ConnectionError::BadRequest(format!("Header line too long")));
                }

                total_header_size += line.len() + 2; // +2 for CRLF 

                if !config.check_header_size(total_header_size) {
                    return Err(ConnectionError::BadRequest(format!("Headers too large")));
                }

                if !config.check_headers_count(headers.len()) {
                    return Err(ConnectionError::BadRequest(format!("Too many headers")));
                }

                // Strip CRLF injection and store
                let safe_line = line.replace("\r", "");
                headers.push(safe_line);
            }

            // Consume the processed data from the buffer
            buf_reader.consume(headers_end);
        } else {
            // Slow path: read headers line by line as before
            if print_raw {
                println!("Slow path: reading headers line by line");
            }

            loop {
                let mut line = String::new();
                let outcome = buf_reader
                    .read_line(&mut line, config.effective_line_length())
                    .await?;
                if outcome.termination == TransferTermination::CapReached {
                    return Err(ConnectionError::PayloadTooLarge);
                }
                let bytes_read = outcome.transferred;
                if print_raw {
                    println!("Read line: {}, buffer: {}", line, bytes_read);
                }

                if bytes_read == 0 || line.trim_end().is_empty() {
                    // println!("[End of headers] No more lines to read, 0 bytes read {}, empty line: {}", bytes_read, line.trim_end().is_empty());
                    break; // End of headers
                }

                total_header_size += line.len();

                // Enforce max header size limit
                if !config.check_header_size(total_header_size) {
                    // println!("[Headers too large] Total header size: {}, allowed: {}", total_header_size, config.effective_header_size());
                    return Err(ConnectionError::PayloadTooLarge);
                }

                // Enforce max number of headers
                if !config.check_headers_count(headers.len()) {
                    // println!("[Too many headers] Current header count: {}", headers.len());
                    return Err(ConnectionError::PayloadTooLarge);
                }

                // Strip CRLF injection and store the header
                let safe_line = line.trim_end().replace("\r", "");
                headers.push(safe_line);
            }
        }

        Ok(headers)
    }

    // Helper function to parse the start line
    fn parse_start_line(line: &str, is_request: bool) -> HttpStartLine {
        if is_request {
            HttpStartLine::parse_request(line)
        } else {
            HttpStartLine::parse_response(line)
        }
    }

    // Helper function to parse headers with special handling for specific header types
    fn parse_headers(header_lines: Vec<String>, _is_response: bool) -> HeaderMap {
        let mut headers: HashMap<String, HeaderValue> = HashMap::new();

        // // List of headers that should not be combined (kept as separate values)
        // // This is especially important for responses with multiple Set-Cookie headers
        // let non_combinable_headers: HashSet<&str> = [
        //     "set-cookie",
        //     // Add other headers that should not be combined if needed
        // ].iter().cloned().collect();

        for line in header_lines {
            if let Some(colon_pos) = line.find(':') {
                let (key, value) = line.split_at(colon_pos);

                // Normalize the header name (case-insensitive in HTTP)
                let header_name = key.trim().to_lowercase();

                // Remove the colon and trim whitespace from the value
                let header_value = value[1..].trim().to_string();

                // Check if this is a special header that should not be combined
                // let is_non_combinable = is_response && non_combinable_headers.contains(header_name.as_str());

                match headers.get_mut(&header_name) {
                    Some(existing_value) => {
                        existing_value.add_without_combining(header_value);
                        // For special headers like Set-Cookie, add without combining
                        // if is_non_combinable {
                        //     existing_value.add_without_combining(header_value);
                        // } else {
                        //     // For regular headers, append (typically combined with commas)
                        //     existing_value.append(header_value);
                        // }
                    }
                    None => {
                        // First occurrence of this header
                        headers.insert(header_name, HeaderValue::new(header_value));
                    }
                }
            }
        }

        headers.into()
    }

    // Expose the specific methods that call the shared implementation
    pub async fn from_request_stream<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<HttpMeta, ConnectionError> {
        Self::from_stream(buf_reader, config, print_raw, true).await
    }

    pub async fn append_from_request_stream<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        &mut self,
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<(), ConnectionError> {
        let mut headers = Self::header_lines_raw_from_stream(buf_reader, config, print_raw).await?;

        if headers.is_empty() {
            return Ok(());
        }

        // Parse the start line
        let start_line = Self::parse_start_line(&headers.remove(0), true);

        // Parse headers
        let header = Self::parse_headers(headers, true);

        if print_raw {
            println!("Parsed request headers: {:?}", header);
            println!("Parsed request start line: {:?}", start_line);
        }

        self.start_line = start_line;
        self.header.extend(header);

        Ok(())
    }

    pub async fn from_response_stream<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<HttpMeta, ConnectionError> {
        Self::from_stream(buf_reader, config, print_raw, false).await
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
