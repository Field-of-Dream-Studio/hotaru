use crate::message::http_value::*;

use super::error::StartLineError;
use super::request::RequestStartLine;
use super::response::ResponseStartLine;

/// HttpStartLine is an enum that can represent either a RequestStartLine or a ResponseStartLine.
/// It provides a unified API for working with HTTP start lines regardless of their type.
#[derive(Debug, Clone)]
pub enum HttpStartLine {
    Request(RequestStartLine),
    Response(ResponseStartLine),
}

impl HttpStartLine {
    /// Creates a new HTTP request start line.
    ///
    /// # Arguments
    ///
    /// * `http_version` - The HTTP version.
    /// * `method` - The HTTP method.
    /// * `path` - The request path.
    ///
    /// # Returns
    ///
    /// A new `HttpStartLine::Request` variant.
    pub fn new_request(http_version: HttpVersion, method: HttpMethod, path: String) -> Self {
        Self::Request(RequestStartLine::new(http_version, method, path))
    }

    /// Creates a new HTTP response start line.
    ///
    /// # Arguments
    ///
    /// * `http_version` - The HTTP version.
    /// * `status_code` - The response status code.
    ///
    /// # Returns
    ///
    /// A new `HttpStartLine::Response` variant.
    pub fn new_response(http_version: HttpVersion, status_code: StatusCode) -> Self {
        Self::Response(ResponseStartLine::new(http_version, status_code))
    }

    /// Parses a string into an HTTP start line.
    ///
    /// This method attempts to parse the string as a request first, and if that fails,
    /// it tries to parse it as a response.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that contains the start line.
    ///
    /// # Returns
    ///
    /// `Ok(Self)` on success, or [`StartLineError::Unrecognised`] if the line
    /// matched neither shape.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::start_line::HttpStartLine;
    /// let line = "GET /index.html HTTP/1.1";
    /// let start_line = HttpStartLine::parse(line).unwrap();
    /// assert!(start_line.is_request());
    /// ```
    pub fn parse<T: AsRef<str>>(line: T) -> Result<Self, StartLineError> {
        let line = line.as_ref();

        if let Ok(request) = Self::parse_request(line) {
            return Ok(request);
        }

        if let Ok(response) = Self::parse_response(line) {
            return Ok(response);
        }

        Err(StartLineError::Unrecognised)
    }

    /// Parses a string specifically as an HTTP request start line.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that contains the request start line.
    ///
    /// # Returns
    ///
    /// `Ok(Self)` on success, or a [`StartLineError`] describing the failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::start_line::HttpStartLine;
    /// let line = "GET /index.html HTTP/1.1";
    /// let start_line = HttpStartLine::parse_request(line).unwrap();
    /// assert!(start_line.is_request());
    /// ```
    pub fn parse_request<T: AsRef<str>>(line: T) -> Result<Self, StartLineError> {
        RequestStartLine::parse(line).map(Self::Request)
    }

    /// Parses a string specifically as an HTTP response start line.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that contains the response start line.
    ///
    /// # Returns
    ///
    /// `Ok(Self)` on success, or a [`StartLineError`] describing the failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::start_line::HttpStartLine;
    /// let line = "HTTP/1.1 200 OK";
    /// let start_line = HttpStartLine::parse_response(line).unwrap();
    /// assert!(start_line.is_response());
    /// ```
    pub fn parse_response<T: AsRef<str>>(line: T) -> Result<Self, StartLineError> {
        ResponseStartLine::parse(line).map(Self::Response)
    }

    /// Parses a string into an HTTP start line, returning a caller-supplied default if parsing fails.
    ///
    /// # Arguments
    ///
    /// * `line` - The string to parse.
    /// * `default` - The default value to return if parsing fails.
    ///
    /// # Returns
    ///
    /// Either the parsed HttpStartLine or the caller-supplied default.
    pub fn parse_or<T: AsRef<str>>(line: T, default: Self) -> Self {
        Self::parse(line).unwrap_or(default)
    }

    /// Parses a string into an HTTP start line, returning the type default if parsing fails.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that contains the start line.
    ///
    /// # Returns
    ///
    /// The parsed HTTP start line, or a default value if parsing fails.
    pub fn parse_or_default<T: AsRef<str>>(line: T) -> Self {
        Self::parse(line).unwrap_or_else(|_| Default::default())
    }

    /// Parses a string specifically as an HTTP request start line, or a default request on failure.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that contains the request start line.
    ///
    /// # Returns
    ///
    /// The parsed HTTP request start line, or a default request if parsing fails.
    pub fn parse_request_or_default<T: AsRef<str>>(line: T) -> Self {
        Self::parse_request(line).unwrap_or_else(|_| {
            // Default to a GET request to "/"
            Self::Request(Default::default())
        })
    }

    /// Parses a string specifically as an HTTP response start line, or a default response on failure.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that contains the response start line.
    ///
    /// # Returns
    ///
    /// The parsed HTTP response start line, or a default response if parsing fails.
    pub fn parse_response_or_default<T: AsRef<str>>(line: T) -> Self {
        Self::parse_response(line).unwrap_or_else(|_| {
            // Default to HTTP/1.1 200 OK
            Self::Response(Default::default())
        })
    }

    /// Gets the HTTP version regardless of the variant.
    ///
    /// # Returns
    ///
    /// The HTTP version.
    pub fn http_version(&self) -> &HttpVersion {
        match self {
            Self::Request(req) => &req.http_version,
            Self::Response(res) => &res.http_version,
        }
    }

    /// Gets a mutable reference to the HTTP version regardless of the variant.
    ///
    /// # Returns
    ///
    /// A mutable reference to the HTTP version.
    pub fn http_version_mut(&mut self) -> &mut HttpVersion {
        match self {
            Self::Request(req) => &mut req.http_version,
            Self::Response(res) => &mut res.http_version,
        }
    }

    /// Checks if this start line is for an HTTP request.
    ///
    /// # Returns
    ///
    /// `true` if this is a request start line, `false` otherwise.
    pub fn is_request(&self) -> bool {
        matches!(self, Self::Request(_))
    }

    /// Checks if this start line is for an HTTP response.
    ///
    /// # Returns
    ///
    /// `true` if this is a response start line, `false` otherwise.
    pub fn is_response(&self) -> bool {
        matches!(self, Self::Response(_))
    }

    /// Attempts to get a reference to the request start line if this is a request.
    ///
    /// # Returns
    ///
    /// * `Some(&RequestStartLine)` - If this is a request start line.
    /// * `None` - If this is a response start line.
    pub fn try_as_request(&self) -> Option<&RequestStartLine> {
        match self {
            Self::Request(req) => Some(req),
            Self::Response(_) => None,
        }
    }

    /// Gets a reference to the request start line.
    ///
    /// # Returns
    ///
    /// A reference to RequestStartLine. If this is a response, returns a reference to a default RequestStartLine.
    pub fn as_request(&self) -> &RequestStartLine {
        static DEFAULT_REQUEST: std::sync::OnceLock<RequestStartLine> = std::sync::OnceLock::new();
        self.try_as_request()
            .unwrap_or_else(|| DEFAULT_REQUEST.get_or_init(|| RequestStartLine::default()))
    }

    /// Gets a reference to the request start line or a provided default.
    ///
    /// # Arguments
    ///
    /// * `default` - The default RequestStartLine to return if this is a response.
    ///
    /// # Returns
    ///
    /// A reference to RequestStartLine or the provided default.
    pub fn as_request_or<'a>(&'a self, default: &'a RequestStartLine) -> &'a RequestStartLine {
        self.try_as_request().unwrap_or(default)
    }

    /// Attempts to get a mutable reference to the request start line if this is a request.
    ///
    /// # Returns
    ///
    /// * `Some(&mut RequestStartLine)` - If this is a request start line.
    /// * `None` - If this is a response start line.
    pub fn try_as_request_mut(&mut self) -> Option<&mut RequestStartLine> {
        match self {
            Self::Request(req) => Some(req),
            Self::Response(_) => None,
        }
    }

    /// Gets a mutable reference to the request start line.
    ///
    /// # Returns
    ///
    /// If this is a request, returns a mutable reference to the RequestStartLine.
    /// If this is a response, converts it to a request with default values and returns a reference to that.
    pub fn as_request_mut(&mut self) -> &mut RequestStartLine {
        if let Self::Response(_) = self {
            *self = Self::Request(Default::default());
        }

        match self {
            Self::Request(req) => req,
            _ => unreachable!(), // We just ensured this is a request
        }
    }

    /// Attempts to get a reference to the response start line if this is a response.
    ///
    /// # Returns
    ///
    /// * `Some(&ResponseStartLine)` - If this is a response start line.
    /// * `None` - If this is a request start line.
    pub fn try_as_response(&self) -> Option<&ResponseStartLine> {
        match self {
            Self::Request(_) => None,
            Self::Response(res) => Some(res),
        }
    }

    /// Gets a reference to the response start line.
    ///
    /// # Returns
    ///
    /// A reference to ResponseStartLine. If this is a request, returns a reference to a default ResponseStartLine.
    pub fn as_response(&self) -> &ResponseStartLine {
        static DEFAULT_RESPONSE: std::sync::OnceLock<ResponseStartLine> =
            std::sync::OnceLock::new();
        self.try_as_response()
            .unwrap_or_else(|| DEFAULT_RESPONSE.get_or_init(|| ResponseStartLine::default()))
    }

    /// Gets a reference to the response start line or a provided default.
    ///
    /// # Arguments
    ///
    /// * `default` - The default ResponseStartLine to return if this is a request.
    ///
    /// # Returns
    ///
    /// A reference to ResponseStartLine or the provided default.
    pub fn as_response_or<'a>(&'a self, default: &'a ResponseStartLine) -> &'a ResponseStartLine {
        self.try_as_response().unwrap_or(default)
    }

    /// Attempts to get a mutable reference to the response start line if this is a response.
    ///
    /// # Returns
    ///
    /// * `Some(&mut ResponseStartLine)` - If this is a response start line.
    /// * `None` - If this is a request start line.
    pub fn try_as_response_mut(&mut self) -> Option<&mut ResponseStartLine> {
        match self {
            Self::Request(_) => None,
            Self::Response(res) => Some(res),
        }
    }

    /// Gets a mutable reference to the response start line.
    ///
    /// # Returns
    ///
    /// If this is a response, returns a mutable reference to the ResponseStartLine.
    /// If this is a request, converts it to a response with default values and returns a reference to that.
    pub fn as_response_mut(&mut self) -> &mut ResponseStartLine {
        if let Self::Request(_) = self {
            *self = Self::Response(Default::default());
        }

        match self {
            Self::Response(res) => res,
            _ => unreachable!(), // We just ensured this is a response
        }
    }

    /// Attempts to get the HTTP method if this is a request.
    ///
    /// # Returns
    ///
    /// * `Some(&HttpMethod)` - If this is a request start line.
    /// * `None` - If this is a response start line.
    pub fn try_method(&self) -> Option<&HttpMethod> {
        self.try_as_request().map(|req| &req.method)
    }

    /// Gets the HTTP method.
    ///
    /// # Returns
    ///
    /// The HTTP method. If this is a response, returns a default HTTP method (GET).
    pub fn method(&self) -> HttpMethod {
        self.try_method().cloned().unwrap_or_default()
    }

    /// Gets the HTTP method, or a provided default if this is a response.
    ///
    /// # Arguments
    ///
    /// * `default` - The default HttpMethod to return if this is a response.
    ///
    /// # Returns
    ///
    /// The HTTP method or the provided default.
    pub fn method_or(&self, default: HttpMethod) -> HttpMethod {
        self.try_method().cloned().unwrap_or(default)
    }

    /// Attempts to get a mutable reference to the HTTP method if this is a request.
    ///
    /// # Returns
    ///
    /// * `Some(&mut HttpMethod)` - If this is a request start line.
    /// * `None` - If this is a response start line.
    pub fn try_method_mut(&mut self) -> Option<&mut HttpMethod> {
        self.try_as_request_mut().map(|req| &mut req.method)
    }

    /// Gets a mutable reference to the HTTP method.
    ///
    /// # Returns
    ///
    /// A mutable reference to the HTTP method. If this is a response,
    /// converts it to a request with default values and returns the method.
    pub fn method_mut(&mut self) -> &mut HttpMethod {
        &mut self.as_request_mut().method
    }

    /// Attempts to get the path if this is a request.
    ///
    /// # Returns
    ///
    /// * `Some(&str)` - If this is a request start line.
    /// * `None` - If this is a response start line.
    pub fn try_path(&self) -> Option<&str> {
        self.try_as_request().map(|req| req.path.as_str())
    }

    /// Gets the request path.
    ///
    /// # Returns
    ///
    /// The request path. If this is a response, returns a default path ("/").
    pub fn path(&self) -> String {
        self.try_path()
            .map(String::from)
            .unwrap_or_else(|| "/".to_string())
    }

    /// Gets the request path, or a provided default if this is a response.
    ///
    /// # Arguments
    ///
    /// * `default` - The default path to return if this is a response.
    ///
    /// # Returns
    ///
    /// The request path or the provided default.
    pub fn path_or<T: AsRef<str>>(&self, default: T) -> String {
        self.try_path()
            .map(String::from)
            .unwrap_or_else(|| default.as_ref().to_string())
    }

    /// Attempts to get a mutable reference to the path if this is a request.
    ///
    /// # Returns
    ///
    /// * `Some(&mut String)` - If this is a request start line.
    /// * `None` - If this is a response start line.
    pub fn try_path_mut(&mut self) -> Option<&mut String> {
        self.try_as_request_mut().map(|req| &mut req.path)
    }

    /// Gets a mutable reference to the request path.
    ///
    /// # Returns
    ///
    /// A mutable reference to the request path. If this is a response,
    /// converts it to a request with default values and returns the path.
    pub fn path_mut(&mut self) -> &mut String {
        &mut self.as_request_mut().path
    }

    /// Attempts to get the parsed URL if this is a request.
    ///
    /// # Returns
    ///
    /// * `Some(&RequestPath)` - If this is a request start line with a parsed URL.
    /// * `None` - If this is a response start line or the URL hasn't been parsed.
    pub fn try_url(&self) -> Option<&RequestPath> {
        self.try_as_request().and_then(|req| req.url.as_ref())
    }

    /// Gets the parsed URL.
    ///
    /// # Returns
    ///
    /// The parsed URL. If this is a response or URL hasn't been parsed, returns a default URL.
    pub fn url(&self) -> RequestPath {
        match self.try_url() {
            Some(url) => url.clone(),
            None => RequestPath::default(),
        }
    }

    /// Gets the parsed URL, or a provided default.
    ///
    /// # Arguments
    ///
    /// * `default` - The default URL to return if this is a response or URL hasn't been parsed.
    ///
    /// # Returns
    ///
    /// The parsed URL or the provided default.
    pub fn url_or(&self, default: RequestPath) -> RequestPath {
        self.try_url().cloned().unwrap_or(default)
    }

    /// Gets or parses the URL if this is a request.
    ///
    /// If the URL hasn't been parsed yet, this will parse it first.
    ///
    /// # Returns
    ///
    /// The parsed RequestPath. If this is a response, returns a default RequestPath.
    pub fn get_url(&mut self) -> RequestPath {
        match self.try_get_url() {
            Some(url) => url,
            None => RequestPath::default(),
        }
    }

    /// Attempts to get or parse the URL if this is a request.
    ///
    /// # Returns
    ///
    /// * `Some(RequestPath)` - If this is a request start line.
    /// * `None` - If this is a response start line.
    pub fn try_get_url(&mut self) -> Option<RequestPath> {
        match self {
            Self::Request(req) => Some(req.get_url()),
            Self::Response(_) => None,
        }
    }

    /// Parses the URL if this is a request.
    ///
    /// # Returns
    ///
    /// The parsed RequestPath. If this is a response, returns a default RequestPath.
    pub fn parse_url(&mut self) -> RequestPath {
        self.try_parse_url().unwrap_or_default()
    }

    /// Attempts to parse the URL if this is a request.
    ///
    /// # Returns
    ///
    /// * `Some(RequestPath)` - If this is a request start line.
    /// * `None` - If this is a response start line.
    pub fn try_parse_url(&mut self) -> Option<RequestPath> {
        match self {
            Self::Request(req) => Some(req.parse_url()),
            Self::Response(_) => None,
        }
    }

    /// Attempts to get the status code if this is a response.
    ///
    /// # Returns
    ///
    /// * `Some(&StatusCode)` - If this is a response start line.
    /// * `None` - If this is a request start line.
    pub fn try_status_code(&self) -> Option<&StatusCode> {
        self.try_as_response().map(|res| &res.status_code)
    }

    /// Gets the status code.
    ///
    /// # Returns
    ///
    /// The status code. If this is a request, returns a default status code (200 OK).
    pub fn status_code(&self) -> StatusCode {
        self.try_status_code().cloned().unwrap_or(StatusCode::OK)
    }

    /// Gets the status code, or a provided default if this is a request.
    ///
    /// # Arguments
    ///
    /// * `default` - The default StatusCode to return if this is a request.
    ///
    /// # Returns
    ///
    /// The status code or the provided default.
    pub fn status_code_or(&self, default: StatusCode) -> StatusCode {
        self.try_status_code().cloned().unwrap_or(default)
    }

    /// Attempts to get a mutable reference to the status code if this is a response.
    ///
    /// # Returns
    ///
    /// * `Some(&mut StatusCode)` - If this is a response start line.
    /// * `None` - If this is a request start line.
    pub fn try_status_code_mut(&mut self) -> Option<&mut StatusCode> {
        self.try_as_response_mut().map(|res| &mut res.status_code)
    }

    /// Gets a mutable reference to the status code.
    ///
    /// # Returns
    ///
    /// A mutable reference to the status code. If this is a request,
    /// converts it to a response with default values and returns the status code.
    pub fn status_code_mut(&mut self) -> &mut StatusCode {
        &mut self.as_response_mut().status_code
    }

    /// Sets the status code of the HTTP start line.
    ///
    /// If this start line represents a request, it will be converted to a response
    /// with the specified status code and default HTTP version (HTTP/1.1).
    ///
    /// # Arguments
    ///
    /// * `status` - The new status code. Can be any type that implements `Into<StatusCode>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::{start_line::HttpStartLine, http_value::{HttpMethod, HttpVersion, StatusCode}};
    ///
    /// // Setting status on a response
    /// let mut response = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
    /// response.set_status_code(StatusCode::NOT_FOUND);
    ///
    /// // Setting status on a request converts it to a response
    /// let mut request = HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, "/".into());
    /// request.set_status_code(404);  // Converts to response with 404 status
    /// ```
    pub fn set_status_code<T: Into<StatusCode>>(&mut self, status: T) {
        if let Self::Response(res) = self {
            res.status_code = status.into();
        } else {
            *self = Self::Response(ResponseStartLine::new(HttpVersion::Http11, status.into()));
        }
    }

    /// Sets the HTTP version of the start line.
    ///
    /// This method works for both request and response start lines.
    ///
    /// # Arguments
    ///
    /// * `version` - The new HTTP version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::{start_line::HttpStartLine, http_value::{HttpMethod, HttpVersion, StatusCode}};
    ///
    /// // Setting version on a request
    /// let mut request = HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, "/".into());
    /// request.set_http_version(HttpVersion::Http20);
    ///
    /// // Setting version on a response
    /// let mut response = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
    /// response.set_http_version(HttpVersion::Http30);
    /// ```
    pub fn set_http_version(&mut self, version: HttpVersion) {
        match self {
            Self::Request(req) => req.http_version = version,
            Self::Response(res) => res.http_version = version,
        }
    }

    /// Sets the request path of the HTTP start line.
    ///
    /// If this start line represents a response, it will be converted to a request
    /// with the specified path and default method (GET) and HTTP version (HTTP/1.1).
    ///
    /// # Arguments
    ///
    /// * `path` - The new request path. Can be any type that implements `Into<String>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::{start_line::HttpStartLine, http_value::{HttpMethod, HttpVersion, StatusCode}};
    ///
    /// // Setting path on a request
    /// let mut request = HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, "/".into());
    /// request.set_path("/new-path");
    ///
    /// // Setting path on a response converts it to a request
    /// let mut response = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
    /// response.set_path("/index.html");  // Converts to request with GET /index.html
    /// ```
    pub fn set_path<T: Into<String>>(&mut self, path: T) {
        if let Self::Request(req) = self {
            req.path = path.into();
            req.clear_url(); // Clear cached URL when path changes
        } else {
            *self = Self::Request(RequestStartLine::new(
                HttpVersion::Http11,
                HttpMethod::GET,
                path.into(),
            ));
        }
    }

    /// Sets the HTTP method of the start line.
    ///
    /// If this start line represents a response, it will be converted to a request
    /// with the specified method and default path ("/") and HTTP version (HTTP/1.1).
    ///
    /// # Arguments
    ///
    /// * `method` - The new HTTP method.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::{start_line::HttpStartLine, http_value::{HttpMethod, HttpVersion, StatusCode}};
    ///
    /// // Setting method on a request
    /// let mut request = HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, "/".into());
    /// request.set_method(HttpMethod::POST);
    ///
    /// // Setting method on a response converts it to a request
    /// let mut response = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
    /// response.set_method(HttpMethod::PUT);  // Converts to request with PUT /
    /// ```
    pub fn set_method(&mut self, method: HttpMethod) {
        if let Self::Request(req) = self {
            req.method = method;
        } else {
            *self = Self::Request(RequestStartLine::new(
                HttpVersion::Http11,
                method,
                "/".to_string(),
            ));
        }
    }

    /// Creates a new HTTP POST request start line.
    ///
    /// # Arguments
    ///
    /// * `url` - The request path. This can be any type that can be converted into a `String`.
    ///
    /// # Returns
    ///
    /// A new `HttpStartLine::Request` variant with the POST method and the given path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::{start_line::HttpStartLine, http_value::{HttpMethod, HttpVersion, StatusCode}};
    /// let start_line = HttpStartLine::request_post("/submit");
    /// ```
    pub fn request_post<T: Into<String>>(url: T) -> Self {
        Self::Request(RequestStartLine::new(
            HttpVersion::Http11,
            HttpMethod::POST,
            url.into(),
        ))
    }

    /// Creates a new HTTP GET request start line.
    ///
    /// # Arguments
    ///
    /// * `url` - The request path. This can be any type that can be converted into a `String`.
    ///
    /// # Returns
    ///
    /// A new `HttpStartLine::Request` variant with the GET method and the given path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::{start_line::HttpStartLine, http_value::{HttpMethod, HttpVersion, StatusCode}};
    /// let start_line = HttpStartLine::request_get("/index.html");
    /// ```
    pub fn request_get<T: Into<String>>(url: T) -> Self {
        Self::Request(RequestStartLine::new(
            HttpVersion::Http11,
            HttpMethod::GET,
            url.into(),
        ))
    }

    /// Creates a new HTTP response start line with a custom status code.
    ///
    /// # Arguments
    ///
    /// * `status` - The response status code. This can be any type that can be converted into a `StatusCode`.
    ///
    /// # Returns
    ///
    /// A new `HttpStartLine::Response` variant with the given status code and HTTP/1.1 version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::{start_line::HttpStartLine, http_value::{HttpMethod, HttpVersion, StatusCode}};
    /// let start_line = HttpStartLine::response(StatusCode::NOT_FOUND);
    /// // or using an integer:
    /// let start_line = HttpStartLine::response(404);
    /// ```
    pub fn response<T: Into<StatusCode>>(status: T) -> Self {
        Self::Response(ResponseStartLine::new(HttpVersion::Http11, status.into()))
    }

    /// Tries to unwrap this start line as a request.
    ///
    /// # Returns
    ///
    /// * `Ok(RequestStartLine)` - If this is a request start line.
    /// * `Err(Self)` - If this is a response start line, returns self.
    pub fn try_into_request(self) -> Result<RequestStartLine, Self> {
        match self {
            Self::Request(req) => Ok(req),
            _ => Err(self),
        }
    }

    /// Tries to unwrap this start line as a response.
    ///
    /// # Returns
    ///
    /// * `Ok(ResponseStartLine)` - If this is a response start line.
    /// * `Err(Self)` - If this is a request start line, returns self.
    pub fn try_into_response(self) -> Result<ResponseStartLine, Self> {
        match self {
            Self::Response(res) => Ok(res),
            _ => Err(self),
        }
    }

    /// Converts this start line to a request.
    ///
    /// # Returns
    ///
    /// If this is a request, returns the inner RequestStartLine.
    /// If this is a response, returns a default RequestStartLine.
    pub fn into_request(self) -> RequestStartLine {
        match self {
            Self::Request(req) => req,
            Self::Response(_) => Default::default(),
        }
    }

    /// Converts this start line to a response.
    ///
    /// # Returns
    ///
    /// If this is a response, returns the inner ResponseStartLine.
    /// If this is a request, returns a default ResponseStartLine.
    pub fn into_response(self) -> ResponseStartLine {
        match self {
            Self::Response(res) => res,
            Self::Request(_) => Default::default(),
        }
    }

    /// Attempts to check if this is a successful response (status code 2xx).
    ///
    /// # Returns
    ///
    /// * `Some(bool)` - If this is a response start line.
    /// * `None` - If this is a request start line.
    pub fn try_is_success(&self) -> Option<bool> {
        self.try_status_code().map(|code| code.is_success())
    }

    /// Checks if this is a successful response (status code 2xx).
    ///
    /// # Returns
    ///
    /// `true` if this is a response with a 2xx status code,
    /// `false` if this is either a request or a response with a non-2xx status code.
    pub fn is_success(&self) -> bool {
        self.try_is_success().unwrap_or(false)
    }

    /// Attempts to check if this is a client error response (status code 4xx).
    ///
    /// # Returns
    ///
    /// * `Some(bool)` - If this is a response start line.
    /// * `None` - If this is a request start line.
    pub fn try_is_client_error(&self) -> Option<bool> {
        self.try_status_code().map(|code| code.is_client_error())
    }

    /// Checks if this is a client error response (status code 4xx).
    ///
    /// # Returns
    ///
    /// `true` if this is a response with a 4xx status code,
    /// `false` if this is either a request or a response with a non-4xx status code.
    pub fn is_client_error(&self) -> bool {
        self.try_is_client_error().unwrap_or(false)
    }

    /// Attempts to check if this is a server error response (status code 5xx).
    ///
    /// # Returns
    ///
    /// * `Some(bool)` - If this is a response start line.
    /// * `None` - If this is a request start line.
    pub fn try_is_server_error(&self) -> Option<bool> {
        self.try_status_code().map(|code| code.is_server_error())
    }

    /// Checks if this is a server error response (status code 5xx).
    ///
    /// # Returns
    ///
    /// `true` if this is a response with a 5xx status code,
    /// `false` if this is either a request or a response with a non-5xx status code.
    pub fn is_server_error(&self) -> bool {
        self.try_is_server_error().unwrap_or(false)
    }

    /// Converts this start line to a string representation.
    ///
    /// # Returns
    ///
    /// A string representation of the start line.
    pub fn represent(&self) -> String {
        match self {
            Self::Request(req) => req.represent(),
            Self::Response(res) => res.represent(),
        }
    }
}

impl std::fmt::Display for HttpStartLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(req) => write!(f, "{}", req),
            Self::Response(res) => write!(f, "{}", res),
        }
    }
}

impl From<RequestStartLine> for HttpStartLine {
    fn from(request: RequestStartLine) -> Self {
        Self::Request(request)
    }
}

impl From<ResponseStartLine> for HttpStartLine {
    fn from(response: ResponseStartLine) -> Self {
        Self::Response(response)
    }
}

impl Default for HttpStartLine {
    fn default() -> Self {
        // Default to an HTTP/1.1 GET request for "/"
        Self::Request(Default::default())
    }
}
