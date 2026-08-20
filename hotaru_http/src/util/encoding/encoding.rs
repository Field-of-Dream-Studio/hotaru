use super::{ContentCoding, ContentCodings, TransferCoding, TransferCodings};

/// Combines HTTP transfer and content encodings into a single structure.
///
/// This struct handles both Transfer-Encoding and Content-Encoding HTTP headers.
#[derive(Debug, Clone, Default)]
pub struct HttpEncoding {
    transfer: TransferCodings,
    content: ContentCodings,
}

impl HttpEncoding {
    /// Creates a new `HttpEncoding` from HTTP header values.
    ///
    /// # Arguments
    ///
    /// * `transfer_header` - Optional Transfer-Encoding header value
    /// * `content_header` - Optional Content-Encoding header value
    ///
    /// # Returns
    ///
    /// A new `HttpEncoding` instance parsed from the provided headers
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::HttpEncoding;
    ///
    /// let encoding = HttpEncoding::from_headers(
    ///     Some("chunked, gzip".to_string()),
    ///     Some("br".to_string())
    /// );
    ///
    /// assert!(encoding.transfer().is_chunked());
    /// assert!(!encoding.content().is_identity());
    /// ```
    pub fn from_headers(transfer_header: Option<String>, content_header: Option<String>) -> Self {
        let mut transfer = TransferCodings::new();
        let mut content = ContentCodings::new();

        if let Some(header) = transfer_header {
            for part in header.split(',') {
                if !part.trim().is_empty() {
                    let coding = TransferCoding::from_string(part);
                    if let Err(e) = transfer.push(coding) {
                        eprintln!("[WARN] Invalid Transfer-Encoding: {}", e);
                    }
                }
            }
        }

        if let Some(header) = content_header {
            for part in header.split(',') {
                if !part.trim().is_empty() {
                    content.push(ContentCoding::from_string(part));
                }
            }
        }

        Self { transfer, content }
    }

    /// Converts the HTTP encodings to header values.
    ///
    /// # Returns
    ///
    /// A tuple of optional strings representing the Transfer-Encoding and
    /// Content-Encoding header values. If an encoding is identity (empty),
    /// its corresponding header value will be None.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::{HttpEncoding, TransferCoding, ContentCoding};
    ///
    /// let mut encoding = HttpEncoding::from_headers(
    ///     Some("chunked".to_string()),
    ///     Some("gzip".to_string())
    /// );
    ///
    /// let (transfer, content) = encoding.to_headers();
    /// assert_eq!(transfer, Some("chunked".to_string()));
    /// assert_eq!(content, Some("gzip".to_string()));
    /// ```
    pub fn to_headers(&self) -> (Option<String>, Option<String>) {
        let transfer = if !self.transfer.is_identity() {
            Some(self.transfer.to_header())
        } else {
            None
        };

        let content = if !self.content.is_identity() {
            Some(self.content.to_header())
        } else {
            None
        };

        (transfer, content)
    }

    /// Returns a reference to the transfer codings.
    ///
    /// # Returns
    ///
    /// A reference to the `TransferCodings` instance
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::HttpEncoding;
    ///
    /// let encoding = HttpEncoding::from_headers(
    ///     Some("chunked".to_string()),
    ///     None
    /// );
    ///
    /// assert!(encoding.transfer().is_chunked());
    /// ```
    pub fn transfer(&self) -> &TransferCodings {
        &self.transfer
    }

    /// Returns a reference to the content codings.
    ///
    /// # Returns
    ///
    /// A reference to the `ContentCodings` instance
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::HttpEncoding;
    ///
    /// let encoding = HttpEncoding::from_headers(
    ///     None,
    ///     Some("gzip, br".to_string())
    /// );
    ///
    /// assert!(!encoding.content().is_identity());
    /// ```
    pub fn content(&self) -> &ContentCodings {
        &self.content
    }
}
