pub mod traits;
pub mod error;
pub mod protocol_impl;
pub mod safety;
pub mod sync;
#[cfg(test)]
mod test;

pub use error::HttpError;
pub use safety::HttpSafety;
pub use sync::HttpSyncSettings;
pub use traits::{DefaultHttpTransport, HTTP, Http1Protocol, Http1TcpProtocol};
#[cfg(feature = "tls")]
pub use traits::{HTTPS, Http1TlsProtocol};
