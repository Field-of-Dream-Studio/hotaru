use akari::Value;

use crate::util::content_type::HttpContentType;
use crate::util::encoding::ContentCodings;
use crate::util::form::{MultiForm, UrlEncodedForm};

#[derive(Debug, Clone)]
pub enum HttpBody {
    Text(String),
    Binary(Vec<u8>),
    Form(UrlEncodedForm),
    Files(MultiForm),
    Json(Value),
    Empty,
    Unparsed,

    /// Content-coded bytes and decoding metadata, without transfer framing.
    /// Multipart boundaries remain in `content_type`; reuse bytes without re-encoding.
    Buffer {
        data: Vec<u8>,
        content_type: HttpContentType,
        content_coding: ContentCodings,
    },
}

impl HttpBody {
    /// Returns the default content type for a structured body, or the stored
    /// content type for a buffer. Empty and unparsed bodies have no content type.
    /// Text and binary defaults do not identify a more specific application type.
    #[av::ver(
        unstable,
        since = "0.8.6",
        note = "Body content type inference without serialization",
        date = "2026-11-11"
    )]
    pub fn get_content_type(&self) -> Option<HttpContentType> {
        match self {
            Self::Text(_) => Some(HttpContentType::TextHtml()),
            Self::Binary(_) => Some(HttpContentType::ApplicationOctetStream()),
            Self::Form(_) => Some(HttpContentType::ApplicationUrlEncodedForm()),
            Self::Files(files) => Some(HttpContentType::Multipart {
                subtype: "form-data".to_string(),
                boundary: Some(files.boundary().to_string()),
            }),
            Self::Json(_) => Some(HttpContentType::ApplicationJson()),
            Self::Buffer { content_type, .. } => Some(content_type.clone()),
            Self::Empty | Self::Unparsed => None,
        }
    }
}

impl Default for HttpBody {
    fn default() -> Self {
        Self::Unparsed
    }
}
