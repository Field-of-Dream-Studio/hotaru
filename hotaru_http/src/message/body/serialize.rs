use akari::Value;

use crate::message::http_value::HttpContentType;
use crate::message::meta::HttpMeta;
use crate::util::form::{MultiForm, UrlEncodedForm};

use super::{BodyError, HttpBody};

impl HttpBody {
    /// Serializes an HTTP body and updates derived body metadata.
    pub async fn into_static(self, meta: &mut HttpMeta) -> Result<Vec<u8>, BodyError> {
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

        fn serialize_from_files(files: MultiForm, boundary: &String) -> Vec<u8> {
            files.to_string(boundary).into_bytes()
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
                let boundary = if let Some(HttpContentType::Multipart {
                    subtype: _,
                    boundary: Some(boundary),
                }) = meta.get_content_type()
                {
                    boundary
                } else {
                    "----DefaultBoundary7MA4YWxkTrZu0gW".to_string()
                };

                (
                    serialize_from_files(files, &boundary),
                    Some(HttpContentType::Multipart {
                        subtype: "form-data".to_string(),
                        boundary: Some(boundary),
                    }),
                )
            }
            _ => (Vec::new(), None),
        };

        if meta
            .get_content_length()
            .map_err(|error| {
                BodyError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
            })?
            .is_none()
        {
            meta.set_content_length(bin.len());
        }
        if meta.get_content_type().is_none()
            && let Some(content_type) = default_content_type
        {
            meta.set_content_type(content_type);
        }

        let content_coding = meta
            .get_encoding()
            .map(|encoding| encoding.content().clone())
            .unwrap_or_default();
        content_coding
            .encode_compressed(bin)
            .map_err(|_| BodyError::InvalidEncoding)
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
    async fn serializes_text_and_sets_body_metadata() {
        let mut meta = HttpMeta::default();

        let body = HttpBody::Text("hello".to_string())
            .into_static(&mut meta)
            .await
            .unwrap();

        assert_eq!(body, b"hello");
        assert_eq!(meta.get_content_length().unwrap(), Some(5));
        assert!(matches!(
            meta.get_content_type(),
            Some(HttpContentType::Text { subtype, .. }) if subtype == "html"
        ));
    }

    #[tokio::test]
    async fn returns_content_coding_failure() {
        let mut meta = HttpMeta::default();
        meta.set_encoding(Some(HttpEncoding::from_headers(
            None,
            Some("compress".to_string()),
        )));

        let result = HttpBody::Text("hello".to_string())
            .into_static(&mut meta)
            .await;

        assert!(matches!(result, Err(BodyError::InvalidEncoding)));
    }
}
