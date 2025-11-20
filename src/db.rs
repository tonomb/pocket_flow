use rusqlite::{Connection, Result};
use std::path::PathBuf;
use chrono::{Local, Timelike};

use crate::models::WorkSession;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .expect("Failed to create application data directory");
        }
        
        let conn = Connection::open(db_path)?;
        
        let db = Database { conn };
        db.initialize()?;
        
        Ok(db)
    }
    
    fn get_db_path() -> PathBuf {
        // Hard-coded for macOS, but modular for future expansion
        let home = std::env::var("HOME").expect("HOME environment variable not set");
        let mut path = PathBuf::from(home);
        path.push("Library/Application Support/pocket_flow");
        path.push("sessions.db");
        path
    }
    
    fn initialize(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS work_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at TEXT NOT NULL,
                completed_at TEXT NOT NULL,
                duration_seconds INTEGER NOT NULL
            )",
            [],
        )?;
        
        Ok(())
    }
    
    pub fn save_work_session(&self, session: &WorkSession) -> Result<()> {
        self.conn.execute(
            "INSERT INTO work_sessions (started_at, completed_at, duration_seconds)
             VALUES (?1, ?2, ?3)",
            (
                session.started_at.to_rfc3339(),
                session.completed_at.to_rfc3339(),
                session.duration_seconds,
            ),
        )?;
        
        Ok(())
    }
    
    pub fn get_sessions_count_for_today(&self) -> Result<usize> {
        // Get start of today in local timezone
        let now = Local::now();
        let start_of_day = now
            .with_hour(0)
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .and_then(|d| d.with_nanosecond(0))
            .expect("Failed to calculate start of day");
        
        let start_of_day_str = start_of_day.to_rfc3339();
        
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM work_sessions WHERE started_at >= ?1",
            [start_of_day_str],
            |row| row.get(0),
        )?;
        
        Ok(count)
    }
}
