//! Typed `Date` header value (RFC 7231 §7.1.1.1). Stored as UTC seconds
//! since the Unix epoch; emits IMF-fixdate on the wire.

use std::fmt;

/// A parsed HTTP `Date` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HttpDate(u64);

impl HttpDate {
    /// Construct from UTC seconds since the Unix epoch.
    pub const fn from_unix_secs(secs: u64) -> Self {
        Self(secs)
    }

    /// UTC seconds since the Unix epoch.
    pub const fn as_unix_secs(self) -> u64 {
        self.0
    }

    /// Current wall-clock time, whole seconds. `None` if the clock is pre-epoch.
    pub fn now() -> Option<Self> {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| Self(d.as_secs()))
    }

    /// Parse IMF-fixdate (`Sun, 06 Nov 1994 08:49:37 GMT`). `None` on any miss.
    pub fn parse_imf_fixdate(s: &str) -> Option<Self> {
        // IMF-fixdate is exactly 29 bytes wide.
        let b = s.as_bytes();
        if b.len() != 29 {
            return None;
        }
        if &b[3..5] != b", "
            || b[7] != b' '
            || b[11] != b' '
            || b[16] != b' '
            || b[19] != b':'
            || b[22] != b':'
            || b[25] != b' '
            || &b[26..29] != b"GMT"
        {
            return None;
        }

        let day = parse_2digit(&b[5..7])?;
        let month = parse_month(&b[8..11])?;
        let year = parse_4digit(&b[12..16])?;
        let hour = parse_2digit(&b[17..19])?;
        let minute = parse_2digit(&b[20..22])?;
        let second = parse_2digit(&b[23..25])?;

        if year < 1970 || day < 1 || day > days_in_month(year, month) {
            return None;
        }
        if hour > 23 || minute > 59 || second > 60 {
            return None;
        }

        let days = days_from_civil(year as i32, month, day);
        let secs = (days as i64) * 86_400
            + (hour as i64) * 3_600
            + (minute as i64) * 60
            + (second as i64);
        if secs < 0 {
            return None;
        }
        Some(Self(secs as u64))
    }

    /// Format as IMF-fixdate.
    pub fn to_imf_fixdate(self) -> String {
        let secs = self.0;
        let days = (secs / 86_400) as i64;
        let tod = (secs % 86_400) as u32;
        let (year, month, day) = civil_from_days(days);
        let dow = weekday_from_days(days);
        format!(
            "{}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
            WEEKDAYS[dow as usize],
            day,
            MONTHS[(month - 1) as usize],
            year,
            tod / 3600,
            (tod / 60) % 60,
            tod % 60,
        )
    }
}

impl fmt::Display for HttpDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_imf_fixdate())
    }
}

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn parse_2digit(b: &[u8]) -> Option<u32> {
    let a = (b[0] as char).to_digit(10)?;
    let c = (b[1] as char).to_digit(10)?;
    Some(a * 10 + c)
}

fn parse_4digit(b: &[u8]) -> Option<u32> {
    let a = (b[0] as char).to_digit(10)?;
    let c = (b[1] as char).to_digit(10)?;
    let d = (b[2] as char).to_digit(10)?;
    let e = (b[3] as char).to_digit(10)?;
    Some(a * 1000 + c * 100 + d * 10 + e)
}

fn parse_month(b: &[u8]) -> Option<u32> {
    match b {
        b"Jan" => Some(1),
        b"Feb" => Some(2),
        b"Mar" => Some(3),
        b"Apr" => Some(4),
        b"May" => Some(5),
        b"Jun" => Some(6),
        b"Jul" => Some(7),
        b"Aug" => Some(8),
        b"Sep" => Some(9),
        b"Oct" => Some(10),
        b"Nov" => Some(11),
        b"Dec" => Some(12),
        _ => None,
    }
}

fn is_leap(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: u32, m: u32) -> u32 {
    const DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if m < 1 || m > 12 {
        return 0;
    }
    if m == 2 && is_leap(y) {
        29
    } else {
        DAYS[(m - 1) as usize]
    }
}

// Howard Hinnant's days_from_civil / civil_from_days on the proleptic
// Gregorian calendar, adapted from https://howardhinnant.github.io/date_algorithms.html.
// Both are exact for the full range we care about (year >= 1970).

fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146_097 + doe as i64 - 719_468
}

fn civil_from_days(z: i64) -> (u32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m, d)
}

// 1970-01-01 was a Thursday. 0=Sun, 4=Thu.
fn weekday_from_days(z: i64) -> u32 {
    (((z + 4) % 7 + 7) % 7) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 7231 §7.1.1.1 example.
    const RFC_EXAMPLE: &str = "Sun, 06 Nov 1994 08:49:37 GMT";
    const RFC_EXAMPLE_SECS: u64 = 784_111_777;

    #[test]
    fn parse_rfc_example() {
        let d = HttpDate::parse_imf_fixdate(RFC_EXAMPLE).unwrap();
        assert_eq!(d.as_unix_secs(), RFC_EXAMPLE_SECS);
    }

    #[test]
    fn format_rfc_example() {
        let d = HttpDate::from_unix_secs(RFC_EXAMPLE_SECS);
        assert_eq!(d.to_imf_fixdate(), RFC_EXAMPLE);
    }

    #[test]
    fn round_trip_epoch() {
        let d = HttpDate::from_unix_secs(0);
        assert_eq!(d.to_imf_fixdate(), "Thu, 01 Jan 1970 00:00:00 GMT");
        let back = HttpDate::parse_imf_fixdate(&d.to_imf_fixdate()).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(HttpDate::parse_imf_fixdate("Sun, 06 Nov 1994 08:49:37 GM").is_none());
        assert!(HttpDate::parse_imf_fixdate("Sun, 06 Nov 1994 08:49:37 GMTX").is_none());
    }

    #[test]
    fn rejects_bad_month() {
        assert!(HttpDate::parse_imf_fixdate("Sun, 06 Xxx 1994 08:49:37 GMT").is_none());
    }

    #[test]
    fn rejects_pre_epoch() {
        assert!(HttpDate::parse_imf_fixdate("Wed, 31 Dec 1969 23:59:59 GMT").is_none());
    }

    #[test]
    fn rejects_bad_day_of_month() {
        assert!(HttpDate::parse_imf_fixdate("Sun, 31 Feb 2021 00:00:00 GMT").is_none());
    }
}
