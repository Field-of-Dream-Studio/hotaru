use crate::message::header::HeaderMap;
use crate::message::http_value::*;
use crate::message::start_line::HttpStartLine;
use crate::util::cookie::CookieMap;
use crate::util::encoding::HttpEncoding;

/// RequestHeader is a struct that represents the headers of an HTTP request.
///
/// Fields marked `pub(in crate::message::meta)` are kept module-private but
/// visible to the sibling submodules (`attributes`, `content_length`, `cookie`,
/// etc.) that carry the split `impl HttpMeta` blocks. The visibility mirrors
/// the previous single-file layout, where these fields were plain `private`
/// and reachable only from within `meta.rs` itself.
#[derive(Debug, Clone)]
pub struct HttpMeta {
    pub start_line: HttpStartLine,
    pub header: HeaderMap,

    // Content-type header, overrides the content type from the hashmap if present
    pub(in crate::message::meta) content_type: Option<HttpContentType>,

    // Content-length header, overrides the content length from the hashmap if present
    pub(in crate::message::meta) content_length: Option<u64>,

    // Cookies header in request, Set-Cookie header in response
    pub(in crate::message::meta) cookies: Option<CookieMap>,

    // Content-Disposition header, used for file downloads in responses
    pub(in crate::message::meta) content_disposition: Option<ContentDisposition>,

    /// Transfer-Encoding header, used for chunked transfer encoding in responses
    pub(in crate::message::meta) encoding: Option<HttpEncoding>,

    // Host header, overrides the content length from the hashmap if present
    pub(in crate::message::meta) host: Option<String>,

    // Accept-Language header in request and Content-Language header in response
    // Overrides the content length from the hashmap if present
    pub(in crate::message::meta) lang: Option<AcceptLang>,

    /// Location header, used for redirects in responses
    pub(in crate::message::meta) location: Option<String>,
}

impl HttpMeta {
    /// It is used to create a new RequestHeader object.
    ///
    /// `headers` accepts any type that converts into [`HeaderMap`], including
    /// a bare `HashMap<String, HeaderValue>` for backwards compatibility.
    pub fn new(start_line: HttpStartLine, headers: impl Into<HeaderMap>) -> Self {
        Self {
            start_line,
            header: headers.into(),
            content_type: None,
            content_length: None,
            content_disposition: None,
            cookies: None,
            encoding: None,
            host: None,
            lang: None,
            location: None,
        }
    }
}

impl Default for HttpMeta {
    fn default() -> Self {
        Self {
            start_line: HttpStartLine::new_request(
                HttpVersion::Http11,
                HttpMethod::GET,
                "/".to_string(),
            ),
            header: HeaderMap::new(),
            content_type: None,
            content_length: None,
            content_disposition: None,
            cookies: None,
            encoding: None,
            host: None,
            lang: None,
            location: None,
        }
    }
}
