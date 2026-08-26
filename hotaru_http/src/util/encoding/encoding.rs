use super::{ContentCoding, ContentCodings, EncodingError, TransferCoding, TransferCodings};

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
    /// Fails with [`EncodingError`] if the Transfer-Encoding header violates
    /// framing rules (duplicate `chunked` or a coding listed after `chunked`).
    /// Content-Encoding tokens are accepted lenient — unknown tokens land as
    /// [`ContentCoding::Other`] and are validated at apply-time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::HttpEncoding;
    ///
    /// let encoding = HttpEncoding::from_headers(
    ///     Some("chunked".to_string()),
    ///     Some("br".to_string()),
    /// ).unwrap();
    ///
    /// assert!(encoding.transfer().is_chunked());
    /// assert!(!encoding.content().is_identity());
    /// ```
    pub fn from_headers(
        transfer_header: Option<String>,
        content_header: Option<String>,
    ) -> Result<Self, EncodingError> {
        let mut transfer = TransferCodings::new();
        let mut content = ContentCodings::new();

        if let Some(header) = transfer_header {
            for part in header.split(',') {
                if !part.trim().is_empty() {
                    transfer.push(TransferCoding::from_string(part))?;
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

        Ok(Self { transfer, content })
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
    ///     Some("gzip".to_string()),
    /// ).unwrap();
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
    ///     None,
    /// ).unwrap();
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
    ///     Some("gzip, br".to_string()),
    /// ).unwrap();
    ///
    /// assert!(!encoding.content().is_identity());
    /// ```
    pub fn content(&self) -> &ContentCodings {
        &self.content
    }
}
