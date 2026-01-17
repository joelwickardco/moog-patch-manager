use rusqlite::{Connection, Result as SqliteResult, params};
use std::path::Path;
use super::SCHEMA;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &Path) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Initialize schema
        conn.execute_batch(SCHEMA)?;

        // Initialize default 16 banks if they don't exist
        let db = Self { conn };
        db.initialize_banks()?;

        Ok(db)
    }

    fn initialize_banks(&self) -> SqliteResult<()> {
        for bank_num in 1..=16 {
            let default_name = format!("Bank {:02}", bank_num);
            self.conn.execute(
                "INSERT OR IGNORE INTO banks (bank_number, name) VALUES (?1, ?2)",
                params![bank_num, default_name],
            )?;

            // Get bank_id for this bank
            let bank_id: i64 = self.conn.query_row(
                "SELECT id FROM banks WHERE bank_number = ?1",
                params![bank_num],
                |row| row.get(0),
            )?;

            // Initialize 16 patch slots for this bank
            for patch_num in 1..=16 {
                self.conn.execute(
                    "INSERT OR IGNORE INTO bank_patches (bank_id, patch_number, patch_id) VALUES (?1, ?2, NULL)",
                    params![bank_id, patch_num],
                )?;
            }

            // Initialize 16 sequence slots for this bank
            for seq_num in 1..=16 {
                self.conn.execute(
                    "INSERT OR IGNORE INTO bank_sequences (bank_id, sequence_number, sequence_id) VALUES (?1, ?2, NULL)",
                    params![bank_id, seq_num],
                )?;
            }
        }
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
