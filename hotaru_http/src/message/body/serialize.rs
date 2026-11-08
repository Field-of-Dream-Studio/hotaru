use akari::Value;

use crate::message::http_value::HttpContentType;
use crate::protocol::HttpError;
use crate::util::encoding::HttpEncoding;
use crate::util::form::{MultiForm, UrlEncodedForm};

use super::HttpBody;

impl HttpBody {
    /// Serializes and content-encodes the body, returning bytes and a suggested
    /// Content-Type. The sender handles metadata updates and transfer framing.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Renamed from into_static",
        date = "2026-11-08"
    )]
    pub async fn encode(
        self,
        encoding: &HttpEncoding,
        multipart_boundary: Option<&str>,
    ) -> Result<(Vec<u8>, Option<HttpContentType>), HttpError> {
        fn serialize_from_json(json: Value) -> Vec<u8> {
            json.into_json().into_bytes()
        }

        fn serialize_from_text(text: String) -> Vec<u8> {
            text.into_bytes()
        }

        fn serialize_from_binary(binary: Vec<u8>) -> Vec<u8> {
            binary
        }

        fn serialize_from_form(form: UrlEncodedForm) -> Vec<u8> {
            form.to_string().into_bytes()
        }

        fn serialize_from_files(files: MultiForm, boundary: &str) -> Vec<u8> {
            files.to_string(&boundary.to_string()).into_bytes()
        }

        let (bin, default_content_type) = match self {
            Self::Text(text) => (serialize_from_text(text), Some(HttpContentType::TextHtml())),
            Self::Binary(binary) => (
                serialize_from_binary(binary),
                Some(HttpContentType::ApplicationOctetStream()),
            ),
            Self::Json(json) => (
                serialize_from_json(json),
                Some(HttpContentType::ApplicationJson()),
            ),
            Self::Form(form) => (
                serialize_from_form(form),
                Some(HttpContentType::ApplicationUrlEncodedForm()),
            ),
            Self::Files(files) => {
                let boundary = multipart_boundary
                    .unwrap_or("----DefaultBoundary7MA4YWxkTrZu0gW")
                    .to_string();
                let bin = serialize_from_files(files, &boundary);
                let ct = HttpContentType::Multipart {
                    subtype: "form-data".to_string(),
                    boundary: Some(boundary),
                };
                (bin, Some(ct))
            }
            _ => (Vec::new(), None),
        };

        let compressed = encoding.content().clone().encode_compressed(bin)?;
        Ok((compressed, default_content_type))
    }

    /// Compatibility alias for [`HttpBody::encode`].
    #[av::ver(
        deprecated,
        since = "0.8.6",
        note = "Use HttpBody::encode instead",
        date = "2026-11-08"
    )]
    pub async fn into_static(
        self,
        encoding: &HttpEncoding,
        multipart_boundary: Option<&str>,
    ) -> Result<(Vec<u8>, Option<HttpContentType>), HttpError> {
        self.encode(encoding, multipart_boundary).await
    }

    /// Returns the raw data from a binary HTTP body.
    ///
    /// A non-binary body must first be converted into binary form.
    pub fn raw(self) -> Vec<u8> {
        match self {
            Self::Binary(data) => data,
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::encoding::HttpEncoding;

    use super::*;

    #[tokio::test]
    async fn serializes_text_and_suggests_content_type() {
        let (body, content_type) = HttpBody::Text("hello".to_string())
            .encode(&HttpEncoding::default(), None)
            .await
            .unwrap();

        assert_eq!(body, b"hello");
        assert!(matches!(
            content_type,
            Some(HttpContentType::Text { subtype, .. }) if subtype == "html"
        ));
    }

    #[tokio::test]
    async fn returns_content_coding_failure() {
        let encoding = HttpEncoding::from_headers(None, Some("compress".to_string())).unwrap();

        let result = HttpBody::Text("hello".to_string())
            .encode(&encoding, None)
            .await;

        use crate::message::body::BodyError;
        use crate::util::encoding::CompressionError;
        assert!(matches!(
            result,
            Err(HttpError::Body(BodyError::Compression(
                CompressionError::Unavailable("compress")
            )))
        ));
    }
}
