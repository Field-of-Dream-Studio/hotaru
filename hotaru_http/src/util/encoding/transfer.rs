use super::error::EncodingError;

/// Represents HTTP transfer coding types as defined in HTTP standards.
///
/// Transfer codings are primarily used to define the message transfer format
/// between HTTP nodes. The most common is "chunked" encoding.
#[derive(Debug, Clone, PartialEq)]
pub enum TransferCoding {
    /// Chunked transfer encoding, where the message body is divided into a series
    /// of chunks, each with its own size indicator.
    Chunked,

    /// Any other transfer encoding not explicitly defined in this enum.
    Other(Box<str>),
}

impl TransferCoding {
    /// Creates a new `TransferCoding` from a string.
    ///
    /// The string is trimmed and converted to lowercase before matching.
    ///
    /// # Arguments
    ///
    /// * `s` - The string representation of the transfer coding
    ///
    /// # Returns
    ///
    /// A `TransferCoding` variant corresponding to the provided string
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::TransferCoding;
    ///
    /// let coding = TransferCoding::from_string("chunked");
    /// assert!(matches!(coding, TransferCoding::Chunked));
    ///
    /// let coding = TransferCoding::from_string("compress");
    /// assert!(matches!(coding, TransferCoding::Other(_)));
    /// ```
    pub fn from_string(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "chunked" => TransferCoding::Chunked,
            other => TransferCoding::Other(other.into()),
        }
    }

    /// Returns the string representation of this transfer coding.
    ///
    /// # Returns
    ///
    /// A string slice representing the transfer coding
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::TransferCoding;
    ///
    /// let coding = TransferCoding::Chunked;
    /// assert_eq!(coding.as_str(), "chunked");
    ///
    /// let coding = TransferCoding::Other("custom".into());
    /// assert_eq!(coding.as_str(), "custom");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Self::Chunked => "chunked",
            Self::Other(s) => s,
        }
    }
}

/// A collection of transfer codings with validation according to HTTP standards.
///
/// This struct ensures that:
/// - "chunked" appears at most once
/// - "chunked" is always the last transfer coding
#[derive(Debug, Clone, Default)]
pub struct TransferCodings {
    codings: Vec<TransferCoding>,
}

impl TransferCodings {
    /// Creates a new empty `TransferCodings` collection.
    ///
    /// # Returns
    ///
    /// A new `TransferCodings` instance
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::TransferCodings;
    ///
    /// let codings = TransferCodings::new();
    /// assert!(codings.is_identity());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a transfer coding to the collection, with validation.
    ///
    /// According to HTTP standards:
    /// - "chunked" can appear at most once
    /// - "chunked" must be the last transfer coding
    ///
    /// # Arguments
    ///
    /// * `coding` - The transfer coding to add
    ///
    /// # Returns
    ///
    /// `Ok(())` if the coding was successfully added, or an error message
    /// explaining why the coding could not be added.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::{TransferCodings, TransferCoding};
    ///
    /// let mut codings = TransferCodings::new();
    ///
    /// // Chunked is the only supported transfer coding.
    /// codings.push(TransferCoding::Chunked).unwrap();
    ///
    /// // Unknown transfer codings are rejected (RFC 9112 §6.1).
    /// let mut codings = TransferCodings::new();
    /// assert!(codings.push(TransferCoding::Other("gzip".into())).is_err());
    ///
    /// // Chunked cannot appear twice.
    /// let mut codings = TransferCodings::new();
    /// codings.push(TransferCoding::Chunked).unwrap();
    /// assert!(codings.push(TransferCoding::Chunked).is_err());
    /// ```
    pub fn push(&mut self, coding: TransferCoding) -> Result<(), EncodingError> {
        // Reject unknown transfer codings — RFC 9112 §6.1: server MUST return
        // 501 Not Implemented for a Transfer-Encoding it cannot honor.
        if let TransferCoding::Other(name) = &coding {
            return Err(EncodingError::UnsupportedTransferCoding(name.to_string()));
        }
        if matches!(coding, TransferCoding::Chunked) {
            if self
                .codings
                .iter()
                .any(|c| matches!(c, TransferCoding::Chunked))
            {
                return Err(EncodingError::DuplicateChunked);
            }
        } else if self
            .codings
            .last()
            .is_some_and(|c| matches!(c, TransferCoding::Chunked))
        {
            return Err(EncodingError::CodingAfterChunked);
        }

        self.codings.push(coding);
        Ok(())
    }

    /// Checks if chunked transfer encoding is used.
    ///
    /// # Returns
    ///
    /// `true` if chunked encoding is present, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::{TransferCodings, TransferCoding};
    ///
    /// let mut codings = TransferCodings::new();
    /// assert!(!codings.is_chunked());
    ///
    /// codings.push(TransferCoding::Chunked).unwrap();
    /// assert!(codings.is_chunked());
    /// ```
    pub fn is_chunked(&self) -> bool {
        self.codings
            .iter()
            .any(|c| matches!(c, TransferCoding::Chunked))
    }

    /// Checks if identity transfer encoding is used (no transfer encoding).
    ///
    /// # Returns
    ///
    /// `true` if no transfer encodings are present, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::{TransferCodings, TransferCoding};
    ///
    /// let mut codings = TransferCodings::new();
    /// assert!(codings.is_identity());
    ///
    /// codings.push(TransferCoding::Chunked).unwrap();
    /// assert!(!codings.is_identity());
    /// ```
    pub fn is_identity(&self) -> bool {
        self.codings.is_empty()
    }

    /// Converts the transfer codings to a header value string.
    ///
    /// # Returns
    ///
    /// A comma-separated string of transfer codings
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::{TransferCodings, TransferCoding};
    ///
    /// let mut codings = TransferCodings::new();
    /// codings.push(TransferCoding::Chunked).unwrap();
    ///
    /// assert_eq!(codings.to_header(), "chunked");
    /// ```
    pub fn to_header(&self) -> String {
        self.codings
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
