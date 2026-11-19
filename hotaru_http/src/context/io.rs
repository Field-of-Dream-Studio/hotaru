use hotaru_core::connection::{HotaruBufRead, HotaruWrite};

use crate::message::body::HttpBody;
use crate::message::meta::HttpMeta;
use crate::message::sync::config_sync;
use crate::protocol::safety::HttpSafety;
use crate::protocol::{HttpError, HttpSyncSettings};

/// Reads metadata and the complete framed body, deferring content decoding.
#[av::ver(
    update,
    since = "0.8.6",
    note = "Renamed from parse_lazy",
    date = "2026-11-08"
)]
pub async fn receive<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
    stream: &mut R,
    config: &HttpSafety,
    is_request: bool,
    print_raw: bool,
) -> Result<(HttpMeta, HttpBody), HttpError> {
    // Create one BufReader up-front, pass this throughout.
    let mut meta = HttpMeta::from_stream(stream, config, print_raw, is_request).await?;

    let body = HttpBody::read_buffer(stream, &mut meta, config).await?;

    Ok((meta, body))
}

/// Compatibility alias for [`receive`].
#[av::ver(
    deprecated,
    since = "0.8.6",
    note = "Use receive instead",
    date = "2026-11-08"
)]
pub async fn parse_lazy<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
    stream: &mut R,
    config: &HttpSafety,
    is_request: bool,
    print_raw: bool,
) -> Result<(HttpMeta, HttpBody), HttpError> {
    receive(stream, config, is_request, print_raw).await
}

/// Sends an HTTP message to a stream.
///
/// Synchronizes metadata and prepares the body before writing headers, then
/// writes the prepared body with transfer framing and flushes the writer.
#[av::ver(
    update,
    since = "0.8.6",
    note = "Send messages through shared synchronization and body writing",
    date = "2026-11-13"
)]
pub async fn send<W: HotaruWrite<Error = std::io::Error> + Unpin + Send>(
    meta: HttpMeta,
    body: HttpBody,
    writer: &mut W,
    settings: &HttpSyncSettings,
) -> Result<(), HttpError> {
    let (mut meta, body) = config_sync(meta, body, settings)?;
    let encoding = meta.get_encoding()?;
    let headers = meta.represent();

    writer.write_all(headers.as_bytes()).await?;
    body.write_buffer(writer, encoding.transfer()).await?;
    writer.flush().await?;

    Ok(())
}
