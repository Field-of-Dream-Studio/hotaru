use super::MultiFormFieldFile;
use super::error::MultiFormFieldError;

/// Represents a field in a multipart form.
#[derive(Debug, Clone)]
pub enum MultiFormField {
    Text(String),
    File(Vec<MultiFormFieldFile>),
}

impl MultiFormField {
    pub fn new_text(value: String) -> Self {
        Self::Text(value)
    }

    pub fn new_file(file: MultiFormFieldFile) -> Self {
        Self::File(vec![file])
    }

    /// Gets the text value from the `MultiFormField`.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Use the optional fallible multipart getter shape",
        date = "2026-11-12"
    )]
    pub fn get_text(&self) -> Result<Option<&String>, MultiFormFieldError> {
        match self {
            Self::Text(value) => Ok(Some(value)),
            Self::File(_) => Err(MultiFormFieldError::ContentTypeError),
        }
    }

    /// Inserts a file into the `MultiFormField`.
    /// Returns [`MultiFormFieldError::ContentTypeError`] when the field is text.
    pub fn insert_file(&mut self, file: MultiFormFieldFile) -> Result<(), MultiFormFieldError> {
        match self {
            Self::File(files) => {
                files.push(file);
                return Result::Ok(());
            }
            Self::Text(_) => {
                return Result::Err(MultiFormFieldError::ContentTypeError);
            }
        }
    }

    /// Gets the file list from the `MultiFormField`.
    /// An existing empty list is returned as `Some`, not as an absent value.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Use the optional fallible multipart getter shape",
        date = "2026-11-12"
    )]
    pub fn get_files(&self) -> Result<Option<&Vec<MultiFormFieldFile>>, MultiFormFieldError> {
        match self {
            Self::File(files) => {
                return Ok(Some(files));
            }
            Self::Text(_) => {
                return Err(MultiFormFieldError::ContentTypeError);
            }
        }
    }

    /// Gets the first file from the `MultiFormField`.
    /// Returns `None` for an empty file list and an error for a text field.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Preserve an absent first multipart file as None",
        date = "2026-11-12"
    )]
    pub fn get_first_file(&self) -> Result<Option<&MultiFormFieldFile>, MultiFormFieldError> {
        match self {
            Self::File(files) => Ok(files.first()),
            Self::Text(_) => Err(MultiFormFieldError::ContentTypeError),
        }
    }

    /// Gets the byte slice of the first file from the `MultiFormField`.
    #[av::ver(
        update,
        since = "0.8.6",
        note = "Preserve absent multipart file content as None",
        date = "2026-11-12"
    )]
    pub fn get_first_file_content(&self) -> Result<Option<&[u8]>, MultiFormFieldError> {
        self.get_first_file()
            .map(|file| file.map(MultiFormFieldFile::data))
    }
}

impl Default for MultiFormField {
    fn default() -> Self {
        Self::Text("".to_string())
    }
}
