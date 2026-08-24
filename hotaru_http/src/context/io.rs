use std::fmt::Write;

use hotaru_core::connection::error::ConnectionError;
use hotaru_core::connection::{HotaruBufRead, HotaruWrite};

use crate::message::body::HttpBody;
use crate::message::meta::HttpMeta;
use crate::protocol::HttpError;
use crate::security::safety::HttpSafety;

pub async fn parse_lazy<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
    stream: &mut R,
    config: &HttpSafety,
    is_request: bool,
    print_raw: bool,
) -> Result<(HttpMeta, HttpBody), ConnectionError> {
    // Create one BufReader up-front, pass this throughout.
    let mut meta = HttpMeta::from_stream(stream, config, print_raw, is_request).await?;

    let body = HttpBody::read_buffer(stream, &mut meta, config).await?;

    Ok((meta, body))
}

pub async fn send<W: HotaruWrite<Error = std::io::Error> + Unpin + Send>(
    mut meta: HttpMeta,
    body: HttpBody,
    writer: &mut W,
) -> Result<(), HttpError> {
    let mut headers = String::with_capacity(256);

    let bin = body.into_static(&mut meta).await?;
    write!(&mut headers, "{}", meta.represent()).map_err(std::io::Error::other)?;

    writer.write_all(headers.as_bytes()).await?;
    writer.write_all(&bin).await?;
    writer.flush().await?;

    Ok(())
}
