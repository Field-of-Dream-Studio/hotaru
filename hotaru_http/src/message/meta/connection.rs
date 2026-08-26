use super::HttpMeta;
use crate::util::connection::{ConnectionError, ConnectionOptions};

impl HttpMeta {
    /// Returns cached connection options, or parses the current header values.
    pub fn get_connection(&self) -> Result<ConnectionOptions, ConnectionError> {
        match &self.connection {
            Some(connection) => Ok(connection.clone()),
            None => ConnectionOptions::from_headers(&self.header),
        }
    }

    /// Whether this HTTP/1 message permits the connection to remain open.
    ///
    /// Malformed Connection values fail closed. Otherwise, persistence is
    /// derived from the parsed options and the start-line HTTP version.
    pub fn is_keep_alive(&self) -> bool {
        let Ok(connection) = self.get_connection() else {
            return false;
        };

        connection.is_keep_alive(self.start_line.http_version())
    }

    /// Parses the current `Connection` header values and caches the result.
    pub fn parse_connection(&mut self) -> Result<ConnectionOptions, ConnectionError> {
        let connection = ConnectionOptions::from_headers(&self.header)?;
        self.connection = Some(connection.clone());
        Ok(connection)
    }

    /// Replaces the typed Connection header cache.
    pub fn set_connection(&mut self, connection: Option<ConnectionOptions>) {
        self.connection = connection;
    }

    /// Clears the typed cache without removing the raw Connection header.
    pub fn clear_connection(&mut self) {
        self.connection = None;
    }

    /// Removes both the typed cache and the raw Connection header.
    pub fn delete_connection(&mut self) {
        self.connection = None;
        self.header.remove("connection");
    }
}
