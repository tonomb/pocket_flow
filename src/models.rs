use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct WorkSession {
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_seconds: i64,
}

impl WorkSession {
    pub fn new(started_at: DateTime<Utc>, completed_at: DateTime<Utc>) -> Self {
        let duration_seconds = (completed_at - started_at).num_seconds();
        Self {
            started_at,
            completed_at,
            duration_seconds,
        }
    }
}
