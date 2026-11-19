use super::HttpMeta;
use crate::util::date::HttpDate;

impl HttpMeta {
    /// Returns the cached `Date` value, parsing the header on first access.
    pub fn get_date(&mut self) -> Option<HttpDate> {
        if let Some(d) = self.date {
            return Some(d);
        }
        self.parse_date()
    }

    /// Parse the `Date` header (IMF-fixdate only) and cache it. `None` on miss.
    pub fn parse_date(&mut self) -> Option<HttpDate> {
        let raw = self.header.get("date").map(|value| value.first())?;
        let parsed = HttpDate::parse_imf_fixdate(&raw)?;
        self.date = Some(parsed);
        Some(parsed)
    }

    /// Sets (or clears) the cached `Date` value without touching the header map.
    pub fn set_date(&mut self, date: Option<HttpDate>) {
        self.date = date;
    }

    /// Clear the cached `Date`; header stays in the map, will re-parse on next get.
    pub fn clear_date(&mut self) {
        self.date = None;
    }

    /// Remove the `Date` header entirely (cache + map entry).
    pub fn delete_date(&mut self) {
        self.date = None;
        self.header.remove("date");
    }
}
