use core::str::FromStr;

use crate::message::header::HeaderMap;
use crate::message::http_value::HttpVersion;

use super::{ConnectionError, ConnectionToken};

/// Ordered connection-options parsed from every `Connection` field value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectionOptions {
    tokens: Vec<ConnectionToken>,
}

impl ConnectionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses all `Connection` field values, including comma-separated lists.
    pub fn from_headers(headers: &HeaderMap) -> Result<Self, ConnectionError> {
        let mut options = Self::new();
        for value in headers.get_all("connection") {
            options.extend_value(value)?;
        }
        Ok(options)
    }

    /// Returns the parsed options in wire order.
    pub fn tokens(&self) -> &[ConnectionToken] {
        &self.tokens
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn contains_close(&self) -> bool {
        self.tokens.contains(&ConnectionToken::Close)
    }

    pub fn contains_keep_alive(&self) -> bool {
        self.tokens.contains(&ConnectionToken::KeepAlive)
    }

    /// Applies HTTP/1 connection persistence rules to these options.
    ///
    /// `close` always wins. HTTP/1.1 is persistent by default, while HTTP/1.0
    /// requires an explicit `keep-alive` option. Other protocol versions do
    /// not use HTTP/1 `Connection` semantics and therefore return `false`.
    pub fn is_keep_alive(&self, version: &HttpVersion) -> bool {
        if self.contains_close() {
            return false;
        }

        match version {
            HttpVersion::Http11 => true,
            HttpVersion::Http10 => self.contains_keep_alive(),
            _ => false,
        }
    }

    /// Produces a normalized field value, or `None` when no options are present.
    pub fn to_header(&self) -> Option<String> {
        (!self.is_empty()).then(|| {
            self.tokens
                .iter()
                .map(ConnectionToken::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
    }

    fn extend_value(&mut self, value: &str) -> Result<(), ConnectionError> {
        // HTTP list syntax permits recipients to ignore empty list elements.
        for part in value
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            self.tokens.push(ConnectionToken::from_str(part)?);
        }
        Ok(())
    }
}
