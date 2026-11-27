//! Small shared helpers: CSV rows, medians, timestamps and errors.

use std::fmt;

/// An error the tools report to the crew in one line, then exit with status 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fail(pub String);

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Fail {}

impl From<std::io::Error> for Fail {
    fn from(error: std::io::Error) -> Self {
        Fail(error.to_string())
    }
}

/// Shorthand for an `Err(Fail(..))`.
///
/// # Errors
/// Always: that is its purpose.
pub fn fail<T>(message: impl Into<String>) -> Result<T, Fail> {
    Err(Fail(message.into()))
}

/// Split one CSV line into fields. Handles quoted fields and doubled quotes.
#[must_use]
pub fn csv_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut field)),
            _ => field.push(ch),
        }
    }
    fields.push(field);
    fields
}

/// Every non-empty line of a CSV text as fields.
#[must_use]
pub fn csv_rows(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .map(|line| line.trim_end_matches('\r'))
        .filter(|line| !line.is_empty())
        .map(csv_fields)
        .collect()
}

/// Write one CSV row. Fields here never need quoting.
#[must_use]
pub fn csv_line(fields: &[String]) -> String {
    let mut line = fields.join(",");
    line.push_str("\r\n");
    line
}

/// The median of `values`, averaging the middle pair for an even count.
///
/// # Panics
/// If `values` is empty.
#[must_use]
pub fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    assert!(n > 0, "median of no values");
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        f64::midpoint(sorted[n / 2 - 1], sorted[n / 2])
    }
}

/// The arithmetic mean of `values`.
#[must_use]
pub fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

/// The current UTC time as `YYYY-MM-DDTHH:MM:SS+00:00`.
#[must_use]
pub fn utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    utc_from_unix(secs)
}

/// Format seconds since 1970 as an ISO 8601 UTC time.
#[must_use]
pub fn utc_from_unix(secs: u64) -> String {
    let days = (secs / 86_400).cast_signed();
    let rem = secs % 86_400;
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}+00:00",
        rem / 3600,
        rem / 60 % 60,
        rem % 60
    )
}

/// Write a JSON sidecar the way Python's `json.dump(indent=2)` does.
///
/// # Errors
/// If the file cannot be written.
pub fn write_json(path: &str, value: &serde_json::Value) -> Result<(), Fail> {
    let text = serde_json::to_string_pretty(value).map_err(|e| Fail(e.to_string()))?;
    std::fs::write(path, text).map_err(|e| Fail(format!("{path}: {e}")))
}

/// Read a JSON sidecar (`<file>.json`) if there is one.
#[must_use]
pub fn sidecar(path: &str) -> serde_json::Map<String, serde_json::Value> {
    std::fs::read_to_string(format!("{path}.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// A string field from a sidecar, or `default`.
#[must_use]
pub fn field<'a>(
    meta: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
    default: &'a str,
) -> &'a str {
    meta.get(key).and_then(|v| v.as_str()).unwrap_or(default)
}

/// A local date and time on this computer as Unix seconds, using its time zone (as `rtl_power`
/// writes its rows). `None` for an impossible date or time, and on systems without `mktime`.
#[must_use]
pub fn local_to_unix_s(
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
) -> Option<u64> {
    if !(1970..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=60).contains(&second)
    {
        return None;
    }
    #[cfg(unix)]
    #[allow(unsafe_code)]
    {
        // SAFETY: `tm` is a plain C struct, zeroed then filled in; mktime only reads it and
        // writes back normalised fields.
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        tm.tm_year = year - 1900;
        tm.tm_mon = month - 1;
        tm.tm_mday = day;
        tm.tm_hour = hour;
        tm.tm_min = minute;
        tm.tm_sec = second;
        tm.tm_isdst = -1; // let the system decide whether daylight saving applies
        let t = unsafe { libc::mktime(&raw mut tm) };
        // mktime normalises out-of-range days (31 February): refuse those instead.
        if t < 0 || tm.tm_mday != day || tm.tm_mon != month - 1 {
            return None;
        }
        u64::try_from(t).ok()
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_quotes() {
        assert_eq!(csv_fields(r#"a,"b,c","d""e""#), vec!["a", "b,c", r#"d"e"#]);
    }

    #[test]
    fn medians() {
        assert!((median(&[3.0, 1.0, 2.0]) - 2.0).abs() < 1e-12);
        assert!((median(&[4.0, 1.0, 2.0, 3.0]) - 2.5).abs() < 1e-12);
    }

    #[test]
    fn utc() {
        assert_eq!(utc_from_unix(0), "1970-01-01T00:00:00+00:00");
        assert_eq!(utc_from_unix(1_795_651_200), "2026-11-26T00:00:00+00:00");
    }
}
