use akari::Value;
use hotaru_core::app::common::{RunMode, RuntimeConfig};
use hotaru_core::connection::TransportSpec;
use hotaru_core::connection::error::ConnectionError;
use hotaru_core::debug_log;
use hotaru_core::extensions::{Locals, Params};
use hotaru_core::protocol::{
    BoxProtocolError, EndpointOutcome, ProtocolError, ProtocolRole, RequestContext,
};
use hotaru_core::url::UrlNode;

use hotaru_core::connection::{HotaruBufRead, HotaruWrite};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use crate::channel::Http1Channel;
use crate::message::body::{BodyError, HttpBody};
use crate::message::http_value::{HttpMethod, StatusCode};
use crate::message::meta::HttpMeta;
use crate::message::request::HttpRequest;
use crate::message::response::{HttpResponse, response_templates};
use crate::protocol::HttpError;
use crate::security::safety::HttpSafety;

use crate::util::cookie::{Cookie, CookieMap};
use crate::util::form::{MultiForm, UrlEncodedForm};

/// Executable context - determines what's available for execution
pub enum Executable<TS: TransportSpec = hotaru_io_tokio::TcpTransport> {
    /// Server context with runtime config and matched endpoint.
    Request {
        runtime: Arc<RuntimeConfig>,
        endpoint: Arc<UrlNode<HttpContext<TS>, TS>>,
    },
    /// Client context (empty for now, will be extended later)
    Response,
}

/// Unified HTTP context for both server and client operations.
///
/// This context flows through handlers and middleware, supporting both
/// server-side request handling and client-side response processing.
pub struct HttpContext<TS: TransportSpec = hotaru_io_tokio::TcpTransport> {
    pub request: HttpRequest,
    pub response: HttpResponse,

    // Execution context determines available operations
    pub executable: Executable<TS>,

    // Additional fields
    pub host: Option<String>, // Used by client for target host
    pub safety: HttpSafety,

    // Socket addresses
    remote_addr: Option<SocketAddr>,
    local_addr: Option<SocketAddr>,

    // Shared fields for middleware/handlers
    pub params: Params,
    pub locals: Locals,

    // Protocol-private exchange channel. Kept off the RequestContext trait.
    channel: Option<Http1Channel<TS::Wire>>,
}

// Type alias for backward compatibility
pub type HttpReqCtx<TS = hotaru_io_tokio::TcpTransport> = HttpContext<TS>;

/// Placeholder address for uninitialized or unknown connections.
/// `0.0.0.0:0` indicates that no socket address information is available.
const UNSET_ADDR: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 0);

impl<TS: TransportSpec> HttpContext<TS> {
    /// Creates a new server context with socket addresses.
    ///
    /// `safety` is the per-connection baseline supplied by the protocol;
    /// per-endpoint overrides overlay on top of it in `request_check` /
    /// `parse_body`.
    pub fn new_server(
        runtime: Arc<RuntimeConfig>,
        endpoint: Arc<UrlNode<HttpContext<TS>, TS>>,
        request: HttpRequest,
        remote_addr: Option<SocketAddr>,
        local_addr: Option<SocketAddr>,
        safety: HttpSafety,
    ) -> Self {
        Self {
            request,
            response: HttpResponse::default(),
            executable: Executable::Request { runtime, endpoint },
            host: None,
            safety,
            remote_addr,
            local_addr,
            params: Default::default(),
            locals: Default::default(),
            channel: None,
        }
    }

    /// Creates a new client context.
    ///
    /// An empty `host` is normalized to `None` so downstream Host auto-fill
    /// never emits a `Host:` header with an empty value (which nginx and
    /// other strict servers reject with 400).
    pub fn new_client(host: String, safety: HttpSafety) -> Self {
        Self {
            request: HttpRequest::default(),
            response: HttpResponse::default(),
            executable: Executable::<TS>::Response,
            host: if host.is_empty() { None } else { Some(host) },
            safety,
            remote_addr: None,
            local_addr: None,
            params: Default::default(),
            locals: Default::default(),
            channel: None,
        }
    }

    pub(crate) fn install_channel(&mut self, channel: Http1Channel<TS::Wire>) {
        self.channel = Some(channel);
    }

    pub(crate) fn channel(&self) -> Option<&Http1Channel<TS::Wire>> {
        self.channel.as_ref()
    }

    // /// Creates a new Request Context (backward compatibility)
    // #[deprecated(note = "Use new_server() with socket addresses instead")]
    // pub fn new(app: Arc<App>, endpoint: Arc<Url<HttpContext>>, request: HttpRequest) -> Self {
    //     Self::new_server(app, endpoint, request, UNSET_ADDR, UNSET_ADDR)
    // }

    // /// Handles the request by parsing it and creating a new `HttpContext`.
    // #[deprecated(note = "Use new_server() with socket addresses instead")]
    // pub async fn handle(
    //     app: Arc<App>,
    //     root_handler: Arc<Url<HttpContext>>,
    //     request: HttpRequest,
    // ) -> Self {
    //     let endpoint = root_handler.walk_str(&request.meta.path()).await;
    //     Self::new(app.clone(), endpoint.clone(), request)
    // }

    // =========================================================================
    // Socket Address Accessors
    // =========================================================================

    /// Returns the client's socket address (IP and port).
    /// For server context, this is the remote peer that connected.
    /// For client context, this is the server we connected to.
    /// If unknown, this returns `None`.
    #[inline]
    pub fn client_ip(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    /// Returns the client's socket address (IP and port), or `0.0.0.0:0` if unknown.
    #[inline]
    pub fn client_ip_or_default(&self) -> SocketAddr {
        self.remote_addr.unwrap_or(UNSET_ADDR)
    }

    /// Returns just the client's IP address without the port.
    /// If unknown, this returns `None`.
    #[inline]
    pub fn client_ip_only(&self) -> Option<IpAddr> {
        self.remote_addr.map(|addr| addr.ip())
    }

    /// Returns just the client's IP address without the port, or `0.0.0.0` if unknown.
    #[inline]
    pub fn client_ip_only_or_default(&self) -> IpAddr {
        self.remote_addr
            .map(|addr| addr.ip())
            .unwrap_or(UNSET_ADDR.ip())
    }

    /// Returns the server's bound socket address.
    /// For server context, this is the local address we're listening on.
    /// For client context, this is our local ephemeral port.
    /// If unknown, this returns `None`.
    #[inline]
    pub fn server_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Returns the server's bound socket address, or `0.0.0.0:0` if unknown.
    #[inline]
    pub fn server_addr_or_default(&self) -> SocketAddr {
        self.local_addr.unwrap_or(UNSET_ADDR)
    }

    /// Returns the remote socket address (alias for client_ip in server context).
    /// If unknown, this returns `None`.
    #[inline]
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    /// Returns the remote socket address, or `0.0.0.0:0` if unknown.
    #[inline]
    pub fn remote_addr_or_default(&self) -> SocketAddr {
        self.remote_addr.unwrap_or(UNSET_ADDR)
    }

    /// Returns the local socket address (alias for server_addr).
    /// If unknown, this returns `None`.
    #[inline]
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Returns the local socket address, or `0.0.0.0:0` if unknown.
    #[inline]
    pub fn local_addr_or_default(&self) -> SocketAddr {
        self.local_addr.unwrap_or(UNSET_ADDR)
    }

    pub async fn read_request<R>(
        runtime: Arc<RuntimeConfig>,
        reader: &mut R,
    ) -> Result<HttpRequest, ConnectionError>
    where
        R: HotaruBufRead<Error = std::io::Error> + Unpin + Send,
    {
        Ok(HttpRequest::parse_lazy(
            reader,
            &runtime.get_config::<HttpSafety>().unwrap_or_default(),
            runtime.mode() == RunMode::Build,
        )
        .await)
    }

    /// Sends the response
    pub async fn send_response<W>(response: HttpResponse, writer: &mut W)
    where
        W: HotaruWrite<Error = std::io::Error> + Unpin + Send,
    {
        let _ = response.send(writer).await;
    }

    /// Runs the endpoint and returns the context with the response set.
    ///
    /// # Return
    ///
    /// Returns `Ok(HttpContext)` with the response populated, or `Err(BoxProtocolError)`
    /// if the endpoint chain fails. Keep-alive is managed by `Http1Channel.open`.
    pub async fn run(mut self) -> Result<HttpContext<TS>, BoxProtocolError> {
        if let Some(endpoint) = self.endpoint() {
            debug_log!("HTTP Context: Found endpoint, checking request");
            if let Err(err) = self.request_check(&endpoint) {
                debug_log!("HTTP Context: Request check failed: {:?}", err);
                let status: StatusCode = (&err).into();
                self.response = response_templates::return_status(status);
                return Ok(self);
            };
            debug_log!("HTTP Context: Running endpoint handler");
            let result = endpoint.run(self).await.map_err(|e| e.boxed());
            debug_log!("HTTP Context: Handler completed");
            result
        } else {
            debug_log!("HTTP Context: No endpoint available (client context)");
            // No endpoint available (client context)
            Ok(self)
        }
    }

    /// Checks whether the request fulfills the endpoint's security requirements.
    ///
    /// Returns `Ok(())` if the request passes all checks, or `Err(HttpError)` with
    /// the appropriate error variant if a check fails.
    pub fn request_check(
        &mut self,
        endpoint: &Arc<UrlNode<HttpContext<TS>, TS>>,
    ) -> Result<(), HttpError> {
        // Start from the protocol baseline (`self.safety`); overlay any
        // per-endpoint override on top.
        let mut config = self.safety.clone();
        if let Some(ep) = endpoint.get_params::<HttpSafety>() {
            config.update(&ep);
        }
        if !config.check_body_size(self.request.meta.get_content_length().unwrap_or(0)) {
            return Err(HttpError::PayloadTooLarge);
        }
        if !config.check_method(&self.request.meta.method()) {
            return Err(HttpError::MethodNotAllowed);
        }
        if !config.check_content_type(&self.request.meta.get_content_type().unwrap_or_default()) {
            return Err(HttpError::UnsupportedMediaType);
        }
        return Ok(());
    }

    /// Returns the meta in the request as reference
    pub fn meta(&mut self) -> &mut HttpMeta {
        &mut self.request.meta
    }

    /// Returns the runtime config if this is a server context.
    pub fn runtime(&self) -> Option<Arc<RuntimeConfig>> {
        match &self.executable {
            Executable::Request { runtime, .. } => Some(runtime.clone()),
            Executable::<TS>::Response => None,
        }
    }

    /// Returns the endpoint URL if this is a server context
    pub fn endpoint(&self) -> Option<Arc<UrlNode<HttpContext<TS>, TS>>> {
        match &self.executable {
            Executable::Request { endpoint, .. } => Some(endpoint.clone()),
            Executable::<TS>::Response => None,
        }
    }

    /// Parses the body of the request, reading it into the `HttpBody` field of the request.
    /// Note that request body will not be automatically parsed unless this function is called
    /// The automatic parsing is not recommended, as it can lead to performance issues and security vulnerabilities.
    /// If you didn't parse body, the body will be `HttpBody::Unparsed`.
    pub async fn parse_body(&mut self) -> Result<(), BodyError> {
        // Start from the protocol baseline (`self.safety`); overlay any
        // per-endpoint override on top. (The prior implementation fetched
        // `endpoint.get_params::<HttpSafety>()` twice — the second call was
        // a no-op clone, and the baseline was ignored entirely.)
        let mut settings = self.safety.clone();
        if let Some(endpoint) = self.endpoint() {
            if let Some(ep) = endpoint.get_params::<HttpSafety>() {
                settings.update(&ep);
            }
        }

        let body = self.request.body.clone().parse_buffer(&settings)?;
        self.request.body = body;
        Ok(())
    }

    /// Parses and returns the URL-encoded request body.
    pub async fn form(&mut self) -> Result<&UrlEncodedForm, BodyError> {
        let content_type = self
            .request
            .meta
            .get_content_type()
            .map(|value| value.to_string());
        self.parse_body().await?;
        if let HttpBody::Form(ref data) = self.request.body {
            Ok(data)
        } else if matches!(self.request.body, HttpBody::Empty | HttpBody::Unparsed) {
            Err(BodyError::Missing)
        } else {
            Err(content_type
                .map(BodyError::UnsupportedContentType)
                .unwrap_or(BodyError::MissingContentType))
        }
    }

    /// Parses and returns the multipart request body.
    pub async fn files(&mut self) -> Result<&MultiForm, BodyError> {
        let content_type = self
            .request
            .meta
            .get_content_type()
            .map(|value| value.to_string());
        self.parse_body().await?;
        if let HttpBody::Files(ref data) = self.request.body {
            Ok(data)
        } else if matches!(self.request.body, HttpBody::Empty | HttpBody::Unparsed) {
            Err(BodyError::Missing)
        } else {
            Err(content_type
                .map(BodyError::UnsupportedContentType)
                .unwrap_or(BodyError::MissingContentType))
        }
    }

    /// Parses and returns the JSON request body.
    pub async fn json(&mut self) -> Result<&Value, BodyError> {
        let content_type = self
            .request
            .meta
            .get_content_type()
            .map(|value| value.to_string());
        self.parse_body().await?;
        if let HttpBody::Json(ref data) = self.request.body {
            Ok(data)
        } else if matches!(self.request.body, HttpBody::Empty | HttpBody::Unparsed) {
            Err(BodyError::Missing)
        } else {
            Err(content_type
                .map(BodyError::UnsupportedContentType)
                .unwrap_or(BodyError::MissingContentType))
        }
    }

    /// Get a path segment by index position, skipping the implicit leading empty
    /// segment produced by the leading `/` in HTTP paths.
    /// For example, in "/api/users/123", segment(0) = "api", segment(1) = "users", segment(2) = "123"
    pub fn segment(&mut self, index: usize) -> String {
        self.request.meta.get_path(index + 1)
    }

    /// Get the whole path
    pub fn path(&self) -> String {
        self.request.meta.path()
    }

    /// Get a named path parameter from the URL pattern
    /// For example, with pattern "/users/<id>", param("id") returns the value in place of <id>
    pub fn param<A: AsRef<str>>(&mut self, name: A) -> Option<String> {
        self.endpoint().and_then(|endpoint| {
            endpoint
                .match_seg_name_with_index(name)
                .map(|index| self.request.meta.get_path(index))
        })
    }

    /// Alias for param() - kept for backward compatibility
    pub fn pattern<A: AsRef<str>>(&mut self, name: A) -> Option<String> {
        self.param(name)
    }

    /// Get a query parameter value by key
    /// For example, in "/search?q=rust&limit=10", query("q") returns Some("rust")
    pub fn query<T: Into<String>>(&mut self, key: T) -> Option<String> {
        self.request.meta.get_url_args(key)
    }

    /// Get the preferred by the user
    pub fn get_preferred_language(&mut self) -> Option<String> {
        self.request
            .meta
            .get_lang()
            .map(|lang_dict| lang_dict.most_preferred())
    }

    /// Get the preferred by the user with a default value
    pub fn get_preferred_language_or_default<T: AsRef<str>>(&mut self, default: T) -> String {
        self.get_preferred_language()
            .unwrap_or_else(|| default.as_ref().to_string())
    }

    /// Returns the method of the request.
    pub fn method(&mut self) -> HttpMethod {
        self.request.meta.method()
    }

    /// Convenience method to get request headers directly.
    /// Avoids the long chain: req.request.meta.header
    pub fn headers(&self) -> &HashMap<String, crate::message::meta::HeaderValue> {
        &self.request.meta.header
    }

    /// Convenience method to get a specific header value.
    pub fn header(&self, key: &str) -> Option<&crate::message::meta::HeaderValue> {
        self.request.meta.header.get(key)
    }

    /// Convenience method to get a header value as a string.
    /// Returns the first value if multiple values exist.
    pub fn header_str(&self, key: &str) -> Option<&str> {
        self.request.meta.header.get(key).and_then(|hv| match hv {
            crate::message::meta::HeaderValue::Single(s) => Some(s.as_str()),
            crate::message::meta::HeaderValue::Multiple(v) => v.first().map(|s| s.as_str()),
        })
    }

    /// Convenience method to check if a header exists.
    pub fn has_header(&self, key: &str) -> bool {
        self.request.meta.header.contains_key(key)
    }

    /// Get the full cookie map
    pub fn get_cookies(&mut self) -> &CookieMap {
        self.request.meta.get_cookies()
    }

    /// Get a single cookie
    pub fn get_cookie(&mut self, key: &str) -> Option<Cookie> {
        self.request.meta.get_cookie(key)
    }

    /// Get a cookie. If not found a default cookie will be returned
    pub fn get_cookie_or_default<T: AsRef<str>>(&mut self, key: T) -> Cookie {
        self.request.meta.get_cookie_or_default(key)
    }

    // ========================================================================
    // Response convenience methods
    // ========================================================================

    /// Get a mutable reference to the response for chaining.
    pub fn response_mut(&mut self) -> &mut HttpResponse {
        &mut self.response
    }

    /// Set the response status code.
    pub fn set_status(&mut self, code: u16) -> &mut Self {
        self.response.meta.start_line.set_status_code(code);
        self
    }

    /// Add a response header.
    pub fn add_response_header(&mut self, key: String, value: String) -> &mut Self {
        self.response.meta.set_attribute(key, value);
        self
    }

    /// Set the response body.
    pub fn set_body(&mut self, body: HttpBody) -> &mut Self {
        self.response.body = body;
        self
    }

    /// Take the request out of this HTTP context for protocol transmission.
    ///
    /// If the request does not already carry a Host value, use the context's
    /// configured host when it is non-empty.
    pub(crate) fn take_request(&mut self) -> HttpRequest {
        if self.request.meta.get_host().is_none() {
            if let Some(host) = self.host.as_deref().filter(|h| !h.is_empty()) {
                self.request.meta.set_host(Some(host.to_string()));
            }
        }

        std::mem::take(&mut self.request)
    }

    /// Store a protocol response into this HTTP context.
    pub(crate) fn set_response(&mut self, response: HttpResponse) {
        self.response = response;
    }
}

impl<TS: TransportSpec> RequestContext for HttpContext<TS> {
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = crate::protocol::HttpError;
    type Channel = Http1Channel<TS::Wire>;

    fn handle_error(&mut self) {
        match &self.executable {
            Executable::Request { .. } => {
                // Server: return a generic error response.
                // The specific status code is determined by the error type
                // in the protocol-level handle loop via error_response_from().
                self.response = response_templates::html_response(
                    "<h1>500 Internal Server Error</h1><br><p>An unexpected error occurred</p>",
                )
                .status(500);
            }
            Executable::<TS>::Response => {
                // Client: set default error response
                self.response = HttpResponse::default();
            }
        }
    }

    fn role(&self) -> ProtocolRole {
        match &self.executable {
            Executable::Request { .. } => ProtocolRole::Server,
            Executable::<TS>::Response => ProtocolRole::Client,
        }
    }

    fn inject_request(&mut self, request: Self::Request) {
        self.request(request);
    }

    fn into_response(self) -> Self::Response {
        self.response
    }
}

/// Endpoint bodies returning `HttpResponse` keep working: the value is stored
/// into the context's response slot here instead of by the macro wrapper.
impl<TS: TransportSpec> EndpointOutcome<HttpContext<TS>> for HttpResponse {
    fn apply_to(self, ctx: &mut HttpContext<TS>) -> Result<(), HttpError> {
        ctx.response = self;
        Ok(())
    }
}

impl<TS: TransportSpec> Default for HttpContext<TS> {
    fn default() -> Self {
        Self::new_client(String::new(), HttpSafety::default())
    }
}

impl<TS: TransportSpec> HttpContext<TS> {
    pub fn bad_request(&mut self) {
        self.handle_error();
    }
}

// Type alias for backward compatibility with client code
pub type HttpResCtx<TS = hotaru_io_tokio::TcpTransport> = HttpContext<TS>;

impl<TS: TransportSpec> HttpContext<TS> {
    /// Creates a client context for sending requests (backward compatibility)
    pub fn new_res(config: HttpSafety, host: impl Into<String>) -> Self {
        Self::new_client(host.into(), config)
    }

    pub fn request(&mut self, mut request: HttpRequest) {
        if request.meta.get_host().is_none() {
            if let Some(host) = self.host.as_deref().filter(|h| !h.is_empty()) {
                request.meta.set_host(Some(host.to_string()));
            }
        };
        self.request = request;
    }
}
