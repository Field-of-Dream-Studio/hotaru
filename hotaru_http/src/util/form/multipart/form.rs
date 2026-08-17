use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::error::MultipartError;
use super::parse::parse_multipart;
use super::serialize::serialize_multipart;

/// Represents multipart form data.
#[derive(Debug, Clone, Default)]
pub struct MultiForm {
    pub(crate) data: HashMap<String, MultiFormField>,
}

/// Represents a field in a multipart form.
#[derive(Debug, Clone)]
pub enum MultiFormField {
    Text(String),
    File(Vec<MultiFormFieldFile>),
}

/// Represents a file in a multipart form.
#[derive(Debug, Clone, Default)]
pub struct MultiFormFieldFile {
    pub(crate) filename: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) data: Vec<u8>,
}

impl From<HashMap<String, MultiFormField>> for MultiForm {
    fn from(data: HashMap<String, MultiFormField>) -> Self {
        Self { data }
    }
}

impl MultiForm {
    /// Creates a new `MultiForm` with an empty `HashMap`.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
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
    /// assert_eq!(form.get_text("field1").unwrap(), "value1");
    /// assert_eq!(form.get_first_file("file1").unwrap().filename(), Some("example.txt".to_string()));
    /// ```
    pub fn parse(body: impl AsRef<[u8]>, boundary: impl AsRef<str>) -> Result<Self, MultipartError> {
        parse_multipart(body, boundary)
    }

    /// Serializes the `MultiForm` into a string using the given boundary.
    pub fn to_string(&self, boundary: &str) -> String {
        serialize_multipart(self, boundary)
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
    pub fn get_text(&self, key: &str) -> Option<&String> {
        if let Some(MultiFormField::Text(value)) = self.data.get(key) {
            return Some(value);
        }
        None
    }

    /// Gets the text value from the `MultiForm`, or an empty string if not found.
    pub fn get_text_or_default(&self, key: &str) -> String {
        if let Some(MultiFormField::Text(value)) = self.data.get(key) {
            return value.clone();
        }
        "".to_string()
    }

    /// Gets the files from the `MultiForm` if the field exists and contains files.
    pub fn get_files(&self, key: &str) -> Option<&Vec<MultiFormFieldFile>> {
        if let Some(MultiFormField::File(files)) = self.data.get(key) {
            return Some(files);
        }
        None
    }

    /// Gets the files from the `MultiForm`, returning an empty slice reference if missing.
    pub fn get_files_or_default(&self, key: &str) -> &Vec<MultiFormFieldFile> {
        if let Some(MultiFormField::File(files)) = self.data.get(key) {
            return files;
        }
        static EMPTY: Lazy<Vec<MultiFormFieldFile>> = Lazy::new(Vec::new);
        &EMPTY
    }

    /// Gets the first file for a given field key.
    pub fn get_first_file(&self, key: &str) -> Option<&MultiFormFieldFile> {
        if let Some(MultiFormField::File(files)) = self.data.get(key) {
            return files.first();
        }
        None
    }

    /// Gets the first file for a given field key, or a default empty file if missing.
    pub fn get_first_file_or_default(&self, key: &str) -> &MultiFormFieldFile {
        if let Some(field) = self.get_first_file(key) {
            return field;
        }
        static EMPTY: Lazy<MultiFormFieldFile> = Lazy::new(MultiFormFieldFile::default);
        &EMPTY
    }

    /// Gets the byte slice of the first file for a given field key.
    pub fn get_first_file_content(&self, key: &str) -> Option<&[u8]> {
        if let Some(MultiFormField::File(files)) = self.data.get(key) {
            return files.first().map(|file| file.data.as_slice());
        }
        None
    }

    /// Gets the byte slice of the first file for a given field key, or an empty slice if missing.
    pub fn get_first_file_content_or_default(&self, key: &str) -> &[u8] {
        if let Some(content) = self.get_first_file_content(key) {
            return content;
        }
        static EMPTY: Lazy<Vec<u8>> = Lazy::new(Vec::new);
        &EMPTY
    }
}

impl MultiFormField {
    pub fn new_text(value: String) -> Self {
        Self::Text(value)
    }

    pub fn new_file(file: MultiFormFieldFile) -> Self {
        Self::File(vec![file])
    }

    /// Inserts a file into the `MultiFormField`.
    /// When the field is currently `Text`, it converts it into a `File` variant.
    pub fn insert_file(&mut self, file: MultiFormFieldFile) {
        if let Self::File(files) = self {
            files.push(file);
        } else {
            *self = Self::File(vec![file]);
        }
    }

    /// Gets the file list from the `MultiFormField`.
    pub fn get_files(&self) -> Option<&Vec<MultiFormFieldFile>> {
        if let Self::File(files) = self {
            Some(files)
        } else {
            None
        }
    }
}

impl Default for MultiFormField {
    fn default() -> Self {
        Self::Text("".to_string())
    }
}

impl MultiFormFieldFile {
    pub fn new(filename: Option<String>, content_type: Option<String>, data: Vec<u8>) -> Self {
        Self {
            filename,
            content_type,
            data,
        }
    }

    pub fn filename(&self) -> Option<String> {
        self.filename.clone()
    }

    pub fn content_type(&self) -> Option<String> {
        self.content_type.clone()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
