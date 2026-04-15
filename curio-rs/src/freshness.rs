//! Age-based freshness scoring for wiki pages.
//!
//! Returns a score in [0.0, 1.0] where 1.0 is brand-new and 0.0 is very stale.
//! Uses exponential decay with a half-life of ~8 months (240 days).

use chrono::{DateTime, Utc};

/// Compute a freshness score from the page's last-updated timestamp.
///
/// Decay model: `score = e^(-λ * days_old)` where λ = ln(2) / 240.
/// Half-life is 240 days (8 months): a page last updated 240 days ago scores ~0.5.
pub fn freshness_score(updated_at: DateTime<Utc>) -> f64 {
    let days_old = (Utc::now() - updated_at).num_days().max(0) as f64;
    let lambda = std::f64::consts::LN_2 / 240.0;
    (-lambda * days_old).exp().clamp(0.0, 1.0)
}

/// Parse an RFC-3339 timestamp string and return a freshness score.
/// Returns `None` if the string cannot be parsed.
pub fn freshness_score_from_str(updated_at: &str) -> Option<f64> {
    updated_at.parse::<DateTime<Utc>>().ok().map(freshness_score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_very_recent_page_is_fresh() {
        let updated = Utc::now();
        let score = freshness_score(updated);
        assert!(score > 0.95, "score was {}", score);
    }

    #[test]
    fn test_two_year_old_page_is_stale() {
        let updated = Utc::now() - chrono::Duration::days(730);
        let score = freshness_score(updated);
        assert!(score < 0.4, "score was {}", score);
    }

    #[test]
    fn test_six_month_page_is_borderline() {
        let updated = Utc::now() - chrono::Duration::days(180);
        let score = freshness_score(updated);
        assert!(score > 0.4 && score < 0.8, "score was {}", score);
    }

    #[test]
    fn test_freshness_from_str_valid() {
        let ts = "2025-01-01T00:00:00Z";
        let score = freshness_score_from_str(ts);
        assert!(score.is_some());
        assert!(score.unwrap() < 1.0);
    }

    #[test]
    fn test_freshness_from_str_invalid() {
        assert!(freshness_score_from_str("not-a-date").is_none());
    }
}
