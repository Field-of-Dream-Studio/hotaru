//! Outgoing body preparation and compatibility wrappers.

use crate::util::content_type::HttpContentType;
use crate::protocol::HttpError;
use crate::util::encoding::{ContentCodings, HttpEncoding};

use super::{BodyError, HttpBody};

impl HttpBody {
    /// Converts a body into a buffer.
    ///
    /// The reverse operation is [`Self::decode`].
    ///
    /// Content types come from the body, including stored multipart boundaries.
    /// Existing buffers are returned unchanged, with their recorded content coding.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Encode bodies into prepared buffers",
        date = "2026-11-13"
    )]
    pub fn encode(self, content_coding: ContentCodings) -> Result<Self, BodyError> {
        let body = match self {
            Self::Unparsed => return Err(BodyError::IncompleteBody),
            Self::Buffer {
                data,
                content_type,
                content_coding,
            } => {
                return Ok(Self::Buffer {
                    data,
                    content_type,
                    content_coding,
                });
            }
            body => body,
        };
        let (data, suggested_content_type) = body.into_bin();
        let content_type =
            suggested_content_type.unwrap_or_else(HttpContentType::ApplicationOctetStream);
        let data = content_coding.encode_compressed(data)?;

        Ok(Self::Buffer {
            data,
            content_type,
            content_coding,
        })
    }

    /// Compatibility alias for [`Self::encode`].
    #[av::ver(
        deprecated,
        since = "0.8.6",
        note = "Use HttpBody::encode instead",
        date = "2026-11-13"
    )]
    pub fn into_buf(self, content_coding: ContentCodings) -> Result<Self, BodyError> {
        self.encode(content_coding)
    }

    /// Compatibility wrapper returning encoded bytes and their content type.
    #[av::ver(
        deprecated,
        since = "0.8.6",
        note = "Use HttpBody::encode instead",
        date = "2026-11-08"
    )]
    pub async fn into_static(
        self,
        encoding: &HttpEncoding,
    ) -> Result<(Vec<u8>, Option<HttpContentType>), HttpError> {
        match self.encode(encoding.content().clone())? {
            Self::Buffer {
                data, content_type, ..
            } => Ok((data, Some(content_type))),
            _ => unreachable!("encode always returns a buffer on success"),
        }
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
    use crate::util::encoding::{CompressionError, ContentCoding};

    use super::*;

    #[test]
    fn encode_uses_generated_content_type_and_preserves_bytes() {
        for (body, expected_type, expected_data) in [
            (
                HttpBody::Text("hello".to_string()),
                HttpContentType::TextHtml(),
                b"hello".to_vec(),
            ),
            (
                HttpBody::Binary(vec![0, 0xff]),
                HttpContentType::ApplicationOctetStream(),
                vec![0, 0xff],
            ),
            (
                HttpBody::Empty,
                HttpContentType::ApplicationOctetStream(),
                Vec::new(),
            ),
        ] {
            let HttpBody::Buffer {
                data,
                content_type,
                content_coding,
            } = body.encode(ContentCodings::new()).unwrap()
            else {
                panic!("expected a buffered body");
            };

            assert_eq!(data, expected_data);
            assert_eq!(content_type, expected_type);
            assert!(content_coding.is_identity());
        }
    }

    #[test]
    fn encode_propagates_compression_errors() {
        let mut coding = ContentCodings::new();
        coding.push(ContentCoding::Compress);

        assert!(matches!(
            HttpBody::Text("hello".to_string()).encode(coding),
            Err(BodyError::Compression(CompressionError::Unavailable(
                "compress"
            )))
        ));
    }
}
