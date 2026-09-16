pub mod traits;
pub mod error;
pub mod protocol_impl;
#[cfg(test)]
mod test;

pub use error::HttpError;
pub use traits::{DefaultHttpTransport, HTTP, Http1Protocol, Http1TcpProtocol};
#[cfg(feature = "tls")]
pub use traits::{HTTPS, Http1TlsProtocol};
