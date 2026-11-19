use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::error::{MultiFormFieldError, MultipartError};
use super::parse::parse_multipart;
use super::serialize::{serialize_multipart, serialize_multipart_bytes};
use super::{Boundary, MultiFormField, MultiFormFieldFile};

/// Represents multipart form data.
#[derive(Debug, Clone)]
#[av::ver(
    update,
    since = "0.8.6",
    note = "Added storage for a multipart boundary",
    date = "2026-11-10"
)]
pub struct MultiForm {
    pub(crate) data: HashMap<String, MultiFormField>,
    /// Validated multipart boundary used for serialization.
    boundary: Boundary,
}

impl Default for MultiForm {
    /// Creates an empty form using the default multipart boundary.
    fn default() -> Self {
        Self::new(Boundary::default())
    }
}

impl From<HashMap<String, MultiFormField>> for MultiForm {
    fn from(data: HashMap<String, MultiFormField>) -> Self {
        Self {
            data,
            ..Self::default()
        }
    }
}

impl MultiForm {
    /// For a string boundary, use [`Self::try_new`].
    ///
    /// Creates an empty form with an already-validated boundary.
    pub fn new(boundary: Boundary) -> Self {
        Self {
            data: HashMap::new(),
            boundary,
        }
    }

    /// Validates a string boundary and creates an empty form.
    ///
    /// Returns an error if [`Boundary::new`] rejects the boundary.
    pub fn try_new(boundary: impl Into<String>) -> Result<Self, MultipartError> {
        Boundary::new(boundary).map(Self::new)
    }

    /// Returns the stored multipart boundary.
    pub fn boundary(&self) -> &str {
        self.boundary.as_str()
    }

    /// For a string boundary, use [`Self::try_set_boundary`].
    ///
    /// Replaces the boundary with an already-validated value.
    pub fn set_boundary(&mut self, boundary: Boundary) {
        self.boundary = boundary;
    }

    /// Validates a string boundary and replaces the stored value only on success.
    pub fn try_set_boundary(&mut self, boundary: impl Into<String>) -> Result<(), MultipartError> {
        self.set_boundary(Boundary::new(boundary)?);
        Ok(())
    }

    /// Parses a multipart form data body into a `MultiForm`.
    ///
    /// # Arguments
    ///
    /// * `body` - The raw bytes of the multipart form data body
    /// * `boundary` - The boundary string specified in the Content-Type header
    ///
    /// # Errors
    ///
    /// Returns a [`MultipartError`] if the payload is malformed, missing headers,
    /// truncated, or invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hotaru_http::form::MultiForm;
    /// let boundary = "boundary123";
    /// let body = concat!(
    ///     "--boundary123\r\n",
    ///     "Content-Disposition: form-data; name=\"field1\"\r\n\r\n",
    ///     "value1\r\n",
    ///     "--boundary123\r\n",
    ///     "Content-Disposition: form-data; name=\"file1\"; filename=\"example.txt\"\r\n",
    ///     "Content-Type: text/plain\r\n\r\n",
    ///     "file content here\r\n",
    ///     "--boundary123--\r\n"
    /// ).as_bytes().to_vec();
    ///
    /// let form = MultiForm::parse(body, boundary).unwrap();
    /// assert_eq!(form.len(), 2);
    /// assert!(form.contains_key("field1"));
    /// assert!(form.contains_key("file1"));
    /// assert_eq!(form.get_text("field1").unwrap().unwrap(), "value1");
    /// assert_eq!(form.get_first_file("file1").unwrap().unwrap().filename(), Some("example.txt".to_string()));
    /// ```
    pub fn parse(
        body: impl AsRef<[u8]>,
        boundary: impl AsRef<str>,
    ) -> Result<Self, MultipartError> {
        parse_multipart(body, boundary)
    }

    /// Serializes the `MultiForm` into a string using its validated boundary.
    pub fn to_string(&self) -> String {
        serialize_multipart(self, self.boundary())
    }

    /// Serializes the form into multipart bytes using its validated boundary.
    /// File contents are preserved without UTF-8 conversion.
    pub fn into_bytes(self) -> Vec<u8> {
        serialize_multipart_bytes(&self)
    }

    /// Inserts a field into the `MultiForm`.
    pub fn insert(&mut self, key: String, value: MultiFormField) {
        self.data.insert(key, value);
    }

    /// Gets the field from the `MultiForm`.
    pub fn get(&self, key: &str) -> Option<&MultiFormField> {
        self.data.get(key)
    }

    /// Gets all fields from the `MultiForm`.
    pub fn get_all(&self) -> &HashMap<String, MultiFormField> {
        &self.data
    }

    /// Checks whether the form contains a specific key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns the number of elements in the form.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the form contains no elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Gets the text value from the `MultiForm` if the field exists and is text.
    /// A missing field returns `None`; a file field returns a type error.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Distinguish absent multipart text fields from type errors",
        date = "2026-11-12"
    )]
    pub fn get_text(&self, key: &str) -> Result<Option<&String>, MultiFormFieldError> {
        match self.data.get(key) {
            Some(field) => field.get_text(),
            None => Ok(None),
        }
    }

    /// Gets the text value from the `MultiForm`, or an empty string if not found.
    pub fn get_text_or_default(&self, key: &str) -> String {
        if let Ok(Some(value)) = self.get_text(key) {
            return value.clone();
        }
        "".to_string()
    }

    /// Gets the files from the `MultiForm` if the field exists and contains files.
    /// A missing field returns `None`; an existing empty file list returns `Some`.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Distinguish absent multipart file fields from type errors",
        date = "2026-11-12"
    )]
    pub fn get_files(
        &self,
        key: &str,
    ) -> Result<Option<&Vec<MultiFormFieldFile>>, MultiFormFieldError> {
        match self.data.get(key) {
            Some(field) => field.get_files(),
            None => Ok(None),
        }
    }

    /// Gets the files from the `MultiForm`, returning an empty slice reference if missing.
    pub fn get_files_or_default(&self, key: &str) -> &Vec<MultiFormFieldFile> {
        if let Ok(Some(files)) = self.get_files(key) {
            return files;
        }
        static EMPTY: Lazy<Vec<MultiFormFieldFile>> = Lazy::new(Vec::new);
        &EMPTY
    }

    /// Gets the first file for a given field key.
    /// Returns `None` if the field or its first file is absent.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Preserve an absent first multipart file as None",
        date = "2026-11-12"
    )]
    pub fn get_first_file(
        &self,
        key: &str,
    ) -> Result<Option<&MultiFormFieldFile>, MultiFormFieldError> {
        match self.data.get(key) {
            Some(field) => field.get_first_file(),
            None => Ok(None),
        }
    }

    /// Gets the first file for a given field key, or a default empty file if missing.
    pub fn get_first_file_or_default(&self, key: &str) -> &MultiFormFieldFile {
        if let Ok(Some(field)) = self.get_first_file(key) {
            return field;
        }
        static EMPTY: Lazy<MultiFormFieldFile> = Lazy::new(MultiFormFieldFile::default);
        &EMPTY
    }

    /// Gets the byte slice of the first file for a given field key.
    /// Returns `None` if the field or its first file is absent.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Return Result<Option<T>, E> for multipart file content",
        date = "2026-11-12"
    )]
    pub fn get_first_file_content(&self, key: &str) -> Result<Option<&[u8]>, MultiFormFieldError> {
        match self.data.get(key) {
            Some(field) => field.get_first_file_content(),
            None => Ok(None),
        }
    }

    /// Gets the byte slice of the first file for a given field key, or an empty slice if missing.
    pub fn get_first_file_content_or_default(&self, key: &str) -> &[u8] {
        if let Ok(Some(content)) = self.get_first_file_content(key) {
            return content;
        }
        static EMPTY: Lazy<Vec<u8>> = Lazy::new(Vec::new);
        &EMPTY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_are_distinct_from_wrong_field_types() {
        let mut form = MultiForm::default();
        assert!(form.get_text("missing").unwrap().is_none());
        assert!(form.get_files("missing").unwrap().is_none());
        assert!(form.get_first_file("missing").unwrap().is_none());
        assert!(form.get_first_file_content("missing").unwrap().is_none());

        form.insert("text".to_string(), MultiFormField::new_text(String::new()));
        form.insert("files".to_string(), MultiFormField::File(Vec::new()));
        assert_eq!(form.get_text("text").unwrap().unwrap(), "");
        assert!(form.get_files("files").unwrap().unwrap().is_empty());
        assert!(matches!(
            form.get_text("files"),
            Err(MultiFormFieldError::ContentTypeError)
        ));
        assert!(matches!(
            form.get_files("text"),
            Err(MultiFormFieldError::ContentTypeError)
        ));
        assert!(matches!(
            form.get_first_file("text"),
            Err(MultiFormFieldError::ContentTypeError)
        ));
        assert!(matches!(
            form.get_first_file_content("text"),
            Err(MultiFormFieldError::ContentTypeError)
        ));
    }

    #[test]
    fn an_empty_file_list_is_distinct_from_an_empty_file() {
        let mut form = MultiForm::default();
        form.insert("files".to_string(), MultiFormField::File(Vec::new()));
        assert!(form.get_first_file("files").unwrap().is_none());
        assert!(form.get_first_file_content("files").unwrap().is_none());

        form.insert(
            "files".to_string(),
            MultiFormField::new_file(MultiFormFieldFile::default()),
        );
        assert!(form.get_first_file("files").unwrap().is_some());
        assert_eq!(
            form.get_first_file_content("files").unwrap(),
            Some(&b""[..])
        );
    }

    #[test]
    fn boundary_setters_preserve_validation_and_previous_value_on_error() {
        let mut form = MultiForm::default();
        form.set_boundary(Boundary::new("validated-boundary").unwrap());
        assert_eq!(form.boundary(), "validated-boundary");

        assert_eq!(
            form.try_set_boundary(""),
            Err(MultipartError::EmptyBoundary)
        );
        assert_eq!(form.boundary(), "validated-boundary");

        form.try_set_boundary("replacement-boundary").unwrap();
        assert_eq!(form.boundary(), "replacement-boundary");
    }
}
