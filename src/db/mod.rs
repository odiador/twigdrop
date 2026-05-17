use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS branch_analysis (
                branch_name TEXT PRIMARY KEY,
                last_commit_hash TEXT NOT NULL,
                summary TEXT,
                cleanup_recommendation TEXT,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    pub fn get_analysis(&self, branch_name: &str) -> Result<Option<(String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT last_commit_hash, summary, cleanup_recommendation FROM branch_analysis WHERE branch_name = ?",
        )?;
        let mut rows = stmt.query([branch_name])?;

        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?)))
        } else {
            Ok(None)
        }
    }

    pub fn save_analysis(
        &self,
        branch_name: &str,
        hash: &str,
        summary: &str,
        cleanup: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO branch_analysis (branch_name, last_commit_hash, summary, cleanup_recommendation, updated_at)
             VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)",
            [branch_name, hash, summary, cleanup],
        )?;
        Ok(())
    }
}
