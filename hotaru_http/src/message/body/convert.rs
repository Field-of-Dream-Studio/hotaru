//! Representation conversions without content coding, transfer framing, or stream I/O.

use akari::Value;

use crate::util::content_type::HttpContentType;
use crate::util::form::{MultiForm, MultipartError, UrlEncodedError, UrlEncodedForm};

use super::{BodyError, HttpBody};

impl HttpBody {
    /// Serializes the body into bytes and a suggested Content-Type.
    /// Content encoding, transfer framing, and metadata updates are handled separately.
    #[av::ver(
        unstable,
        since = "0.8.6",
        note = "Body serialization without content encoding",
        date = "2026-11-08"
    )]
    pub fn into_bin(self) -> (Vec<u8>, Option<HttpContentType>) {
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

        fn serialize_from_files(files: MultiForm) -> Vec<u8> {
            files.into_bytes()
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
                let boundary = files.boundary().to_string();
                let bin = serialize_from_files(files);
                let ct = HttpContentType::Multipart {
                    subtype: "form-data".to_string(),
                    boundary: Some(boundary),
                };
                (bin, Some(ct))
            }
            _ => (Vec::new(), None),
        };

        (bin, default_content_type)
    }
}

pub(super) fn parse_into_json(body: Vec<u8>) -> Result<HttpBody, BodyError> {
    let body = String::from_utf8(body).map_err(|_| BodyError::InvalidUtf8)?;
    let value = Value::from_json(&body)
        .map_err(|_| BodyError::InvalidJson("invalid JSON syntax".to_string()))?;
    Ok(HttpBody::Json(value))
}

pub(super) fn parse_into_text(body: Vec<u8>) -> Result<HttpBody, BodyError> {
    String::from_utf8(body)
        .map(HttpBody::Text)
        .map_err(|_| BodyError::InvalidUtf8)
}

pub(super) fn parse_into_binary(body: Vec<u8>) -> HttpBody {
    HttpBody::Binary(body)
}

pub(super) fn parse_into_form(body: Vec<u8>) -> Result<HttpBody, BodyError> {
    UrlEncodedForm::parse(body)
        .map(HttpBody::Form)
        .map_err(|err| match err {
            // Fold UTF-8 into the shared BodyError::InvalidUtf8, same as JSON.
            UrlEncodedError::InvalidUtf8 => BodyError::InvalidUtf8,
            other => BodyError::Form(other),
        })
}

pub(super) fn parse_into_files(body: Vec<u8>, boundary: String) -> Result<HttpBody, BodyError> {
    MultiForm::parse(body, boundary)
        .map(HttpBody::Files)
        .map_err(|err| match err {
            MultipartError::InvalidUtf8 => BodyError::InvalidUtf8,
            other => BodyError::Multipart(other),
        })
}

#[cfg(test)]
mod tests {
    use crate::protocol::safety::HttpSafety;
    use crate::util::encoding::ContentCodings;
    use crate::util::form::{MultiFormField, MultiFormFieldFile};

    use super::*;

    #[test]
    fn multipart_binary_files_round_trip_through_into_bin() {
        let first_data = vec![0, 0xff, b'\r', b'\n'];
        let second_data = vec![0x80, 0xfe, b'x'];
        let mut files = MultiFormField::new_file(MultiFormFieldFile::new(
            Some("first.bin".to_string()),
            None,
            first_data.clone(),
        ));
        files
            .insert_file(MultiFormFieldFile::new(
                Some("second.bin".to_string()),
                None,
                second_data.clone(),
            ))
            .unwrap();
        let mut form = MultiForm::try_new("body-boundary").unwrap();
        form.insert("files".to_string(), files);
        form.insert(
            "note".to_string(),
            MultiFormField::new_text("hello".to_string()),
        );

        let (data, content_type) = HttpBody::Files(form).into_bin();
        let decoded = HttpBody::Buffer {
            data,
            content_type: content_type.unwrap(),
            content_coding: ContentCodings::new(),
        }
        .decode(&HttpSafety::default())
        .unwrap();
        let HttpBody::Files(parsed) = decoded else {
            panic!("expected a decoded multipart form");
        };

        assert_eq!(parsed.boundary(), "body-boundary");
        assert_eq!(parsed.get_text("note").unwrap().unwrap(), "hello");
        let files = parsed.get_files("files").unwrap().unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].filename().as_deref(), Some("first.bin"));
        assert_eq!(files[0].data(), first_data);
        assert_eq!(files[1].filename().as_deref(), Some("second.bin"));
        assert_eq!(files[1].data(), second_data);
    }
}
