use std::collections::HashMap;

use crate::message::http_value::ContentDisposition;

use super::error::MultipartError;
use super::{MultiForm, MultiFormField, MultiFormFieldFile};

/// Finds a subsequence within a larger sequence of bytes.
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Extract headers from part and parse Content-Disposition and Content-Type.
fn parse_headers(
    headers: &[u8],
) -> Result<(ContentDisposition, Option<String>), MultipartError> {
    let headers_str = std::str::from_utf8(headers).map_err(|_| MultipartError::InvalidHeaders)?;
    let lines: Vec<&str> = headers_str.split("\r\n").collect();

    let mut content_disposition = None;
    let mut content_type = None;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((name, val)) = trimmed.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-disposition") {
                let disposition = ContentDisposition::parse(val.trim())
                    .map_err(|_| MultipartError::InvalidHeaders)?;
                content_disposition = Some(disposition);
            } else if name.trim().eq_ignore_ascii_case("content-type") {
                content_type = Some(val.trim().to_string());
            }
        }
    }

    let disposition = content_disposition.ok_or(MultipartError::MissingContentDisposition)?;
    Ok((disposition, content_type))
}

/// Parses raw bytes of a multipart form body using the specified boundary.
///
/// # Errors
///
/// Returns [`MultipartError`] if the boundary is invalid, delimiters are missing,
/// headers are malformed, required fields are absent, or text content is invalid UTF-8.
pub fn parse_multipart(
    body: impl AsRef<[u8]>,
    boundary: impl AsRef<str>,
) -> Result<MultiForm, MultipartError> {
    let boundary_str = boundary.as_ref();
    if boundary_str.is_empty() {
        return Err(MultipartError::EmptyBoundary);
    }

    let body = body.as_ref();
    if body.is_empty() {
        return Err(MultipartError::MissingBoundary);
    }

    let delimiter = format!("--{}", boundary_str);
    let delimiter_bytes = delimiter.as_bytes();

    let mut parts: Vec<&[u8]> = Vec::new();
    let mut start_idx = 0;
    let mut found_end = false;

    while let Some(idx) = find_subsequence(&body[start_idx..], delimiter_bytes) {
        if start_idx > 0 {
            let part_end = start_idx + idx;
            // Trim trailing \r\n before the delimiter
            let part_slice = if part_end >= start_idx + 2 && &body[part_end - 2..part_end] == b"\r\n" {
                &body[start_idx..part_end - 2]
            } else if part_end > start_idx && body[part_end - 1] == b'\n' {
                &body[start_idx..part_end - 1]
            } else {
                &body[start_idx..part_end]
            };
            parts.push(part_slice);
        }

        start_idx += idx + delimiter_bytes.len();

        // Check for end boundary: "--"
        if start_idx < body.len() && body.len() - start_idx >= 2 && &body[start_idx..start_idx + 2] == b"--" {
            found_end = true;
            break;
        }

        // Skip newline after delimiter
        if start_idx < body.len() && body.len() - start_idx >= 2 && &body[start_idx..start_idx + 2] == b"\r\n" {
            start_idx += 2;
        } else if start_idx < body.len() && body[start_idx] == b'\n' {
            start_idx += 1;
        }
    }

    if !found_end {
        return Err(MultipartError::IncompleteSection);
    }

    let mut form_map: HashMap<String, MultiFormField> = HashMap::new();

    for part in parts {
        if part.is_empty() {
            continue;
        }

        let header_end = find_subsequence(part, b"\r\n\r\n")
            .or_else(|| find_subsequence(part, b"\n\n"))
            .ok_or(MultipartError::MissingHeaders)?;

        let separator_len = if part[header_end..].starts_with(b"\r\n\r\n") {
            4
        } else {
            2
        };

        let headers = &part[..header_end];
        let content = &part[header_end + separator_len..];

        let (disposition, content_type) = parse_headers(headers)?;

        let field_name = disposition
            .get_parameter("name")
            .ok_or(MultipartError::MissingFieldName)?
            .to_string();

        if let Some(filename) = disposition.filename() {
            let file = MultiFormFieldFile::new(
                Some(filename.to_string()),
                content_type,
                content.to_vec(),
            );
            match form_map.get_mut(&field_name) {
                Some(field) => field.insert_file(file).map_err(|_| {
                    MultipartError::InvalidData(
                        "multipart field contains mixed text and file values".to_string(),
                    )
                })?,
                None => {
                    form_map.insert(field_name, MultiFormField::new_file(file));
                }
            }
        } else {
            let text_value = std::str::from_utf8(content).map_err(|_| MultipartError::InvalidUtf8)?;
            form_map.insert(
                field_name,
                MultiFormField::Text(text_value.to_string()),
            );
        }
    }

    Ok(form_map.into())
}
