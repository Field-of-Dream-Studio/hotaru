//! HTTP content types and value reconciliation.

#![allow(non_snake_case)]

/// Represents the content type of an HTTP message.
/// This enum is used to parse and construct HTTP headers related to content type.
/// It includes well-known content types like text, application, image, audio, video, and multipart.
/// It also includes a generic Other variant for any other content types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpContentType {
    // Well-known content types
    Text {
        subtype: String,
        charset: Option<String>,
    },
    Application {
        subtype: String,
        parameters: Option<Vec<(String, String)>>,
    },
    Image {
        subtype: String,
    },
    Audio {
        subtype: String,
    },
    Video {
        subtype: String,
    },
    Model {
        subtype: String,
    },
    Multipart {
        subtype: String,
        boundary: Option<String>,
    },
    Other {
        type_name: String,
        subtype: String,
        parameters: Option<Vec<(String, String)>>,
    },
}

impl HttpContentType {
    /// Converts a string into an HttpContentType enum variant
    /// This function parses the content type string and extracts the main type, subtype, and any parameters.
    /// It supports well-known content types like text, application, image, audio, video, and multipart. ]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::util::content_type::HttpContentType;
    /// let content_type = HttpContentType::from_str("text/html; charset=UTF-8");
    /// assert_eq!(content_type, HttpContentType::Text { subtype: "html".to_string(), charset: Some("UTF-8".to_string()) });
    /// ```
    pub fn from_str(content_type: &str) -> Self {
        let parts: Vec<&str> = content_type.split(';').collect();
        let main_part = parts[0].trim();
        let mut parameters = Vec::new();

        for part in &parts[1..] {
            let param_parts: Vec<&str> = part.split('=').collect();
            if param_parts.len() == 2 {
                parameters.push((
                    param_parts[0].trim().to_string(),
                    param_parts[1].trim().to_string(),
                ));
            }
        }

        let (type_name, subtype) = if let Some(pos) = main_part.find('/') {
            (&main_part[..pos], &main_part[pos + 1..])
        } else {
            ("unknown", "unknown")
        };

        match type_name {
            "text" => Self::Text {
                subtype: subtype.to_string(),
                charset: Self::find_value_from_vec(&parameters, "charset"),
            },
            "application" => Self::Application {
                subtype: subtype.to_string(),
                parameters: Some(parameters),
            },
            "image" => Self::Image {
                subtype: subtype.to_string(),
            },
            "audio" => Self::Audio {
                subtype: subtype.to_string(),
            },
            "video" => Self::Video {
                subtype: subtype.to_string(),
            },
            "model" => Self::Model {
                subtype: subtype.to_string(),
            },
            "multipart" => Self::Multipart {
                subtype: subtype.to_string(),
                boundary: Self::find_value_from_vec(&parameters, "boundary"),
            },
            _ => Self::Other {
                type_name: type_name.to_string(),
                subtype: subtype.to_string(),
                parameters: Some(parameters),
            },
        }
    }

    /// Find value from Vec<(String, String)>
    /// # Examples
    /// ```rust
    /// # use hotaru_http::util::content_type::HttpContentType;
    /// let vec = vec![("key1".to_string(), "value1".to_string()), ("key2".to_string(), "value2".to_string())];
    /// let value = HttpContentType::find_value_from_vec(&vec, "key1");
    /// assert_eq!(value, Some("value1".to_string()));
    /// ```
    pub fn find_value_from_vec(vec: &Vec<(String, String)>, key: &str) -> Option<String> {
        for (k, v) in vec {
            if k == key {
                return Some(v.clone());
            }
        }
        None
    }

    /// Converts an HttpContentType enum variant into its string representation
    pub fn to_string(&self) -> String {
        match self {
            HttpContentType::Text { subtype, charset } => match charset {
                Some(cs) => format!("text/{}; charset={}", subtype, cs),
                None => format!("text/{}", subtype),
            },
            HttpContentType::Application {
                subtype,
                parameters,
            } => {
                let mut result = format!("application/{}", subtype);
                if let Some(params) = parameters {
                    for (key, value) in params {
                        result.push_str(&format!("; {}={}", key, value));
                    }
                }
                result
            }
            HttpContentType::Image { subtype } => format!("image/{}", subtype),
            HttpContentType::Audio { subtype } => format!("audio/{}", subtype),
            HttpContentType::Video { subtype } => format!("video/{}", subtype),
            HttpContentType::Model { subtype } => format!("model/{}", subtype),
            HttpContentType::Multipart { subtype, boundary } => {
                if let Some(b) = boundary {
                    format!("multipart/{}; boundary={}", subtype, b)
                } else {
                    format!("multipart/{}", subtype)
                }
            }
            HttpContentType::Other {
                type_name,
                subtype,
                parameters,
            } => {
                let mut result = format!("{}/{}", type_name, subtype);
                if let Some(params) = parameters {
                    for (key, value) in params {
                        result.push_str(&format!("; {}={}", key, value));
                    }
                }
                result
            }
        }
    }

    /// Get content type based on file extension
    /// # Examples
    /// ```
    /// # use hotaru_http::util::content_type::HttpContentType;
    ///
    /// let content_type = HttpContentType::from_extension("html");
    /// assert_eq!(content_type, HttpContentType::TextHtml());
    ///
    /// let content_type = HttpContentType::from_extension("svg");
    /// assert_eq!(content_type, HttpContentType::ImageSvg());
    /// ```
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Text types
            "html" | "htm" => Self::TextHtml(),
            "txt" => Self::TextPlain(),
            "css" => Self::TextCss(),
            "csv" => Self::TextCsv(),
            "xml" => Self::TextXml(),

            // Application types
            "js" | "mjs" | "cjs" => Self::ApplicationJavascript(),
            "json" => Self::ApplicationJson(),
            "pdf" => Self::ApplicationPdf(),
            "zip" | "jar" | "war" => Self::ApplicationZip(),
            "wasm" => Self::ApplicationWasm(),
            "bin" => Self::ApplicationOctetStream(),

            // Image types
            "png" => Self::ImagePng(),
            "jpg" | "jpeg" | "jpe" | "jif" | "jfif" => Self::ImageJpeg(),
            "gif" => Self::ImageGif(),
            "svg" | "svgz" => Self::ImageSvg(),
            "webp" => Self::ImageWebp(),
            "bmp" | "dib" => Self::ImageBmp(),
            "ico" => Self::Image {
                subtype: "x-icon".to_string(),
            },
            "tiff" | "tif" => Self::Image {
                subtype: "tiff".to_string(),
            },

            // Audio types
            "mp3" => Self::AudioMpeg(),
            "wav" => Self::Audio {
                subtype: "wav".to_string(),
            },
            "ogg" => Self::Audio {
                subtype: "ogg".to_string(),
            },
            "flac" => Self::Audio {
                subtype: "flac".to_string(),
            },

            // Video types
            "mp4" | "m4v" => Self::VideoMp4(),
            "mov" => Self::Video {
                subtype: "quicktime".to_string(),
            },
            "webm" => Self::Video {
                subtype: "webm".to_string(),
            },
            "avi" => Self::Video {
                subtype: "x-msvideo".to_string(),
            },

            // Model types
            "stl" => Self::ModelStl(),
            "obj" => Self::ModelObj(),
            "glb" => Self::Model {
                subtype: "gltf-binary".to_string(),
            },
            "gltf" => Self::Model {
                subtype: "gltf+json".to_string(),
            },

            // Documents
            "doc" => Self::Application {
                subtype: "msword".to_string(),
                parameters: None,
            },
            "docx" => Self::Application {
                subtype: "vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
                parameters: None,
            },
            "xls" => Self::Application {
                subtype: "vnd.ms-excel".to_string(),
                parameters: None,
            },
            "xlsx" => Self::Application {
                subtype: "vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
                parameters: None,
            },
            "ppt" => Self::Application {
                subtype: "vnd.ms-powerpoint".to_string(),
                parameters: None,
            },
            "pptx" => Self::Application {
                subtype: "vnd.openxmlformats-officedocument.presentationml.presentation"
                    .to_string(),
                parameters: None,
            },

            // Archive formats
            "tar" => Self::Application {
                subtype: "x-tar".to_string(),
                parameters: None,
            },
            "gz" | "tgz" => Self::Application {
                subtype: "gzip".to_string(),
                parameters: None,
            },

            // Special cases
            "form" => Self::ApplicationUrlEncodedForm(),
            // "multipart" => Self::MultipartFormData(),
            _ => Self::ApplicationOctetStream(),
        }
    }

    /// Get content type based on file name
    /// This function extracts the extension from the file name and determines the appropriate content type.
    /// If the file has no extension or an unrecognized extension, it defaults to application/octet-stream.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::util::content_type::HttpContentType;
    ///
    /// let content_type = HttpContentType::from_file_name("index.html");
    /// assert_eq!(content_type, HttpContentType::TextHtml());
    ///
    /// let content_type = HttpContentType::from_file_name("document.pdf");
    /// assert_eq!(content_type, HttpContentType::ApplicationPdf());
    ///
    /// // Works with paths too
    /// let content_type = HttpContentType::from_file_name("/var/www/images/logo.svg");
    /// assert_eq!(content_type, HttpContentType::ImageSvg());
    ///
    /// // Files without extension default to application/octet-stream
    /// let content_type = HttpContentType::from_file_name("README");
    /// assert_eq!(content_type, HttpContentType::ApplicationOctetStream());
    /// ```
    pub fn from_file_name(file_name: &str) -> Self {
        let path = std::path::Path::new(file_name);
        match path.extension() {
            Some(os_str) => match os_str.to_str() {
                Some(ext) => Self::from_extension(ext),
                None => Self::ApplicationOctetStream(),
            },
            None => Self::ApplicationOctetStream(),
        }
    }

    // ======================
    // TEXT Content Types
    // ======================
    pub fn TextHtml() -> Self {
        Self::Text {
            subtype: "html".to_string(),
            charset: Some("UTF-8".to_string()),
        }
    }

    pub fn TextPlain() -> Self {
        Self::Text {
            subtype: "plain".to_string(),
            charset: Some("UTF-8".to_string()),
        }
    }

    pub fn TextCss() -> Self {
        Self::Text {
            subtype: "css".to_string(),
            charset: Some("UTF-8".to_string()),
        }
    }

    pub fn TextXml() -> Self {
        Self::Text {
            subtype: "xml".to_string(),
            charset: Some("UTF-8".to_string()),
        }
    }

    pub fn TextCsv() -> Self {
        Self::Text {
            subtype: "csv".to_string(),
            charset: Some("UTF-8".to_string()),
        }
    }

    // ======================
    // APPLICATION Content Types
    // ======================
    pub fn ApplicationJavascript() -> Self {
        Self::Application {
            subtype: "javascript".to_string(),
            parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]),
        }
    }

    pub fn ApplicationJson() -> Self {
        Self::Application {
            subtype: "json".to_string(),
            parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]),
        }
    }

    pub fn ApplicationUrlEncodedForm() -> Self {
        Self::Application {
            subtype: "x-www-form-urlencoded".to_string(),
            parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]),
        }
    }

    pub fn ApplicationXml() -> Self {
        Self::Application {
            subtype: "xml".to_string(),
            parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]),
        }
    }

    pub fn ApplicationOctetStream() -> Self {
        Self::Application {
            subtype: "octet-stream".to_string(),
            parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]),
        }
    }

    pub fn ApplicationPdf() -> Self {
        Self::Application {
            subtype: "pdf".to_string(),
            parameters: None,
        }
    }

    pub fn ApplicationZip() -> Self {
        Self::Application {
            subtype: "zip".to_string(),
            parameters: None,
        }
    }

    pub fn ApplicationPkcs8() -> Self {
        Self::Application {
            subtype: "pkcs8".to_string(),
            parameters: None,
        }
    }

    pub fn ApplicationWasm() -> Self {
        Self::Application {
            subtype: "wasm".to_string(),
            parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]),
        }
    }

    // ======================
    // IMAGE Content Types
    // ======================
    pub fn ImagePng() -> Self {
        Self::Image {
            subtype: "png".to_string(),
        }
    }

    pub fn ImageJpeg() -> Self {
        Self::Image {
            subtype: "jpeg".to_string(),
        }
    }

    pub fn ImageGif() -> Self {
        Self::Image {
            subtype: "gif".to_string(),
        }
    }

    pub fn ImageSvg() -> Self {
        Self::Image {
            subtype: "svg+xml".to_string(),
        }
    }

    pub fn ImageWebp() -> Self {
        Self::Image {
            subtype: "webp".to_string(),
        }
    }

    pub fn ImageBmp() -> Self {
        Self::Image {
            subtype: "bmp".to_string(),
        }
    }

    // ======================
    // AUDIO/VIDEO Content Types
    // ======================
    pub fn AudioMpeg() -> Self {
        Self::Audio {
            subtype: "mpeg".to_string(),
        }
    }

    pub fn VideoMp4() -> Self {
        Self::Video {
            subtype: "mp4".to_string(),
        }
    }

    // ======================
    // MODEL Content Types
    // ======================
    pub fn ModelStl() -> Self {
        Self::Model {
            subtype: "stl".to_string(),
        }
    }

    pub fn ModelObj() -> Self {
        Self::Model {
            subtype: "obj".to_string(),
        }
    }
}

impl std::fmt::Display for HttpContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for HttpContentType {
    fn default() -> Self {
        Self::Other {
            type_name: "Unknown".to_string(),
            subtype: "Unknown".to_string(),
            parameters: None,
        }
    }
}

impl HttpContentType {
    /// Reconciles an existing content type with a generated content type.
    ///
    /// Missing types use the available value. A generic generated binary type
    /// preserves the existing type. Text retains the existing subtype because
    /// the generated text subtype is a default. Matching application/custom
    /// subtypes retain extra parameters. Generated charset/parameter values win
    /// conflicts, and multipart boundaries always come from the generated type.
    #[av::ver(
        unstable,
        since = "0.8.6",
        note = "Reconcile declared and generated content types",
        date = "2026-11-13"
    )]
    pub fn reconcile(existing: Option<Self>, generated: Option<Self>) -> Option<Self> {
        fn merge_parameters(
            existing: Option<Vec<(String, String)>>,
            generated: Option<Vec<(String, String)>>,
        ) -> Option<Vec<(String, String)>> {
            match (existing, generated) {
                (Some(mut existing), Some(generated)) => {
                    for (key, value) in generated {
                        if let Some((_, existing_value)) = existing
                            .iter_mut()
                            .find(|(existing_key, _)| existing_key.eq_ignore_ascii_case(&key))
                        {
                            *existing_value = value;
                        } else {
                            existing.push((key, value));
                        }
                    }
                    Some(existing)
                }
                (Some(parameters), None) | (None, Some(parameters)) => Some(parameters),
                (None, None) => None,
            }
        }

        let (existing, generated) = match (existing, generated) {
            (Some(existing), Some(generated)) => (existing, generated),
            (existing, generated) => return existing.or(generated),
        };
        if let Self::Application { subtype, .. } = &generated
            && subtype.eq_ignore_ascii_case("octet-stream")
        {
            return Some(existing);
        }

        Some(match (existing, generated) {
            (
                Self::Text { subtype, charset },
                Self::Text {
                    subtype: _,
                    charset: generated_charset,
                },
            ) => Self::Text {
                subtype,
                charset: generated_charset.or(charset),
            },
            (
                Self::Application {
                    subtype,
                    parameters,
                },
                Self::Application {
                    subtype: generated_subtype,
                    parameters: generated_parameters,
                },
            ) if subtype.eq_ignore_ascii_case(&generated_subtype) => Self::Application {
                subtype,
                parameters: merge_parameters(parameters, generated_parameters),
            },
            (
                Self::Other {
                    type_name,
                    subtype,
                    parameters,
                },
                Self::Other {
                    type_name: generated_type_name,
                    subtype: generated_subtype,
                    parameters: generated_parameters,
                },
            ) if type_name.eq_ignore_ascii_case(&generated_type_name)
                && subtype.eq_ignore_ascii_case(&generated_subtype) =>
            {
                Self::Other {
                    type_name,
                    subtype,
                    parameters: merge_parameters(parameters, generated_parameters),
                }
            }
            (_, generated) => generated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconcile_handles_missing_types() {
        let content_type = HttpContentType::TextPlain();
        assert_eq!(HttpContentType::reconcile(None, None), None);
        assert_eq!(
            HttpContentType::reconcile(Some(content_type.clone()), None),
            Some(content_type.clone())
        );
        assert_eq!(
            HttpContentType::reconcile(None, Some(content_type.clone())),
            Some(content_type)
        );
    }

    #[test]
    fn reconcile_retains_extra_details_and_resolves_parameter_conflicts() {
        for (existing, generated, expected) in [
            (
                "text/html; charset=ISO-8859-1",
                "text/html",
                "text/html; charset=ISO-8859-1",
            ),
            (
                "text/plain; charset=ASCII",
                "text/html; charset=UTF-8",
                "text/plain; charset=UTF-8",
            ),
            (
                "application/json; charset=latin1; profile=example",
                "application/json; charset=UTF-8",
                "application/json; charset=UTF-8; profile=example",
            ),
            (
                "custom/type; MODE=old; profile=example",
                "custom/type; mode=new",
                "custom/type; MODE=new; profile=example",
            ),
        ] {
            assert_eq!(
                HttpContentType::reconcile(
                    Some(HttpContentType::from_str(existing)),
                    Some(HttpContentType::from_str(generated)),
                ),
                Some(HttpContentType::from_str(expected)),
                "existing: {existing}, generated: {generated}"
            );
        }
    }

    #[test]
    fn reconcile_prefers_generated_types_for_conflicts_and_multipart_boundaries() {
        for (existing, generated) in [
            ("application/xml; profile=old", "application/json"),
            ("first/custom; mode=old", "second/custom"),
            (
                "multipart/form-data; boundary=old",
                "multipart/form-data; boundary=body",
            ),
        ] {
            let expected = HttpContentType::from_str(generated);
            assert_eq!(
                HttpContentType::reconcile(
                    Some(HttpContentType::from_str(existing)),
                    Some(expected.clone())
                ),
                Some(expected),
                "existing: {existing}, generated: {generated}"
            );
        }
    }

    #[test]
    fn reconcile_preserves_specific_types_over_generic_binary_fallbacks() {
        let specific = HttpContentType::from_str("image/png");
        let generic = HttpContentType::ApplicationOctetStream();
        for (existing, generated) in [
            (specific.clone(), generic.clone()),
            (generic, specific.clone()),
        ] {
            assert_eq!(
                HttpContentType::reconcile(Some(existing), Some(generated)),
                Some(specific.clone())
            );
        }
    }
}
