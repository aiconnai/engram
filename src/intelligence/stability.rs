//! Spacing-effect stability for memory retention and salience
//!
//! Implements cognitive spaced reinforcement (Cepeda et al. 2006) to compute
//! dynamic memory stability, distinguishing bursty transient accesses from
//! genuinely spaced durable usage.

use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};

use crate::error::{EngramError, Result};

/// Base stability for new or non-reinforced memories
pub const STABILITY_FLOOR: f32 = 1.0;

/// Maximum earned stability ceiling (bounds effective half-life at 4x base)
pub const STABILITY_CEILING: f32 = 4.0;

/// Minimum memory importance required to be eligible for stability growth
pub const MIN_REINFORCEMENT_IMPORTANCE: f32 = 0.3;

/// Minimum time interval required between reinforcements for the same memory (1 hour)
pub const SPACING_INTERVAL_SECONDS: i64 = 3600;

/// Maximum reinforcing events permitted in any rolling 24-hour window
pub const MAX_REINFORCEMENTS_PER_24H: i64 = 3;

/// Asymptotic step factor for diminishing returns
pub const STABILITY_INCREMENT_BASE: f32 = 0.15;

/// Compute next stability value given current stability using diminishing returns curve:
/// `stability += 0.15 * (1 - stability / 4.0)` capped at `4.0`.
pub fn calculate_next_stability(current: f32) -> f32 {
    let clamped = current.clamp(STABILITY_FLOOR, STABILITY_CEILING);
    let step = STABILITY_INCREMENT_BASE * (1.0 - clamped / STABILITY_CEILING);
    (clamped + step).min(STABILITY_CEILING)
}

/// Record a reinforcing engagement event for a memory if temporal spacing and eligibility criteria are met.
///
/// Returns `Ok(Some(new_stability))` if reinforcement occurred and stability was updated,
/// or `Ok(None)` if the event was gated (e.g. within 1h cooldown, >= 3 per 24h, or importance < 0.3).
pub fn record_reinforcement(
    conn: &Connection,
    memory_id: i64,
    now: DateTime<Utc>,
) -> Result<Option<f32>> {
    // 1. Check memory eligibility (exists, not deleted, importance >= 0.3)
    let memory_query = conn.query_row(
        "SELECT importance, COALESCE(stability, 1.0) FROM memories WHERE id = ? AND valid_to IS NULL",
        params![memory_id],
        |row| Ok((row.get::<_, f32>(0)?, row.get::<_, f32>(1)?)),
    );

    let (importance, current_stability) = match memory_query {
        Ok(res) => res,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Err(EngramError::NotFound(memory_id)),
        Err(e) => return Err(EngramError::from(e)),
    };

    if importance < MIN_REINFORCEMENT_IMPORTANCE {
        return Ok(None);
    }

    // 2. Check 1h minimum spacing cooldown from last reinforcement
    let last_reinforced_str: Option<String> = conn
        .query_row(
            "SELECT MAX(reinforced_at) FROM memory_reinforcements WHERE memory_id = ?",
            params![memory_id],
            |row| row.get(0),
        )
        .unwrap_or(None);

    if let Some(ref last_str) = last_reinforced_str {
        if let Ok(last_dt) = DateTime::parse_from_rfc3339(last_str) {
            let last_utc = last_dt.with_timezone(&Utc);
            if (now - last_utc).num_seconds() < SPACING_INTERVAL_SECONDS {
                return Ok(None);
            }
        }
    }

    // 3. Check rolling 24-hour reinforcement cap (< 3 in last 24h)
    let window_start = (now - Duration::hours(24)).to_rfc3339();
    let rolling_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_reinforcements WHERE memory_id = ? AND reinforced_at >= ?",
        params![memory_id, window_start],
        |row| row.get(0),
    )?;

    if rolling_count >= MAX_REINFORCEMENTS_PER_24H {
        return Ok(None);
    }

    // 4. All gates passed: calculate new stability and persist
    let next_stability = calculate_next_stability(current_stability);
    let now_str = now.to_rfc3339();

    conn.execute(
        "INSERT INTO memory_reinforcements (memory_id, reinforced_at) VALUES (?, ?)",
        params![memory_id, now_str],
    )?;

    conn.execute(
        "UPDATE memories SET stability = ? WHERE id = ?",
        params![next_stability, memory_id],
    )?;

    Ok(Some(next_stability))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    fn create_test_db() -> Storage {
        Storage::open_in_memory().expect("in-memory storage")
    }

    fn insert_test_memory(conn: &Connection, importance: f32, stability: f32) -> i64 {
        conn.execute(
            "INSERT INTO memories (content, memory_type, importance, stability, visibility, metadata, valid_from)
             VALUES ('test memory', 'note', ?1, ?2, 'private', '{}', CURRENT_TIMESTAMP)",
            params![importance, stability],
        )
        .expect("insert memory");
        conn.last_insert_rowid()
    }

    #[test]
    fn test_stability_curve_diminishing_returns() {
        let s0 = 1.0;
        let s1 = calculate_next_stability(s0);
        assert!((s1 - (1.0 + 0.15 * 0.75)).abs() < 1e-4); // ~1.1125

        let s2 = calculate_next_stability(s1);
        assert!(s2 > s1);
        assert!(s2 - s1 < s1 - s0); // Diminishing step

        // Ceiling bounded at 4.0
        let s_near_cap = calculate_next_stability(3.99);
        assert!(s_near_cap <= 4.0);
        assert_eq!(calculate_next_stability(4.0), 4.0);
    }

    #[test]
    fn test_reinforcement_temporal_gates() {
        let storage = create_test_db();
        let now = Utc::now();

        storage
            .with_connection(|conn| {
                let memory_id = insert_test_memory(conn, 0.5, 1.0);

                // 1st reinforcement: succeeds
                let res1 = record_reinforcement(conn, memory_id, now).expect("first reinforcement");
                assert!(res1.is_some());
                let s1 = res1.unwrap();
                assert!(s1 > 1.0);

                // Burst attempt (30 mins later): gated by 1h cooldown
                let res_burst = record_reinforcement(conn, memory_id, now + Duration::minutes(30))
                    .expect("burst reinforcement");
                assert!(res_burst.is_none());

                // 2nd reinforcement (2 hours later): succeeds
                let res2 = record_reinforcement(conn, memory_id, now + Duration::hours(2))
                    .expect("second reinforcement");
                assert!(res2.is_some());
                let s2 = res2.unwrap();
                assert!(s2 > s1);

                // 3rd reinforcement (4 hours later): succeeds
                let res3 = record_reinforcement(conn, memory_id, now + Duration::hours(4))
                    .expect("third reinforcement");
                assert!(res3.is_some());
                let s3 = res3.unwrap();
                assert!(s3 > s2);

                // 4th reinforcement in same 24h window (6 hours later): gated by 3/day rolling cap
                let res4 = record_reinforcement(conn, memory_id, now + Duration::hours(6))
                    .expect("fourth reinforcement");
                assert!(res4.is_none());

                // 5th reinforcement (25 hours later): rolling window shifted, succeeds
                let res5 = record_reinforcement(conn, memory_id, now + Duration::hours(25))
                    .expect("next day reinforcement");
                assert!(res5.is_some());
                let s5 = res5.unwrap();
                assert!(s5 > s3);

                Ok(())
            })
            .expect("run temporal gates test");
    }

    #[test]
    fn test_reinforcement_importance_floor() {
        let storage = create_test_db();
        let now = Utc::now();

        storage
            .with_connection(|conn| {
                let low_importance_id = insert_test_memory(conn, 0.2, 1.0);

                // Memory with importance < 0.3 is not eligible for stability reinforcement
                let res = record_reinforcement(conn, low_importance_id, now)
                    .expect("reinforce low importance");
                assert!(res.is_none());

                let stability: f32 = conn
                    .query_row(
                        "SELECT stability FROM memories WHERE id = ?",
                        params![low_importance_id],
                        |row| row.get(0),
                    )
                    .expect("query stability");
                assert_eq!(stability, 1.0);

                Ok(())
            })
            .expect("run importance floor test");
    }
}
