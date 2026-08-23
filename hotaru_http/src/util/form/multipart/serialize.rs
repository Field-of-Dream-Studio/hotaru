use crate::message::http_value::ContentDisposition;

use super::{MultiForm, MultiFormField};

/// Serializes a [`MultiForm`] into a multipart string representation using the specified boundary.
pub fn serialize_multipart(form: &MultiForm, boundary: &str) -> String {
    let mut form_data = String::new();

    for (key, field) in &form.data {
        form_data.push_str(&format!("--{}\r\n", boundary));

        match field {
            MultiFormField::Text(value) => {
                let disposition = ContentDisposition::form_data::<_, String>(key, None);
                form_data.push_str(&format!(
                    "Content-Disposition: {}\r\n\r\n{}\r\n",
                    disposition.to_string(),
                    value
                ));
            }
            MultiFormField::File(files) => {
                for file in files {
                    let disposition = ContentDisposition::form_data(
                        key,
                        file.filename.as_ref().map(|f| f.to_string()),
                    );

                    form_data.push_str(&format!(
                        "Content-Disposition: {}\r\n",
                        disposition.to_string()
                    ));

                    if let Some(content_type) = &file.content_type {
                        form_data.push_str(&format!("Content-Type: {}\r\n", content_type));
                    }

                    form_data.push_str("\r\n");

                    if let Ok(data_str) = std::str::from_utf8(&file.data) {
                        form_data.push_str(data_str);
                    } else {
                        form_data.push_str("[binary data]");
                    }

                    form_data.push_str("\r\n");
                }
            }
        }
    }

    form_data.push_str(&format!("--{}--\r\n", boundary));
    form_data
}
