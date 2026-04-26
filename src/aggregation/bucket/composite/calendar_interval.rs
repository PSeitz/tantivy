use time::convert::{Day, Nanosecond};

const NS_IN_DAY: i64 = Nanosecond::per_t::<i128>(Day) as i64;

/// Computes the timestamp in nanoseconds corresponding to the beginning of the
/// year (January 1st at midnight UTC).
pub(super) fn try_year_bucket(timestamp_ns: i64) -> crate::Result<i64> {
    fast_year_bucket(timestamp_ns).ok_or_else(|| {
        crate::TantivyError::InvalidArgument(format!(
            "Failed to compute year bucket for timestamp {}",
            timestamp_ns
        ))
    })
}

/// Computes the timestamp in nanoseconds corresponding to the beginning of the
/// month (1st at midnight UTC).
pub(super) fn try_month_bucket(timestamp_ns: i64) -> crate::Result<i64> {
    fast_month_bucket(timestamp_ns).ok_or_else(|| {
        crate::TantivyError::InvalidArgument(format!(
            "Failed to compute month bucket for timestamp {}",
            timestamp_ns
        ))
    })
}

/// Computes the timestamp in nanoseconds corresponding to the beginning of the
/// week (Monday at midnight UTC).
pub(super) fn week_bucket(timestamp_ns: i64) -> i64 {
    // 1970-01-01 was a Thursday (weekday = 4)
    let days_since_epoch = timestamp_ns.div_euclid(NS_IN_DAY);
    // Find the weekday: 0=Monday, ..., 6=Sunday
    let weekday = (days_since_epoch + 3).rem_euclid(7);
    let monday_days_since_epoch = days_since_epoch - weekday;
    monday_days_since_epoch * NS_IN_DAY
}

/// Convert days since unix epoch (1970-01-01) into (year, month, day) using
/// the civil_from_days algorithm by Howard Hinnant. This is exact and fast,
/// branch-light, and avoids constructing a `UtcDateTime`.
///
/// Reference: http://howardhinnant.github.io/date_algorithms.html#civil_from_days
#[inline]
fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    // Shift epoch to 0000-03-01 (so leap day is at end of "year")
    let z = days_since_epoch + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp.wrapping_sub(9) }) as u32; // [1, 12]
    let year_adjusted = y + (if m <= 2 { 1 } else { 0 });
    (year_adjusted, m, d)
}

/// Inverse of `civil_from_days`. Returns days since unix epoch (1970-01-01).
#[inline]
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = year - (if month <= 2 { 1 } else { 0 });
    let era = y.div_euclid(400);
    let yoe = (y - era * 400) as u64; // [0, 399]
    let m = month as u64;
    let m_adj = if m > 2 { m - 3 } else { m + 9 }; // 0..=11
    let doy = (153 * m_adj + 2) / 5 + (day as u64) - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + (doe as i64) - 719_468
}

#[inline]
fn fast_year_bucket(timestamp_ns: i64) -> Option<i64> {
    let days = timestamp_ns.div_euclid(NS_IN_DAY);
    let (year, _m, _d) = civil_from_days(days);
    let start_days = days_from_civil(year, 1, 1);
    start_days.checked_mul(NS_IN_DAY)
}

#[inline]
fn fast_month_bucket(timestamp_ns: i64) -> Option<i64> {
    let days = timestamp_ns.div_euclid(NS_IN_DAY);
    let (year, month, _d) = civil_from_days(days);
    let start_days = days_from_civil(year, month, 1);
    start_days.checked_mul(NS_IN_DAY)
}

#[cfg(test)]
mod tests {
    use time::format_description::well_known::Iso8601;
    use time::UtcDateTime;

    use super::*;

    fn ts_ns(iso: &str) -> i64 {
        UtcDateTime::parse(iso, &Iso8601::DEFAULT)
            .unwrap()
            .unix_timestamp_nanos() as i64
    }

    #[test]
    fn test_year_bucket() {
        let ts = ts_ns("1970-01-01T00:00:00Z");
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1970-01-01T00:00:00Z"));

        let ts = ts_ns("1970-06-01T10:00:01.010Z");
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1970-01-01T00:00:00Z"));

        let ts = ts_ns("2008-12-31T23:59:59.999999999Z"); // leap year
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("2008-01-01T00:00:00Z"));

        let ts = ts_ns("2008-01-01T00:00:00Z"); // leap year
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("2008-01-01T00:00:00Z"));

        let ts = ts_ns("2010-12-31T23:59:59.999999999Z");
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("2010-01-01T00:00:00Z"));

        let ts = ts_ns("1972-06-01T00:10:00Z");
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1972-01-01T00:00:00Z"));

        // Pre-epoch (negative timestamp): exercises div_euclid flooring.
        let ts = ts_ns("1969-06-15T12:34:56.789Z");
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1969-01-01T00:00:00Z"));

        // One nanosecond before the unix epoch.
        let ts = ts_ns("1969-12-31T23:59:59.999999999Z");
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1969-01-01T00:00:00Z"));

        // Century non-leap year (1900 is divisible by 100 but not 400).
        let ts = ts_ns("1900-12-31T23:59:59.999999999Z");
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1900-01-01T00:00:00Z"));

        // Year 2000: leap-by-400, also an era boundary in the shifted calendar.
        let ts = ts_ns("2000-01-01T00:00:00Z");
        let res = try_year_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("2000-01-01T00:00:00Z"));
    }

    #[test]
    fn test_month_bucket() {
        let ts = ts_ns("1970-01-15T00:00:00Z");
        let res = try_month_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1970-01-01T00:00:00Z"));

        let ts = ts_ns("1970-02-01T00:00:00Z");
        let res = try_month_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1970-02-01T00:00:00Z"));

        let ts = ts_ns("2000-01-31T23:59:59.999999999Z");
        let res = try_month_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("2000-01-01T00:00:00Z"));

        // Pre-epoch month (negative timestamp).
        let ts = ts_ns("1969-06-15T12:00:00Z");
        let res = try_month_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1969-06-01T00:00:00Z"));

        // Leap day in a year-divisible-by-400 (Feb 29 exists, last ns of Feb).
        let ts = ts_ns("2000-02-29T23:59:59.999999999Z");
        let res = try_month_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("2000-02-01T00:00:00Z"));

        // Last ns of Feb in a regular leap year.
        let ts = ts_ns("2024-02-29T23:59:59.999999999Z");
        let res = try_month_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("2024-02-01T00:00:00Z"));

        // Century non-leap year: 1900 has no Feb 29, so Feb 28 is the last day of Feb.
        let ts = ts_ns("1900-02-28T23:59:59Z");
        let res = try_month_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1900-02-01T00:00:00Z"));

        // The day after, in the same non-leap century year, must snap to March.
        let ts = ts_ns("1900-03-01T00:00:00Z");
        let res = try_month_bucket(ts).unwrap();
        assert_eq!(res, ts_ns("1900-03-01T00:00:00Z"));
    }

    #[test]
    fn test_week_bucket() {
        let ts = ts_ns("1970-01-05T00:00:00Z"); // Monday
        let res = week_bucket(ts);
        assert_eq!(res, ts_ns("1970-01-05T00:00:00Z"));

        let ts = ts_ns("1970-01-05T23:59:59Z"); // Monday
        let res = week_bucket(ts);
        assert_eq!(res, ts_ns("1970-01-05T00:00:00Z"));

        let ts = ts_ns("1970-01-07T01:13:00Z"); // Wednesday
        let res = week_bucket(ts);
        assert_eq!(res, ts_ns("1970-01-05T00:00:00Z"));

        let ts = ts_ns("1970-01-11T23:59:59.999999999Z"); // Sunday
        let res = week_bucket(ts);
        assert_eq!(res, ts_ns("1970-01-05T00:00:00Z"));

        let ts = ts_ns("2025-10-16T10:41:59.010Z"); // Thursday
        let res = week_bucket(ts);
        assert_eq!(res, ts_ns("2025-10-13T00:00:00Z"));

        let ts = ts_ns("1970-01-01T00:00:00Z"); // Thursday
        let res = week_bucket(ts);
        assert_eq!(res, ts_ns("1969-12-29T00:00:00Z")); // Negative
    }
}
