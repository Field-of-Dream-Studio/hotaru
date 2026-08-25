use core::str::FromStr;

use super::ConnectionError;

/// One case-insensitive connection-option from the HTTP `Connection` header.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConnectionToken {
    Close,
    KeepAlive,
    Upgrade,
    Other(Box<str>),
}

impl ConnectionToken {
    /// Returns the normalized lowercase representation of this token.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Close => "close",
            Self::KeepAlive => "keep-alive",
            Self::Upgrade => "upgrade",
            Self::Other(token) => token,
        }
    }
}

impl FromStr for ConnectionToken {
    type Err = ConnectionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let token = value.trim();
        if token.is_empty() {
            return Err(ConnectionError::EmptyToken);
        }
        if !token.bytes().all(is_token_byte) {
            return Err(ConnectionError::InvalidToken(token.to_string()));
        }

        let token = token.to_ascii_lowercase();
        Ok(match token.as_str() {
            "close" => Self::Close,
            "keep-alive" => Self::KeepAlive,
            "upgrade" => Self::Upgrade,
            _ => Self::Other(token.into_boxed_str()),
        })
    }
}

impl fmt::Display for ConnectionToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

use core::fmt;

fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}
