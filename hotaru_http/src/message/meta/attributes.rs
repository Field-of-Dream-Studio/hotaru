use super::{HeaderValue, HttpMeta};
use crate::message::http_value::HttpMethod;
use std::collections::HashMap;

impl HttpMeta {
    pub fn set_header_hashmap(&mut self, header: HashMap<String, HeaderValue>) {
        self.header = header;
    }

    /// Returns the hashed, unparsed header.
    /// Note this reference is not intended for you to mutate.
    /// If yo do want to mutate, please use .set_attribute() method
    pub fn get_header_hashmap(&self) -> &HashMap<String, HeaderValue> {
        &self.header
    }

    pub fn get_header<T: Into<String>>(&self, key: T) -> Option<String> {
        self.header
            .get(&key.into().trim().to_lowercase())
            .and_then(|v| Some(v.as_str()))
    }

    ///
    pub fn set_attribute<T: Into<String>, S: Into<HeaderValue>>(&mut self, key: T, value: S) {
        self.header
            .insert(key.into().trim().to_lowercase(), value.into());
    }

    pub fn get_path(&mut self, part: usize) -> String {
        self.start_line.get_url().url_part(part)
    }

    pub fn url(&self) -> String {
        self.start_line.path()
    }

    pub fn path(&self) -> String {
        // Return the path part of the URL, removing the query string if present
        self.start_line
            .path()
            .split('?')
            .next()
            .unwrap_or("")
            .to_string()
    }

    pub fn get_url_args<T: Into<String>>(&mut self, key: T) -> Option<String> {
        self.start_line.get_url().get_url_args(&key.into())
    }

    pub fn method(&self) -> HttpMethod {
        self.start_line.method()
    }
}
