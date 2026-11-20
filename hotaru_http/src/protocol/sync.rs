//! Outgoing metadata synchronization settings.

/// Controls which outgoing metadata fields are updated during synchronization.
///
/// These flags control header updates. Compression and transfer framing are
/// separate operations. All fields are synchronized by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[av::ver(
    unstable,
    since = "0.8.6",
    note = "Typed outgoing metadata synchronization settings",
    date = "2026-11-13"
)]
pub struct HttpSyncSettings {
    /// Update Content-Encoding from the prepared body.
    pub sync_encoding: bool,
    /// Update Content-Length for the selected message framing.
    pub sync_length: bool,
    /// Reconcile Content-Type with the prepared body.
    pub sync_content_type: bool,
    /// Add a Date header to outgoing messages when missing or unparseable.
    pub sync_date: bool,
}
impl Default for HttpSyncSettings {
    fn default() -> Self {
        Self {
            sync_encoding: true,
            sync_length: true,
            sync_content_type: true,
            sync_date: true,
        }
    }
}
