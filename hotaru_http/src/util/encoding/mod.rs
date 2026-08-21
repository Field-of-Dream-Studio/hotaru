//! # HTTP Encoding
//!
//! This module provides types and functionality for working with HTTP encoding mechanisms,
//! specifically Transfer-Encoding and Content-Encoding as defined in HTTP standards.
//!
//! ## Overview
//!
//! HTTP allows for various encoding mechanisms:
//!
//! - **Transfer-Encoding**: Specifies the form in which the message body is transferred
//!   between HTTP nodes. The most common is "chunked" encoding.
//!
//! - **Content-Encoding**: Specifies how the content is compressed, such as gzip,
//!   deflate, or brotli.
//!
//! This module provides strongly-typed representations of these encodings with proper
//! validation according to HTTP standards.
//!
//! ## Examples
//!
//! ```
//! # use hotaru_http::encoding::HttpEncoding;
//!
//! // Parse from headers
//! let encoding = HttpEncoding::from_headers(
//!     Some("chunked".to_string()),
//!     Some("br".to_string())
//! );
//!
//! // Check if chunked encoding is used
//! assert!(encoding.transfer().is_chunked());
//!
//! // Serialize back to headers
//! let (transfer, content) = encoding.to_headers();
//! assert_eq!(transfer, Some("chunked".to_string()));
//! assert_eq!(content, Some("br".to_string()));
//! ```

mod content;
mod encoding;
mod error;
mod transfer;

pub use content::{ContentCoding, ContentCodings};
pub use encoding::HttpEncoding;
pub use error::{CompressionFailure, EncodingError};
pub use transfer::{TransferCoding, TransferCodings};
