use super::*;

#[test]
fn test_parse_valid_multipart() {
    let boundary = "boundary123";
    let body = concat!(
        "--boundary123\r\n",
        "Content-Disposition: form-data; name=\"field1\"\r\n\r\n",
        "value1\r\n",
        "--boundary123\r\n",
        "Content-Disposition: form-data; name=\"file1\"; filename=\"example.txt\"\r\n",
        "Content-Type: text/plain\r\n\r\n",
        "file content here\r\n",
        "--boundary123--\r\n"
    )
    .as_bytes()
    .to_vec();

    let form = MultiForm::parse(body, boundary).unwrap();
    assert_eq!(form.len(), 2);
    assert!(form.contains_key("field1"));
    assert!(form.contains_key("file1"));
    assert_eq!(form.get_text("field1").unwrap(), "value1");
    assert_eq!(
        form.get_first_file("file1").unwrap().filename(),
        Some("example.txt".to_string())
    );
    assert_eq!(
        form.get_first_file_content("file1").unwrap(),
        b"file content here"
    );
}

#[test]
fn test_parse_multiple_files_same_field() {
    let boundary = "bound";
    let body = concat!(
        "--bound\r\n",
        "Content-Disposition: form-data; name=\"upload\"; filename=\"f1.txt\"\r\n",
        "Content-Type: text/plain\r\n\r\n",
        "file 1\r\n",
        "--bound\r\n",
        "Content-Disposition: form-data; name=\"upload\"; filename=\"f2.txt\"\r\n",
        "Content-Type: text/plain\r\n\r\n",
        "file 2\r\n",
        "--bound--\r\n"
    )
    .as_bytes();

    let form = MultiForm::parse(body, boundary).unwrap();
    assert_eq!(form.len(), 1);
    let files = form.get_files("upload").unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].filename(), Some("f1.txt".to_string()));
    assert_eq!(files[1].filename(), Some("f2.txt".to_string()));
}

#[test]
fn test_parse_empty_boundary_fails() {
    let result = MultiForm::parse(b"--xyz--", "");
    assert_eq!(result.unwrap_err(), MultipartError::EmptyBoundary);
}

#[test]
fn test_parse_missing_boundary_fails() {
    let result = MultiForm::parse(b"some random body without boundary", "bound");
    assert_eq!(result.unwrap_err(), MultipartError::IncompleteSection);
}

#[test]
fn test_parse_incomplete_section_fails() {
    let body = concat!(
        "--boundary123\r\n",
        "Content-Disposition: form-data; name=\"field1\"\r\n\r\n",
        "value1\r\n"
    )
    .as_bytes();
    let result = MultiForm::parse(body, "boundary123");
    assert_eq!(result.unwrap_err(), MultipartError::IncompleteSection);
}

#[test]
fn test_parse_missing_content_disposition_fails() {
    let body = concat!(
        "--boundary123\r\n",
        "Content-Type: text/plain\r\n\r\n",
        "value1\r\n",
        "--boundary123--\r\n"
    )
    .as_bytes();
    let result = MultiForm::parse(body, "boundary123");
    assert_eq!(
        result.unwrap_err(),
        MultipartError::MissingContentDisposition
    );
}

#[test]
fn test_parse_missing_field_name_fails() {
    let body = concat!(
        "--boundary123\r\n",
        "Content-Disposition: form-data; filename=\"foo.txt\"\r\n\r\n",
        "value1\r\n",
        "--boundary123--\r\n"
    )
    .as_bytes();
    let result = MultiForm::parse(body, "boundary123");
    assert_eq!(result.unwrap_err(), MultipartError::MissingFieldName);
}

#[test]
fn test_parse_invalid_utf8_in_text_field_fails() {
    let mut body = Vec::new();
    body.extend_from_slice(
        b"--boundary123\r\nContent-Disposition: form-data; name=\"text\"\r\n\r\n",
    );
    body.extend_from_slice(&[0xff, 0xfe]);
    body.extend_from_slice(b"\r\n--boundary123--\r\n");

    let result = MultiForm::parse(body, "boundary123");
    assert_eq!(result.unwrap_err(), MultipartError::InvalidUtf8);
}

#[test]
fn test_round_trip_serialization() {
    let mut form = MultiForm::new();
    form.insert(
        "author".to_string(),
        MultiFormField::new_text("Hotaru".to_string()),
    );
    form.insert(
        "file".to_string(),
        MultiFormField::new_file(MultiFormFieldFile::new(
            Some("doc.txt".to_string()),
            Some("text/plain".to_string()),
            b"hello world".to_vec(),
        )),
    );

    let serialized = form.to_string("myboundary");
    let parsed = MultiForm::parse(serialized.as_bytes(), "myboundary").unwrap();

    assert_eq!(parsed.get_text("author").unwrap(), "Hotaru");
    assert_eq!(
        parsed.get_first_file("file").unwrap().filename(),
        Some("doc.txt".to_string())
    );
    assert_eq!(
        parsed.get_first_file_content("file").unwrap(),
        b"hello world"
    );
}
