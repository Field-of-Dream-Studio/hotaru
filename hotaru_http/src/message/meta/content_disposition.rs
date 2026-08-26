use super::HttpMeta;
use crate::message::http_value::ContentDisposition;

impl HttpMeta {
    /// Gets the Content-Disposition header value from the HTTP metadata.
    ///
    /// This method returns the cached Content-Disposition value if available.
    /// If not cached, it parses the "Content-Disposition" header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<ContentDisposition>` - The parsed Content-Disposition value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::http_value::ContentDisposition;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "content-disposition".to_string(),
    ///     HeaderValue::new("attachment; filename=\"report.pdf\"")
    /// );
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let content_disp = meta.get_content_disposition();
    /// assert!(content_disp.is_some());
    /// assert_eq!(content_disp.unwrap().filename().unwrap(), "report.pdf");
    /// ```
    pub fn get_content_disposition(&mut self) -> Option<ContentDisposition> {
        if let Some(ref content_disposition) = self.content_disposition {
            return Some(content_disposition.clone());
        }
        self.parse_content_disposition()
    }

    /// Parses the Content-Disposition header from the headers map and stores it in the content_disposition field.
    ///
    /// # Returns
    ///
    /// * `Option<ContentDisposition>` - The parsed Content-Disposition value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::http_value::ContentDisposition;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "content-disposition".to_string(),
    ///     HeaderValue::new("form-data; name=\"file\"; filename=\"data.txt\"")
    /// );
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let content_disp = meta.parse_content_disposition();
    /// assert!(content_disp.is_some());
    /// assert_eq!(content_disp.unwrap().filename().unwrap(), "data.txt");
    /// ```
    pub fn parse_content_disposition(&mut self) -> Option<ContentDisposition> {
        let content_disposition = self
            .header
            .get("content-disposition")
            .and_then(|s| ContentDisposition::parse(&s.first()).ok());

        if let Some(ref cd) = content_disposition {
            self.content_disposition = Some(cd.clone());
        }
        content_disposition
    }

    /// Sets the content_disposition field.
    ///
    /// # Arguments
    ///
    /// * `content_disposition` - The Content-Disposition value to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::http_value::{ContentDisposition, ContentDispositionType};
    ///
    /// let mut meta = HttpMeta::default();
    /// let cd = ContentDisposition::attachment("report.pdf");
    /// meta.set_content_disposition(cd.clone());
    ///
    /// assert_eq!(meta.get_content_disposition(), Some(cd));
    /// ```
    pub fn set_content_disposition(&mut self, content_disposition: ContentDisposition) {
        self.content_disposition = Some(content_disposition);
    }

    /// Clears the cached content_disposition field without modifying the header map.
    ///
    /// This method invalidates the cached Content-Disposition value, which will cause
    /// subsequent calls to `get_content_disposition()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_content_disposition()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::http_value::ContentDisposition;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "content-disposition".to_string(),
    ///     HeaderValue::new("inline; filename=\"image.jpg\"")
    /// );
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Parse the value into the cache
    /// let content_disp = meta.get_content_disposition();
    /// assert!(content_disp.is_some());
    ///
    /// // Clear the cache only
    /// meta.clear_content_disposition();
    ///
    /// // The header is still intact and will be re-parsed
    /// assert!(meta.get_content_disposition().is_some());
    /// ```
    pub fn clear_content_disposition(&mut self) {
        self.content_disposition = None;
    }

    /// Deletes the Content-Disposition header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the content-disposition header from the headers map and
    /// clears the cached content_disposition value. Subsequent calls to `get_content_disposition()`
    /// will return None unless a new header is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::http_value::ContentDisposition;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "content-disposition".to_string(),
    ///     HeaderValue::new("attachment; filename=\"data.zip\"")
    /// );
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Delete both the cache and header
    /// meta.delete_content_disposition();
    ///
    /// // The header is gone
    /// assert!(meta.get_header("content-disposition").is_none());
    ///
    /// // And get_content_disposition will now return None
    /// assert!(meta.get_content_disposition().is_none());
    /// ```
    pub fn delete_content_disposition(&mut self) {
        self.content_disposition = None;
        self.header.remove("content-disposition");
    }
}
