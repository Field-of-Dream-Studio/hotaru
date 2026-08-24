#[cfg(feature = "compression")]
use hotaru_lib::compression;

use super::{CompressionError, CompressionFailure};

/// Represents HTTP content coding types as defined in HTTP standards.
///
/// Content codings are compression algorithms applied to the message body.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentCoding {
    /// gzip compression algorithm
    Gzip,

    /// deflate compression algorithm
    Deflate,

    /// compress compression algorithm
    Compress,

    /// Brotli compression algorithm (represented as "br" in HTTP headers)
    Brotli,

    /// Zstandard compression algorithm (represented as "zstd" in HTTP headers)
    Zstd,

    /// Any other content coding not explicitly defined in this enum
    Other(Box<str>),
}

impl ContentCoding {
    /// Creates a new `ContentCoding` from a string.
    ///
    /// The string is trimmed and converted to lowercase before matching.
    ///
    /// # Arguments
    ///
    /// * `s` - The string representation of the content coding
    ///
    /// # Returns
    ///
    /// A `ContentCoding` variant corresponding to the provided string
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::ContentCoding;
    ///
    /// let coding = ContentCoding::from_string("gzip");
    /// assert!(matches!(coding, ContentCoding::Gzip));
    ///
    /// let coding = ContentCoding::from_string("br");
    /// assert!(matches!(coding, ContentCoding::Brotli));
    /// ```
    pub fn from_string(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "gzip" => ContentCoding::Gzip,
            "deflate" => ContentCoding::Deflate,
            "compress" => ContentCoding::Compress,
            "br" => ContentCoding::Brotli,
            "zstd" => ContentCoding::Zstd,
            other => ContentCoding::Other(other.into()),
        }
    }

    /// Returns the string representation of this content coding.
    ///
    /// # Returns
    ///
    /// A string slice representing the content coding
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::ContentCoding;
    ///
    /// let coding = ContentCoding::Gzip;
    /// assert_eq!(coding.as_str(), "gzip");
    ///
    /// let coding = ContentCoding::Brotli;
    /// assert_eq!(coding.as_str(), "br");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Self::Gzip => "gzip",
            Self::Deflate => "deflate",
            Self::Compress => "compress",
            Self::Brotli => "br",
            Self::Zstd => "zstd",
            Self::Other(s) => s,
        }
    }

    pub fn decode_compressed(
        encoding: &ContentCoding,
        data: &[u8],
        max_size: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        match encoding {
            #[cfg(feature = "compression")]
            ContentCoding::Gzip => compression::decompress_gzip(data, max_size)
                .map_err(|_| CompressionError::DecodeFailed(CompressionFailure::InvalidStream)),
            #[cfg(feature = "compression")]
            ContentCoding::Deflate => compression::decompress_deflate(data, max_size)
                .map_err(|_| CompressionError::DecodeFailed(CompressionFailure::InvalidStream)),
            #[cfg(feature = "compression")]
            ContentCoding::Brotli => compression::decompress_brotli(data, max_size)
                .map_err(|_| CompressionError::DecodeFailed(CompressionFailure::InvalidStream)),
            #[cfg(feature = "compression")]
            ContentCoding::Zstd => compression::decompress_zstd(data, max_size)
                .map_err(|_| CompressionError::DecodeFailed(CompressionFailure::InvalidStream)),
            #[cfg(not(feature = "compression"))]
            ContentCoding::Gzip => {
                let _ = max_size;
                Err(CompressionError::Unavailable("gzip"))
            }
            #[cfg(not(feature = "compression"))]
            ContentCoding::Deflate => {
                let _ = max_size;
                Err(CompressionError::Unavailable("deflate"))
            }
            #[cfg(not(feature = "compression"))]
            ContentCoding::Brotli => {
                let _ = max_size;
                Err(CompressionError::Unavailable("br"))
            }
            #[cfg(not(feature = "compression"))]
            ContentCoding::Zstd => {
                let _ = max_size;
                Err(CompressionError::Unavailable("zstd"))
            }
            ContentCoding::Compress => Err(CompressionError::Unavailable("compress")),
            _ => Ok(data.to_vec()), // Identity or unknown coding — pass through.
        }
    }

    pub fn encode_compressed(
        encoding: &ContentCoding,
        data: &[u8],
    ) -> Result<Vec<u8>, CompressionError> {
        match encoding {
            #[cfg(feature = "compression")]
            ContentCoding::Gzip => compression::compress_gzip(data)
                .map_err(|_| CompressionError::EncodeFailed(CompressionFailure::InvalidStream)),
            #[cfg(feature = "compression")]
            ContentCoding::Deflate => compression::compress_deflate(data)
                .map_err(|_| CompressionError::EncodeFailed(CompressionFailure::InvalidStream)),
            #[cfg(feature = "compression")]
            ContentCoding::Brotli => compression::compress_brotli(data)
                .map_err(|_| CompressionError::EncodeFailed(CompressionFailure::InvalidStream)),
            #[cfg(feature = "compression")]
            ContentCoding::Zstd => compression::compress_zstd(data, 1)
                .map_err(|_| CompressionError::EncodeFailed(CompressionFailure::InvalidStream)),
            #[cfg(not(feature = "compression"))]
            ContentCoding::Gzip => Err(CompressionError::Unavailable("gzip")),
            #[cfg(not(feature = "compression"))]
            ContentCoding::Deflate => Err(CompressionError::Unavailable("deflate")),
            #[cfg(not(feature = "compression"))]
            ContentCoding::Brotli => Err(CompressionError::Unavailable("br")),
            #[cfg(not(feature = "compression"))]
            ContentCoding::Zstd => Err(CompressionError::Unavailable("zstd")),
            ContentCoding::Compress => Err(CompressionError::Unavailable("compress")),
            _ => Ok(data.to_vec()), // Identity or unknown coding — pass through.
        }
    }
}

/// A collection of content codings.
#[derive(Debug, Clone, Default)]
pub struct ContentCodings {
    codings: Vec<ContentCoding>,
}

impl ContentCodings {
    /// Creates a new empty `ContentCodings` collection.
    ///
    /// # Returns
    ///
    /// A new `ContentCodings` instance
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::ContentCodings;
    ///
    /// let codings = ContentCodings::new();
    /// assert!(codings.is_identity());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a content coding to the collection.
    ///
    /// # Arguments
    ///
    /// * `coding` - The content coding to add
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::{ContentCodings, ContentCoding};
    ///
    /// let mut codings = ContentCodings::new();
    /// codings.push(ContentCoding::Gzip);
    /// codings.push(ContentCoding::Brotli);
    /// ```
    pub fn push(&mut self, coding: ContentCoding) {
        self.codings.push(coding);
    }

    /// Checks if identity content encoding is used (no content encoding).
    ///
    /// # Returns
    ///
    /// `true` if no content encodings are present, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::{ContentCodings, ContentCoding};
    ///
    /// let mut codings = ContentCodings::new();
    /// assert!(codings.is_identity());
    ///
    /// codings.push(ContentCoding::Gzip);
    /// assert!(!codings.is_identity());
    /// ```
    pub fn is_identity(&self) -> bool {
        self.codings.is_empty()
    }

    /// Converts the content codings to a header value string.
    ///
    /// # Returns
    ///
    /// A comma-separated string of content codings
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::encoding::{ContentCodings, ContentCoding};
    ///
    /// let mut codings = ContentCodings::new();
    /// codings.push(ContentCoding::Gzip);
    /// codings.push(ContentCoding::Brotli);
    ///
    /// assert_eq!(codings.to_header(), "gzip, br");
    /// ```
    pub fn to_header(&self) -> String {
        self.codings
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Decodes compressed data using the content codings in this collection,
    /// bounded by `max_size`.
    ///
    /// # Arguments
    ///
    /// * `data` - The compressed data to decode
    /// * `max_size` - Maximum decompressed size in bytes
    ///
    /// # Returns
    ///
    /// A `Result` containing the decompressed data as a `Vec<u8>`, or an error
    /// if decoding fails or the output exceeds `max_size`.
    pub fn decode_compressed(
        &self,
        data: Vec<u8>,
        max_size: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        if self.is_identity() {
            return Ok(data);
        }

        let mut result = data;
        // Decompress in REVERSE order (last applied first)
        for coding in self.codings.iter().rev() {
            result = ContentCoding::decode_compressed(coding, &result, max_size)?;
        }
        Ok(result)
    }

    /// Encodes data using the content codings in this collection.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to encode
    ///
    /// # Returns
    ///
    /// A `Result` containing the encoded data as a `Vec<u8>`, or an error if encoding fails.
    ///
    /// # Examples
    /// ```
    /// # use hotaru_http::encoding::ContentCodings;
    /// let codings = ContentCodings::new();
    /// let data = b"hello".to_vec();
    /// let result = codings.encode_compressed(data.clone()).unwrap();
    /// assert_eq!(result, data);
    /// ```
    pub fn encode_compressed(&self, data: Vec<u8>) -> Result<Vec<u8>, CompressionError> {
        if self.is_identity() {
            return Ok(data);
        }

        let mut result = data;
        // Compress in ORDER (first applied first)
        for coding in &self.codings {
            result = ContentCoding::encode_compressed(coding, &result)?;
        }
        Ok(result)
    }
}
