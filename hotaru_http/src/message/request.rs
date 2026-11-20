use crate::protocol::{HttpSafety, HttpSyncSettings};
use crate::util::cookie::Cookie;

use crate::context::io;
use crate::message::body::{BodyError, HttpBody};
use crate::message::http_value::*;
use crate::util::content_type::HttpContentType;
use crate::message::meta::HttpMeta;
use crate::message::start_line::HttpStartLine;
use hotaru_core::connection::{HotaruBufRead, HotaruWrite};
use std::collections::HashMap;

/// Represents an HTTP request with metadata and body.
///
/// This struct contains all information about an incoming HTTP request,
/// including headers, method, URL, and body content.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub meta: HttpMeta,
    pub body: HttpBody,
}

impl HttpRequest {
    pub fn new(meta: HttpMeta, body: HttpBody) -> Self {
        HttpRequest { meta, body }
    }

    pub fn meta(&self) -> &HttpMeta {
        &self.meta
    }

    /// Consumes the request and returns it with a prepared body buffer and
    /// matching metadata. Content types are reconciled with the body's generated
    /// type; multipart boundaries come from the body. No stream I/O is performed.
    #[av::ver(
        unstable,
        since = "0.8.6",
        note = "Synchronize outgoing metadata with the prepared body",
        date = "2026-11-12"
    )]
    pub fn sync(self) -> Result<Self, crate::protocol::HttpError> {
        self.config_sync(&HttpSyncSettings::default())
    }

    /// Prepares the request body and updates metadata according to `settings`.
    /// The flags control encoding, length, content-type, and Date header updates.
    /// Date synchronization fills missing or unparseable values when a clock value
    /// is available. Body preparation still runs.
    #[av::ver(
        unstable,
        since = "0.8.6",
        note = "Prepare an HTTP request with selective metadata synchronization",
        date = "2026-11-12"
    )]
    pub fn config_sync(
        self,
        settings: &HttpSyncSettings,
    ) -> Result<Self, crate::protocol::HttpError> {
        let (meta, body) = super::sync::config_sync(self.meta, self.body, settings)?;
        Ok(Self::new(meta, body))
    }

    /// Check whether this request allows the connection to remain open.
    pub fn is_keep_alive(&self) -> bool {
        self.meta.is_keep_alive()
    }

    /// Reads the request headers and complete body into a buffer.
    /// Content decoding is deferred until [`HttpRequest::decode_body`] is called.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Renamed from parse_lazy",
        date = "2026-11-08"
    )]
    pub async fn receive<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        stream: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<Self, crate::protocol::HttpError> {
        let (meta, body) = io::receive(stream, config, true, print_raw).await?;
        Ok(Self::new(meta, body))
    }

    /// Compatibility alias for [`HttpRequest::receive`].
    #[av::ver(
        deprecated,
        since = "0.8.6",
        note = "Use HttpRequest::receive instead",
        date = "2026-11-08"
    )]
    pub async fn parse_lazy<R: HotaruBufRead<Error = std::io::Error> + Unpin + Send>(
        stream: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<Self, crate::protocol::HttpError> {
        Self::receive(stream, config, print_raw).await
    }

    /// Decodes and interprets the already-buffered request body.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Renamed from parse_body",
        date = "2026-11-08"
    )]
    pub async fn decode_body(&mut self, safety_setting: &HttpSafety) -> Result<(), BodyError> {
        let body = self.body.clone().decode(safety_setting)?;
        self.body = body;
        Ok(())
    }

    /// Compatibility alias for [`HttpRequest::decode_body`].
    #[av::ver(
        deprecated,
        since = "0.8.6",
        note = "Use HttpRequest::decode_body instead",
        date = "2026-11-08"
    )]
    pub async fn parse_body(&mut self, safety_setting: &HttpSafety) -> Result<(), BodyError> {
        self.decode_body(safety_setting).await
    }

    /// Add a cookie into the request metadata.
    pub fn add_cookie<T: Into<String>>(mut self, key: T, cookie: Cookie) -> Self {
        self.meta.add_cookie(key, cookie);
        self
    }

    /// Set content type for Http Response
    pub fn content_type(mut self, content_type: HttpContentType) -> Self {
        self.meta.set_content_type(content_type);
        self
    }

    /// Add a header for Http Request
    pub fn add_header<T: Into<String>, U: Into<String>>(mut self, key: T, value: U) -> Self {
        self.meta.set_attribute(key, value.into());
        self
    }

    /// Set the content disposition for the request.
    pub fn content_disposition(mut self, disposition: ContentDisposition) -> Self {
        self.meta.set_content_disposition(disposition);
        self
    }

    /// Sends the request after synchronizing it according to `settings`.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Send HTTP requests with explicit synchronization settings",
        date = "2026-11-13"
    )]
    pub async fn send<W: HotaruWrite<Error = std::io::Error> + Unpin + Send>(
        self,
        writer: &mut W,
        settings: &HttpSyncSettings,
    ) -> Result<(), crate::protocol::HttpError> {
        io::send(self.meta, self.body, writer, settings).await
    }
}

impl Default for HttpRequest {
    fn default() -> Self {
        let meta = HttpMeta::new(
            HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, "/".to_string()),
            HashMap::new(),
        );
        let body = HttpBody::Empty;
        HttpRequest::new(meta, body)
    }
}

/// Collection of helper functions to easily create common HTTP requests.
pub mod request_templates {
    use std::collections::HashMap;

    use akari::Value;

    use crate::message::body::HttpBody;
    use crate::message::http_value::{HttpMethod, HttpVersion};
    use crate::util::content_type::HttpContentType;
    use crate::message::meta::HttpMeta;
    use crate::message::start_line::HttpStartLine;
    use crate::util::form::UrlEncodedForm;

    use super::HttpRequest;

    pub fn get_request<T: Into<String>>(url: T) -> HttpRequest {
        let meta = HttpMeta::new(
            HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, url.into()),
            HashMap::new(),
        );
        let body = HttpBody::Empty;
        HttpRequest::new(meta, body)
    }

    pub fn json_request<T: Into<String>>(url: T, body: Value) -> HttpRequest {
        let start_line =
            HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::POST, url.into());
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::ApplicationJson());
        HttpRequest::new(meta, HttpBody::Json(body))
    }

    /// Creates a POST request with form data (application/x-www-form-urlencoded)
    /// User passes in a UrlEncodedForm directly
    pub fn form_post<T: Into<String>>(url: T, form: UrlEncodedForm) -> HttpRequest {
        let start_line =
            HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::POST, url.into());
        let meta = HttpMeta::new(start_line, HashMap::new());
        HttpRequest::new(meta, HttpBody::Form(form))
    }

    /// Creates a GET request with query parameters
    /// Builds the query string using UrlEncodedForm's encoding
    pub fn get_with_params<T: Into<String>>(
        url: T,
        params: HashMap<String, String>,
    ) -> HttpRequest {
        let base_url = url.into();

        // Build query string using UrlEncodedForm
        let query = if params.is_empty() {
            String::new()
        } else {
            let form = UrlEncodedForm::from(params);
            let encoded = form.to_string();

            if base_url.contains('?') {
                format!("&{}", encoded)
            } else {
                format!("?{}", encoded)
            }
        };

        let full_url = format!("{}{}", base_url, query);
        get_request(full_url)
    }

    /// Creates a POST request with plain text body
    pub fn text_post<T: Into<String>, B: Into<String>>(url: T, text: B) -> HttpRequest {
        let start_line =
            HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::POST, url.into());
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::Text {
            subtype: "plain".to_string(),
            charset: Some("utf-8".to_string()),
        });
        HttpRequest::new(meta, HttpBody::Binary(text.into().into_bytes()))
    }

    /// Creates a PUT request with JSON body
    pub fn json_put<T: Into<String>>(url: T, body: Value) -> HttpRequest {
        let start_line =
            HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::PUT, url.into());
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::ApplicationJson());
        HttpRequest::new(meta, HttpBody::Json(body))
    }

    /// Creates a PATCH request with JSON body
    pub fn json_patch<T: Into<String>>(url: T, body: Value) -> HttpRequest {
        let start_line =
            HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::PATCH, url.into());
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::ApplicationJson());
        HttpRequest::new(meta, HttpBody::Json(body))
    }

    /// Creates a DELETE request
    pub fn delete<T: Into<String>>(url: T) -> HttpRequest {
        let start_line =
            HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::DELETE, url.into());
        let meta = HttpMeta::new(start_line, HashMap::new());
        HttpRequest::new(meta, HttpBody::Empty)
    }

    /// Creates a HEAD request
    pub fn head<T: Into<String>>(url: T) -> HttpRequest {
        let start_line =
            HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::HEAD, url.into());
        let meta = HttpMeta::new(start_line, HashMap::new());
        HttpRequest::new(meta, HttpBody::Empty)
    }
}
