//! Typed parsing for the HTTP `Connection` header.

mod error;
mod options;
mod token;
#[cfg(test)]
mod test;

pub use error::ConnectionError;
pub use options::ConnectionOptions;
pub use token::ConnectionToken;
