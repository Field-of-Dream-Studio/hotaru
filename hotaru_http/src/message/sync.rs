//! Shared outgoing preparation for requests and responses.

use crate::protocol::HttpError;
use crate::protocol::HttpSyncSettings;
use crate::util::date::HttpDate;
use crate::util::encoding::HttpEncoding;

use super::body::HttpBody;
use super::meta::HttpMeta;

pub(crate) fn config_sync(
    mut meta: HttpMeta,
    body: HttpBody,
    settings: &HttpSyncSettings,
) -> Result<(HttpMeta, HttpBody), HttpError> {
    let encoding = meta.get_encoding()?;
    let body = body.encode(encoding.content().clone())?;
    let HttpBody::Buffer {
        data,
        content_type,
        content_coding,
    } = &body
    else {
        unreachable!("encode always returns a buffer on success");
    };

    if settings.sync_content_type {
        meta.sync_content_type(Some(content_type.clone()));
    }
    if settings.sync_length {
        meta.delete_content_length();
        if encoding.transfer().is_identity() {
            meta.set_content_length(data.len());
        }
    }
    if settings.sync_encoding {
        let (transfer_header, _) = encoding.to_headers();
        let content_header = (!content_coding.is_identity()).then(|| content_coding.to_header());
        let encoding = HttpEncoding::from_headers(transfer_header, content_header)?;
        meta.delete_encoding();
        meta.set_encoding(Some(encoding));
    }

    // get_date() treats missing and unparseable Date headers alike.
    if settings.sync_date && meta.get_date().is_none() {
        meta.set_date(HttpDate::now());
    }

    Ok((meta, body))
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "compression")]
    use hotaru_io_tokio::TokioIo;

    use crate::message::http_value::{HttpVersion, StatusCode};
    use crate::util::content_type::HttpContentType;
    use crate::message::request::HttpRequest;
    use crate::message::response::HttpResponse;
    use crate::message::start_line::HttpStartLine;
    use crate::util::form::MultiForm;

    use super::*;

    fn sync_both(meta: HttpMeta, body: HttpBody) -> [(HttpMeta, HttpBody); 2] {
        let request = HttpRequest::new(meta.clone(), body.clone()).sync().unwrap();
        let mut response_meta = meta;
        response_meta.start_line = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
        let response = HttpResponse::new(response_meta, body).sync().unwrap();
        [(request.meta, request.body), (response.meta, response.body)]
    }

    #[test]
    fn configurable_sync_preserves_content_type_and_absence_when_disabled() {
        let settings = HttpSyncSettings {
            sync_content_type: false,
            ..HttpSyncSettings::default()
        };
        for original_type in [None, Some(HttpContentType::from_str("image/png"))] {
            let mut meta = HttpMeta::default();
            if let Some(content_type) = &original_type {
                meta.set_attribute("content-type", content_type.to_string());
            }
            assert_eq!(meta.get_content_type(), original_type);
            let body = HttpBody::Binary(vec![0, 0xff]);
            let (mut meta, body) = config_sync(meta, body, &settings).unwrap();
            assert_eq!(meta.get_content_type(), original_type);
            assert_eq!(
                meta.get_header("content-type").is_some(),
                original_type.is_some()
            );
            assert_eq!(meta.get_content_length().unwrap(), Some(2));
            assert_eq!(
                body.get_content_type(),
                Some(HttpContentType::ApplicationOctetStream())
            );
        }
    }

    #[test]
    fn sync_reconciles_media_types_and_replaces_stale_lengths() {
        let mut meta = HttpMeta::default();
        meta.set_attribute("content-type", "image/png");
        meta.set_attribute("content-length", "999");
        meta.set_content_length(999);
        for (mut meta, body) in sync_both(meta, HttpBody::Binary(vec![0, 0xff])) {
            assert_eq!(
                meta.get_content_type(),
                Some(HttpContentType::from_str("image/png"))
            );
            assert_eq!(meta.get_content_length().unwrap(), Some(2));
            let HttpBody::Buffer {
                data, content_type, ..
            } = body
            else {
                panic!("expected a prepared body buffer");
            };
            assert_eq!(data, [0, 0xff]);
            assert_eq!(content_type, HttpContentType::ApplicationOctetStream());
            let headers = meta.represent();
            assert_eq!(headers.matches("content-length:").count(), 1);
            assert_eq!(headers.matches("content-type:").count(), 1);
            assert!(!headers.contains("999"));
        }
    }

    #[test]
    fn sync_uses_the_body_multipart_boundary() {
        for stale_type in [
            HttpContentType::Multipart {
                subtype: "form-data".to_string(),
                boundary: None,
            },
            HttpContentType::Multipart {
                subtype: "form-data".to_string(),
                boundary: Some("different-boundary".to_string()),
            },
            HttpContentType::ApplicationJson(),
        ] {
            let mut meta = HttpMeta::default();
            meta.set_content_type(stale_type);
            let body = HttpBody::Files(MultiForm::try_new("stored-boundary").unwrap());
            for (mut meta, body) in sync_both(meta, body) {
                assert_eq!(
                    meta.get_content_type(),
                    Some(HttpContentType::Multipart {
                        subtype: "form-data".to_string(),
                        boundary: Some("stored-boundary".to_string()),
                    })
                );
                assert_eq!(meta.get_content_length().unwrap(), Some(21));
                let HttpBody::Buffer { data, .. } = body else {
                    panic!("expected a prepared body buffer");
                };
                assert_eq!(data, b"--stored-boundary--\r\n");
            }
        }
    }

    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn sync_preserves_gzip_bytes_and_uses_the_encoded_length() {
        use crate::protocol::safety::HttpSafety;

        let text = "hello from a synchronized body";
        let mut meta = HttpMeta::default();
        meta.set_attribute("content-encoding", "gzip");
        meta.set_attribute("content-length", "999");
        let request = HttpRequest::new(meta, HttpBody::Text(text.to_string()))
            .sync()
            .unwrap();
        let HttpBody::Buffer { data, .. } = &request.body else {
            panic!("expected a prepared body buffer");
        };
        let encoded = data.clone();
        assert!(encoded.starts_with(&[0x1f, 0x8b]));
        let mut request = request.sync().unwrap();
        assert_eq!(
            request.meta.get_content_length().unwrap(),
            Some(encoded.len() as u64)
        );
        let mut writer = TokioIo::new(Vec::new());
        request
            .send(&mut writer, &HttpSyncSettings::default())
            .await
            .unwrap();
        let wire = writer.into_inner();
        let split = wire
            .windows(4)
            .position(|part| part == b"\r\n\r\n")
            .unwrap();
        assert_eq!(&wire[split + 4..], encoded);

        let body = HttpBody::Text(text.to_string())
            .encode(
                HttpEncoding::from_headers(None, Some("gzip".to_string()))
                    .unwrap()
                    .content()
                    .clone(),
            )
            .unwrap();
        let mut response = HttpResponse::new(HttpMeta::default(), body).sync().unwrap();
        assert_eq!(
            response.meta.get_encoding().unwrap().content().to_header(),
            "gzip"
        );
        let HttpBody::Text(decoded) = response.body.decode(&HttpSafety::default()).unwrap() else {
            panic!("expected decoded text");
        };
        assert_eq!(decoded, text);
    }
}
