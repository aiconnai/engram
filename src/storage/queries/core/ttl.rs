//! Checked relative-TTL arithmetic.
//!
//! `chrono::Duration::seconds` and `DateTime + Duration` panic on overflow, so a
//! caller-controlled `ttl_seconds` such as `i64::MAX` would abort the process.
//! Every TTL that reaches storage goes through [`expiry_after`] instead.

use super::*;

/// Largest relative TTL accepted, in seconds (100 years).
pub const MAX_TTL_SECONDS: i64 = 100 * 365 * 24 * 60 * 60;

/// `now + ttl_seconds`, or `InvalidInput` when `|ttl_seconds|` exceeds
/// [`MAX_TTL_SECONDS`] or the result is outside chrono's representable range.
pub fn expiry_after(now: DateTime<Utc>, ttl_seconds: i64) -> Result<DateTime<Utc>> {
    let out_of_range = || {
        EngramError::InvalidInput(format!(
            "ttl_seconds {ttl_seconds} is out of range (must be within +/-{MAX_TTL_SECONDS} seconds)"
        ))
    };
    if ttl_seconds.unsigned_abs() > MAX_TTL_SECONDS as u64 {
        return Err(out_of_range());
    }
    let delta = chrono::Duration::try_seconds(ttl_seconds).ok_or_else(out_of_range)?;
    now.checked_add_signed(delta).ok_or_else(out_of_range)
}

/// Largest day-based lookback/offset accepted (100 years).
pub const MAX_OFFSET_DAYS: i64 = MAX_TTL_SECONDS / 86_400;

/// `now - days`, or `InvalidInput` when `|days|` exceeds [`MAX_OFFSET_DAYS`].
///
/// Replaces `now - Duration::days(n)` for caller-controlled `n`, which panics
/// in chrono for huge values.
pub fn cutoff_days_ago(now: DateTime<Utc>, days: i64) -> Result<DateTime<Utc>> {
    if days.unsigned_abs() > MAX_OFFSET_DAYS as u64 {
        return Err(EngramError::InvalidInput(format!(
            "{days} days is out of range (must be within +/-{MAX_OFFSET_DAYS} days)"
        )));
    }
    // |days| <= 36500, so both steps are in range; keep them checked anyway.
    chrono::Duration::try_days(days)
        .and_then(|delta| now.checked_sub_signed(delta))
        .ok_or_else(|| EngramError::InvalidInput(format!("{days} days is out of range")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_days_ago_bounds_the_offset() {
        let now = Utc::now();
        assert_eq!(
            cutoff_days_ago(now, 30).unwrap(),
            now - chrono::Duration::days(30)
        );
        assert!(cutoff_days_ago(now, MAX_OFFSET_DAYS).is_ok());
        for bad in [MAX_OFFSET_DAYS + 1, i64::MAX, i64::MIN] {
            assert!(
                matches!(cutoff_days_ago(now, bad), Err(EngramError::InvalidInput(_))),
                "{bad}"
            );
        }
    }

    #[test]
    fn accepts_in_range_and_rejects_extremes() {
        let now = Utc::now();
        assert_eq!(
            expiry_after(now, 60).unwrap(),
            now + chrono::Duration::seconds(60)
        );
        assert!(expiry_after(now, -60).is_ok());
        assert!(expiry_after(now, MAX_TTL_SECONDS).is_ok());
        for bad in [MAX_TTL_SECONDS + 1, i64::MAX, i64::MIN, i64::MAX / 1000] {
            assert!(
                matches!(expiry_after(now, bad), Err(EngramError::InvalidInput(_))),
                "{bad}"
            );
        }
    }
}
