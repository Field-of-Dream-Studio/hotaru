use super::HttpMeta;
use std::collections::HashSet;

impl HttpMeta {
    /// Serializes the HTTP meta data to a string representation.
    ///
    /// This method generates a properly formatted HTTP header section,
    /// including the start line and all headers.
    ///
    /// # Returns
    ///
    /// A string containing the start line and all headers, formatted
    /// according to the HTTP protocol.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::http_value::{HttpStartLine, HttpVersion, HttpMethod};
    /// use std::collections::HashMap;
    ///
    /// // Create a request meta
    /// let mut meta = HttpMeta::new(
    ///     HttpStartLine::new_request(
    ///         HttpVersion::Http11,
    ///         HttpMethod::GET,
    ///         "/index.html".to_string()
    ///     ),
    ///     HashMap::new()
    /// );
    /// meta.set_header("host", "example.com");
    ///
    /// let http_string = meta.represent();
    /// assert!(http_string.starts_with("GET /index.html HTTP/1.1\r\n"));
    /// assert!(http_string.contains("host: example.com\r\n"));
    /// assert!(http_string.ends_with("\r\n\r\n"));
    /// ```
    pub fn represent(&self) -> String {
        let mut result = String::new();
        let mut handled_headers = HashSet::new();

        // Add the start line (works for both request and response)
        result.push_str(&format!("{}\r\n", self.start_line));

        // Process field values first (they have priority)

        // Add content-type if present
        if let Some(ref content_type) = self.content_type {
            result.push_str(&format!("content-type: {}\r\n", content_type));
            handled_headers.insert("content-type".to_string());
        }

        // Add content-length if present
        if let Some(content_length) = self.content_length {
            result.push_str(&format!("content-length: {}\r\n", content_length));
            handled_headers.insert("content-length".to_string());
        }

        // Add content-disposition if present
        if let Some(ref content_disposition) = self.content_disposition {
            result.push_str(&format!(
                "content-disposition: {}\r\n",
                content_disposition.to_string()
            ));
            handled_headers.insert("content-disposition".to_string());
        }

        // Add host if present
        if let Some(ref host) = self.host {
            result.push_str(&format!("host: {}\r\n", host));
            handled_headers.insert("host".to_string());
        }

        // Add language if present
        if let Some(ref lang) = self.lang {
            if self.start_line.is_request() {
                result.push_str(&format!("accept-language: {}\r\n", lang.to_header_string()));
                handled_headers.insert("host".to_string());
            } else {
                result.push_str(&format!(
                    "content-language: {}\r\n",
                    lang.to_response_header()
                ));
                handled_headers.insert("content-language".to_string());
            }
        }

        // Add location if present
        if let Some(ref location) = self.location {
            result.push_str(&format!("location: {}\r\n", location));
            handled_headers.insert("location".to_string());
        }

        // Add transfer-encoding if present
        if let Some(ref transfer_encoding) = self.encoding {
            let (transfer, content) = transfer_encoding.to_headers();
            if let Some(transfer) = transfer {
                result.push_str(&format!("transfer-encoding: {}\r\n", transfer));
                handled_headers.insert("transfer-encoding".to_string());
            }
            if let Some(content) = content {
                result.push_str(&format!("content-encoding: {}\r\n", content));
                handled_headers.insert("content-encoding".to_string());
            }
        }

        // Add cookies based on whether this is a request or response
        if let Some(ref cookies) = self.cookies {
            if self.start_line.is_request() {
                // For requests, we use the Cookie header
                let cookie_header = cookies.request();
                if !cookie_header.is_empty() {
                    result.push_str(&format!("{}\r\n", cookie_header));
                    handled_headers.insert("cookie".to_string());
                }
            } else {
                // For responses, we use Set-Cookie headers
                let cookie_header = cookies.response();
                if !cookie_header.is_empty() {
                    result.push_str(&format!(
                        "{}",
                        cookie_header.into_header_string("set-cookie")
                    ));
                    handled_headers.insert("set-cookie".to_string());
                }
            }
        }

        // Now process any remaining headers from the hashmap
        for (key, value) in &self.header {
            if !handled_headers.contains(key) {
                result.push_str(&value.into_header_string(key));
            }
        }

        // End headers with an extra CRLF
        result.push_str("\r\n");

        result
    }
}
