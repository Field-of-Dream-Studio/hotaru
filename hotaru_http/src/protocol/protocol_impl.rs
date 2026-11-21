//! HTTP/1.1 protocol handler — `Http1Protocol` struct and `Protocol` trait impl.
//!
//! This is the main protocol implementation for HTTP/1.1, handling both
//! server and client roles. It is generic over the wire stream type so the
//! same logic works for TCP and TLS transports.

use std::marker::PhantomData;
use std::sync::Arc;

use hotaru_core::{
    app::common::RuntimeConfig,
    connection::{ConnStream, HotaruRead, HotaruWrite, Outbound, TransportSpec},
    protocol::{Channel, CtxError, Protocol, ProtocolFlow, ProtocolRole, RequestContext},
    url::UrlRoot,
};
use hotaru_io_tokio::TcpStream;

use crate::{
    channel::{Http1Channel, HttpChannel},
    context::HttpContext,
    message::{http_value::HttpVersion, response::response_templates, start_line::StartLineError},
    protocol::{error::HttpError, sync::HttpSyncSettings},
    security::safety::HttpSafety,
};

// ============================================================================
// Type Aliases
// ============================================================================

/// Default transport spec used by HTTP when callers don't specify one.
pub type DefaultHttpTransport = hotaru_io_tokio::TcpTransport;

/// Default HTTP protocol (currently HTTP/1.1)
/// This provides a simpler name for user-facing code while maintaining
/// version-specific naming in the implementation.
///
/// In the new transport design this defaults to plain TCP transport.
pub type HTTP = Http1Protocol<TcpStream, DefaultHttpTransport>;

/// HTTP/1.1 over plain TCP transport.
pub type Http1TcpProtocol = Http1Protocol<TcpStream, DefaultHttpTransport>;

/// HTTP/1.1 over TLS transport (enabled by `tls` feature).
#[cfg(feature = "tls")]
pub type Http1TlsProtocol = Http1Protocol<hotaru_tls::TlsStream, hotaru_tls::TlsTransport>;

/// HTTPS alias (HTTP/1.1 over TLS), enabled by `tls` feature.
#[cfg(feature = "tls")]
pub type HTTPS = Http1TlsProtocol;

// ============================================================================
// Http1Protocol
// ============================================================================

/// HTTP/1.1 protocol handler.
///
/// Generic over the concrete wire stream type so the same logic serves TCP
/// and TLS transports. Owns independent safety and synchronization settings
/// shared with its channels through `Arc`.
pub struct Http1Protocol<
    W: ConnStream = TcpStream,
    TS: TransportSpec<Wire = W> = DefaultHttpTransport,
> {
    role: ProtocolRole,
    safety: Arc<HttpSafety>,
    sync_settings: Arc<HttpSyncSettings>,
    _wire: PhantomData<fn() -> W>,
    _ts: PhantomData<fn() -> TS>,
}

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Clone for Http1Protocol<W, TS> {
    fn clone(&self) -> Self {
        Self {
            role: self.role,
            safety: self.safety.clone(),
            sync_settings: self.sync_settings.clone(),
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }
}

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Default for Http1Protocol<W, TS> {
    /// Creates a server protocol with default safety and synchronization settings.
    #[av::ver(
        unstable,
        since = "0.8.6",
        note = "Default HTTP server safety and synchronization settings",
        date = "2026-11-14"
    )]
    fn default() -> Self {
        Self::server(HttpSafety::default(), HttpSyncSettings::default())
    }
}

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Http1Protocol<W, TS> {
    /// Server role with independent safety and synchronization settings.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Accept explicit server safety and synchronization settings",
        date = "2026-11-14"
    )]
    pub fn server(safety: HttpSafety, sync_settings: HttpSyncSettings) -> Self {
        Self {
            role: ProtocolRole::Server,
            safety: Arc::new(safety),
            sync_settings: Arc::new(sync_settings),
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }

    /// Client role with independent safety and synchronization settings.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Accept explicit client safety and synchronization settings",
        date = "2026-11-14"
    )]
    pub fn client(safety: HttpSafety, sync_settings: HttpSyncSettings) -> Self {
        Self {
            role: ProtocolRole::Client,
            safety: Arc::new(safety),
            sync_settings: Arc::new(sync_settings),
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }

    /// Borrows this protocol's safety baseline.
    pub fn safety(&self) -> &HttpSafety {
        &self.safety
    }

    /// Borrows this protocol's outgoing synchronization settings.
    #[av::ver(
        unstable,
        since = "0.8.6",
        note = "Inspect protocol-owned outgoing synchronization settings",
        date = "2026-11-14"
    )]
    pub fn sync_settings(&self) -> &HttpSyncSettings {
        &self.sync_settings
    }
}

/// Require a version whose wire format is implemented by `Http1Protocol`.
///
/// `HttpVersion` also represents versions used by other protocol
/// implementations. Recognising one of those versions while parsing must not
/// allow it to enter the HTTP/1 router.
fn ensure_http1_version(version: &HttpVersion) -> Result<(), StartLineError> {
    match version {
        HttpVersion::Http10 | HttpVersion::Http11 => Ok(()),
        _ => Err(StartLineError::UnsupportedHttpVersion),
    }
}

// ============================================================================
// Protocol trait implementation
// ============================================================================

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Protocol for Http1Protocol<W, TS>
where
    HttpError: From<<TS as TransportSpec>::IoError>,
    W::ReadHalf: HotaruRead<Error = std::io::Error>,
    W::WriteHalf: HotaruWrite<Error = std::io::Error>,
{
    type Wire = W;
    type TS = TS;
    type Channel = Http1Channel<W>;
    type Stream = ();
    type Message = ();
    type Context = HttpContext<TS>;

    fn name(&self) -> &'static str {
        "http"
    }

    fn role(&self) -> ProtocolRole {
        self.role
    }

    fn lit_parser<'a>(input: &'a str) -> Vec<&'a str> {
        // Mirrors UrlRoot::walk_str: empty input goes to the root
        // endpoint slot (returned via an empty segment vec), everything
        // else splits on '/' preserving empties so "/foo" walks as
        // ["", "foo"] and matches the leading Literal("") that the
        // pattern parser emits for a leading '/'.
        if input.is_empty() {
            Vec::new()
        } else {
            input.split('/').collect()
        }
    }

    fn detect(initial_bytes: &[u8]) -> bool {
        initial_bytes.starts_with(b"GET ")
            || initial_bytes.starts_with(b"POST ")
            || initial_bytes.starts_with(b"PUT ")
            || initial_bytes.starts_with(b"DELETE ")
            || initial_bytes.starts_with(b"HEAD ")
            || initial_bytes.starts_with(b"OPTIONS ")
            || initial_bytes.starts_with(b"PATCH ")
            || initial_bytes.starts_with(b"CONNECT ")
            || initial_bytes.starts_with(b"TRACE ")
    }

    fn open_channel(
        self,
        reader: <<<Self::TS as TransportSpec>::Wire as ConnStream>::ReadHalf as HotaruRead>::Buffered,
        writer: <<<Self::TS as TransportSpec>::Wire as ConnStream>::WriteHalf as HotaruWrite>::Buffered,
        meta: <<Self::TS as TransportSpec>::Wire as ConnStream>::Meta,
    ) -> Self::Channel {
        Http1Channel::new(reader, writer, meta, self.safety, self.sync_settings)
    }

    async fn handle(
        channel: &Self::Channel,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<Self::Context, Self::TS>>,
    ) -> Result<ProtocolFlow, <Self::Context as RequestContext>::Error> {
        // 1. Parse one request using the channel-stored safety baseline
        //    (no per-request HashMap lookup against RuntimeConfig).
        let request = match channel.parse_request(channel.safety()).await {
            Ok(request) => request,
            Err(error) if matches!(&error, HttpError::Meta(_) | HttpError::Body(_)) => {
                let response = response_templates::error_response(&error);
                let keep_alive = response.is_keep_alive();
                channel.send_response(response).await?;
                return Ok(if keep_alive {
                    ProtocolFlow::Continue
                } else {
                    ProtocolFlow::Close
                });
            }
            Err(error) => return Err(error),
        };

        if let Err(error) = ensure_http1_version(request.meta.start_line.http_version()) {
            let error: HttpError = error.into();
            let response = response_templates::error_response(&error);
            let keep_alive = request.is_keep_alive() && response.is_keep_alive();
            channel.send_response(response).await?;
            return Ok(if keep_alive {
                ProtocolFlow::Continue
            } else {
                ProtocolFlow::Close
            });
        }

        let request_keep_alive = request.is_keep_alive();

        // 2. Walk URL tree.
        let path = request.meta.path();
        let endpoint = match root.walk_str(&path).await {
            Some(node) => node,
            None => {
                let error = HttpError::NoRoute(path);
                let response = response_templates::error_response(&error);
                let keep_alive = request_keep_alive && response.is_keep_alive();
                channel.send_response(response).await?;
                return Ok(if keep_alive {
                    ProtocolFlow::Continue
                } else {
                    ProtocolFlow::Close
                });
            }
        };

        // 3. Build context, run chain. Addresses come from the channel's meta.
        //    Seed ctx.safety from the protocol baseline so endpoint overrides
        //    overlay on top of it instead of falling back to defaults.
        let mut ctx = HttpContext::new_server(
            runtime.clone(),
            request,
            channel.remote_addr(),
            channel.local_addr(),
            channel.safety().clone(),
        );
        ctx.bind_endpoint(endpoint.clone())?;
        ctx.install_channel(channel.clone());

        match endpoint.run(ctx).await {
            Ok(ctx) => {
                let response = ctx.response;
                let keep_alive = request_keep_alive && response.is_keep_alive();
                channel.send_response(response).await?;
                Ok(if keep_alive {
                    ProtocolFlow::Continue
                } else {
                    ProtocolFlow::Close
                })
            }
            Err(error) => {
                let response = response_templates::error_response(&error);
                let keep_alive = request_keep_alive && response.is_keep_alive();
                channel.send_response(response).await?;
                Ok(if keep_alive {
                    ProtocolFlow::Continue
                } else {
                    ProtocolFlow::Close
                })
            }
        }
    }

    async fn acquire_channel(
        &self,
        _runtime: &Arc<RuntimeConfig>,
        outbound: Arc<<Self::TS as TransportSpec>::Outbound>,
    ) -> Result<Self::Channel, CtxError<Self>> {
        // Dial fresh per request. Future pooling slots in behind a
        // `self.pool` field without changing this signature.
        let wire = outbound.connect().await?;
        let (read, write, meta) = wire.split();
        let reader = read.into_buf();
        let writer = write.into_buf_write();
        Ok(Http1Channel::new(
            reader,
            writer,
            meta,
            self.safety.clone(),
            self.sync_settings.clone(),
        ))
    }

    async fn send(
        mut ctx: Self::Context,
    ) -> Result<Self::Context, <Self::Context as RequestContext>::Error> {
        let channel = ctx.channel().cloned().ok_or_else(|| {
            HttpError::Io(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "outpoint channel is not installed",
            ))
        })?;

        let safety = ctx.safety.clone();

        let request = ctx.take_request();
        channel.send_request(request).await?;

        let response = channel.parse_response(&safety).await?;
        let keep_alive = response.is_keep_alive();
        ctx.set_response(response);

        if !keep_alive {
            channel.close();
        }
        Ok(ctx)
    }

    fn install_channel(ctx: &mut Self::Context, channel: Self::Channel) {
        ctx.install_channel(channel);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::body::HttpBody;
    use crate::message::http_value::StatusCode;
    use crate::message::request::HttpRequest;
    use crate::message::response::HttpResponse;

    #[tokio::test]
    async fn protocol_sync_settings_reach_server_and_client_channels() {
        let server = HTTP::default();
        assert_eq!(server.role(), ProtocolRole::Server);
        assert_eq!(server.sync_settings(), &HttpSyncSettings::default());

        let settings = HttpSyncSettings {
            sync_content_type: false,
            ..HttpSyncSettings::default()
        };
        let server = HTTP::server(HttpSafety::default(), settings);
        let client = HTTP::client(HttpSafety::default(), settings);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let outbound = Arc::new(
            hotaru_io_tokio::TcpOutbound::build(listener.local_addr().unwrap().to_string())
                .await
                .unwrap(),
        );
        let runtime = Arc::new(RuntimeConfig::default());
        let client_clone = client.clone();
        let (client_channel, accepted) = tokio::join!(
            client_clone.acquire_channel(&runtime, outbound),
            listener.accept(),
        );
        let client_channel = client_channel.unwrap();
        let (read, write, meta) = TcpStream::new(accepted.unwrap().0).split();
        let server_channel =
            server
                .clone()
                .open_channel(read.into_buf(), write.into_buf_write(), meta);

        let mut request = HttpRequest::default();
        request.meta.set_host(Some("localhost".to_string()));
        request.body = HttpBody::Binary(b"ping".to_vec());
        client_channel.clone().send_request(request).await.unwrap();
        let mut received = server_channel.parse_request(server.safety()).await.unwrap();
        assert_eq!(received.meta.get_content_type(), None);
        assert_eq!(received.meta.get_content_length().unwrap(), Some(4));
        assert!(matches!(received.body, HttpBody::Buffer { data, .. } if data == b"ping"));

        let mut response = HttpResponse::default();
        response.body = HttpBody::Binary(b"pong".to_vec());
        server_channel
            .clone()
            .send_response(response)
            .await
            .unwrap();
        let mut received = client_channel
            .parse_response(client.safety())
            .await
            .unwrap();
        assert_eq!(received.meta.get_content_type(), None);
        assert_eq!(received.meta.get_content_length().unwrap(), Some(4));
        assert!(matches!(received.body, HttpBody::Buffer { data, .. } if data == b"pong"));
    }

    #[test]
    fn test_http1_detection() {
        assert!(HTTP::detect(b"GET / HTTP/1.1\r\n"));
        assert!(HTTP::detect(b"POST /api HTTP/1.1\r\n"));
        assert!(HTTP::detect(b"PUT /resource HTTP/1.1\r\n"));
        assert!(!HTTP::detect(b"INVALID REQUEST\r\n"));
        assert!(!HTTP::detect(b""));
    }

    #[test]
    fn http_10_and_11_are_supported() {
        assert!(ensure_http1_version(&HttpVersion::Http10).is_ok());
        assert!(ensure_http1_version(&HttpVersion::Http11).is_ok());
    }

    #[test]
    fn non_http1_versions_are_rejected() {
        for version in [
            HttpVersion::Http09,
            HttpVersion::Http20,
            HttpVersion::Http30,
            HttpVersion::Unknown,
        ] {
            assert!(matches!(
                ensure_http1_version(&version),
                Err(StartLineError::UnsupportedHttpVersion)
            ));
        }
    }

    #[test]
    fn pattern_and_literal_sides_align() {
        use hotaru_core::url::tokens_to_patterns;

        let tokens = HTTP::tokenize_url("/users/<int:id>").unwrap();
        let (patterns, _names) = tokens_to_patterns(&tokens).unwrap();
        let segments = HTTP::lit_parser("/users/42");

        assert_eq!(
            patterns.len(),
            segments.len(),
            "leading-slash arity mismatch"
        );
        for (pat, seg) in patterns.iter().zip(segments.iter()) {
            assert!(
                pat.matches(seg),
                "pattern {:?} did not match segment {:?}",
                pat,
                seg
            );
        }
    }

    #[test]
    fn root_slash_aligns() {
        use hotaru_core::url::tokens_to_patterns;

        let tokens = HTTP::tokenize_url("/").unwrap();
        let (patterns, _) = tokens_to_patterns(&tokens).unwrap();
        let segments = HTTP::lit_parser("/");

        assert_eq!(patterns.len(), segments.len());
        for (pat, seg) in patterns.iter().zip(segments.iter()) {
            assert!(pat.matches(seg));
        }
    }
}
