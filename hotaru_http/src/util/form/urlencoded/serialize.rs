use hotaru_lib::url_encoding::encode_url_owned;

use super::form::UrlEncodedForm;

/// Serializes a [`UrlEncodedForm`] into an `application/x-www-form-urlencoded` string.
pub fn serialize_urlencoded(form: &UrlEncodedForm) -> String {
    let mut form_data = String::new();
    for (key, value) in &form.data {
        if !form_data.is_empty() {
            form_data.push('&');
        }
        form_data.push_str(&format!(
            "{}={}",
            encode_url_owned(key),
            encode_url_owned(value)
        ));
    }
    form_data
}
