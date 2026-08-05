use akari::Value;

use crate::message::http_value::HttpContentType;
use crate::security::safety::HttpSafety;
use crate::util::form::{MultiForm, UrlEncodedForm};

use super::{BodyError, HttpBody};

impl HttpBody {
    /// Parses a buffered body according to its declared content type.
    pub(crate) fn parse_buffer(self, safety: &HttpSafety) -> Result<Self, BodyError> {
        fn parse_into_json(body: Vec<u8>) -> Result<HttpBody, BodyError> {
            let body = String::from_utf8(body).map_err(|_| BodyError::InvalidUtf8)?;
            let value = Value::from_json(&body).map_err(BodyError::InvalidJson)?;
            Ok(HttpBody::Json(value))
        }

        fn parse_into_text(body: Vec<u8>) -> Result<HttpBody, BodyError> {
            String::from_utf8(body)
                .map(HttpBody::Text)
                .map_err(|_| BodyError::InvalidUtf8)
        }

        fn parse_into_binary(body: Vec<u8>) -> HttpBody {
            HttpBody::Binary(body)
        }

        fn parse_into_form(body: Vec<u8>) -> HttpBody {
            HttpBody::Form(UrlEncodedForm::parse(body))
        }

        fn parse_into_files(body: Vec<u8>, boundary: String) -> HttpBody {
            HttpBody::Files(MultiForm::parse(body, boundary))
        }

        match self {
            Self::Buffer {
                data,
                content_type,
                content_coding,
            } => {
                if !safety.check_body_size(data.len()) {
                    return Err(BodyError::TooLarge);
                }

                let data = content_coding
                    .decode_compressed(data)
                    .map_err(|_| BodyError::InvalidEncoding)?;
                if !safety.check_body_size(data.len()) {
                    return Err(BodyError::TooLarge);
                }

                match content_type {
                    HttpContentType::Application { subtype, .. } if subtype == "json" => {
                        parse_into_json(data)
                    }
                    HttpContentType::Text { subtype, .. }
                        if subtype == "html" || subtype == "plain" =>
                    {
                        parse_into_text(data)
                    }
                    HttpContentType::Application { subtype, .. }
                        if subtype == "x-www-form-urlencoded" =>
                    {
                        Ok(parse_into_form(data))
                    }
                    HttpContentType::Multipart { subtype, boundary } if subtype == "form-data" => {
                        let boundary = boundary.ok_or_else(|| {
                            BodyError::InvalidMultipart(
                                "multipart/form-data boundary is missing".to_string(),
                            )
                        })?;
                        Ok(parse_into_files(data, boundary))
                    }
                    _ => Ok(parse_into_binary(data)),
                }
            }
            _ => Ok(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::message::meta::HttpMeta;
    use crate::message::request::HttpRequest;
    use crate::util::encoding::ContentCodings;

    use super::*;

    fn buffered_json(data: Vec<u8>) -> HttpBody {
        HttpBody::Buffer {
            data,
            content_type: HttpContentType::ApplicationJson(),
            content_coding: ContentCodings::new(),
        }
    }

    #[test]
    fn parses_valid_json() {
        let body = buffered_json(br#"{"name":"Hotaru"}"#.to_vec())
            .parse_buffer(&HttpSafety::new())
            .unwrap();

        assert!(matches!(body, HttpBody::Json(_)));
    }

    #[test]
    fn rejects_invalid_utf8_json() {
        let result = buffered_json(vec![0xff]).parse_buffer(&HttpSafety::new());

        assert!(matches!(result, Err(BodyError::InvalidUtf8)));
    }

    #[test]
    fn rejects_oversized_buffer() {
        let mut safety = HttpSafety::new();
        safety.set_max_body_size(Some(1));

        let result = buffered_json(b"{}".to_vec()).parse_buffer(&safety);

        assert!(matches!(result, Err(BodyError::TooLarge)));
    }

    #[tokio::test]
    async fn repeated_parse_returns_the_same_error_class() {
        let mut request = HttpRequest::new(HttpMeta::default(), buffered_json(b"{".to_vec()));
        let safety = HttpSafety::new();

        let first = request.parse_body(&safety).await;
        let second = request.parse_body(&safety).await;

        assert!(matches!(first, Err(BodyError::InvalidJson(_))));
        assert!(matches!(second, Err(BodyError::InvalidJson(_))));
        assert!(matches!(request.body, HttpBody::Buffer { .. }));
    }
}
