//! Typed `Date` header value (RFC 7231 §7.1.1.1).
//!
//! Only the storage shape is defined here; parsing, formatting, and clock
//! stamping are wired in later steps.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpDate(String);
