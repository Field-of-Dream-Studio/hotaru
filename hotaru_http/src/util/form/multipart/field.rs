use super::error::MultiFormFieldError;
use super::MultiFormFieldFile;

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
    pub fn get_text(&self) -> Result<&String, MultiFormFieldError> {
        match self {
            Self::Text(value) => Ok(value),
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
    pub fn get_files(&self) -> Result<&Vec<MultiFormFieldFile>, MultiFormFieldError> {
        match self {
            Self::File(files) => {
                return Ok(files);
            }
            Self::Text(_) => {
                return Err(MultiFormFieldError::ContentTypeError);
            }
        }
    }

    /// Gets the first file from the `MultiFormField`.
    pub fn get_first_file(&self) -> Result<&MultiFormFieldFile, MultiFormFieldError> {
        match self {
            Self::File(files) => files.first().ok_or(MultiFormFieldError::NoFile),
            Self::Text(_) => Err(MultiFormFieldError::ContentTypeError),
        }
    }

    /// Gets the byte slice of the first file from the `MultiFormField`.
    pub fn get_first_file_content(&self) -> Result<&[u8], MultiFormFieldError> {
        self.get_first_file().map(MultiFormFieldFile::data)
    }
}

impl Default for MultiFormField {
    fn default() -> Self {
        Self::Text("".to_string())
    }
}
