use std::fmt::Write;

use hotaru_core::connection::{HotaruBufRead, HotaruWrite};

use crate::message::body::HttpBody;
use crate::message::http_value::HttpContentType;
use crate::message::meta::HttpMeta;
use crate::protocol::HttpError;
use crate::security::safety::HttpSafety;

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

pub async fn send<W: HotaruWrite<Error = std::io::Error> + Unpin + Send>(
    mut meta: HttpMeta,
    body: HttpBody,
    writer: &mut W,
) -> Result<(), HttpError> {
    let mut headers = String::with_capacity(256);

    let encoding = meta.get_encoding()?;
    let content_type = meta.get_content_type();
    let boundary = match &content_type {
        Some(HttpContentType::Multipart { boundary, .. }) => boundary.as_deref(),
        _ => None,
    };
    let (bin, suggested_content_type) = body.encode(&encoding, boundary).await?;
    if meta.get_content_length()?.is_none() {
        meta.set_content_length(bin.len());
    }
    if content_type.is_none()
        && let Some(content_type) = suggested_content_type
    {
        meta.set_content_type(content_type);
    }
    write!(&mut headers, "{}", meta.represent()).map_err(std::io::Error::other)?;

    writer.write_all(headers.as_bytes()).await?;
    writer.write_all(&bin).await?;
    writer.flush().await?;

    Ok(())
}
