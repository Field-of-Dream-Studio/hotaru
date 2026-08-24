/// Represents a file in a multipart form.
#[derive(Debug, Clone, Default)]
pub struct MultiFormFieldFile {
    pub(crate) filename: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) data: Vec<u8>,
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

    pub fn set_filename(&mut self, filename: Option<String>) {
        self.filename = filename;
    }

    pub fn set_content_type(&mut self, content_type: Option<String>) {
        self.content_type = content_type;
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}
