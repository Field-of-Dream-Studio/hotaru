use std::collections::HashMap;
use hotaru_lib::url_encoding::decode_form_url_owned;

use super::error::UrlEncodedError;
use super::form::UrlEncodedForm;

/// Parses a URL-encoded form byte slice into a [`UrlEncodedForm`].
///
/// # Errors
///
/// Returns [`UrlEncodedError::InvalidUtf8`] if the bytes are not valid UTF-8,
/// or [`UrlEncodedError::MalformedPair`] if any key-value pair is malformed.
pub fn parse_urlencoded(body: impl AsRef<[u8]>) -> Result<UrlEncodedForm, UrlEncodedError> {
    let body = body.as_ref();
    if body.is_empty() {
        return Ok(UrlEncodedForm::new());
    }

    let form_data = std::str::from_utf8(body).map_err(|_| UrlEncodedError::InvalidUtf8)?;
    let mut form_map = HashMap::new();

    for pair in form_data.split('&') {
        if pair.is_empty() {
            return Err(UrlEncodedError::MalformedPair(
                "empty pair found in form data".to_string(),
            ));
        }

        if let Some((key, value)) = pair.split_once('=') {
            form_map.insert(
                decode_form_url_owned(key),
                decode_form_url_owned(value),
            );
        } else {
            return Err(UrlEncodedError::MalformedPair(format!(
                "missing '=' in key-value pair: '{pair}'"
            )));
        }
    }

    Ok(UrlEncodedForm { data: form_map })
}
