use std::path::{Path, PathBuf};
use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HistoryRecord {
    pub id: i64,
    pub command: String,
    pub cwd: String,
    pub git_root: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
    pub freq: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AliasRecord {
    pub id: i64,
    pub trigger: String,
    pub expansion: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open_default() -> Result<Self> {
        let db_dir = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("smart-hub");
        std::fs::create_dir_all(&db_dir).ok();
        let db_path = db_dir.join("history.db");
        Self::open(&db_path)
    }

    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Performance optimizations
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -32000;"
        )?;

        // Schema migrations
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command TEXT NOT NULL,
                cwd TEXT NOT NULL,
                git_root TEXT,
                exit_code INTEGER,
                duration_ms INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_history_cwd ON history(cwd);
            CREATE INDEX IF NOT EXISTS idx_history_cmd ON history(command);
            CREATE INDEX IF NOT EXISTS idx_history_created ON history(created_at DESC);

            CREATE TABLE IF NOT EXISTS aliases (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trigger TEXT NOT NULL UNIQUE,
                expansion TEXT NOT NULL,
                description TEXT,
                category TEXT
            );

            CREATE TABLE IF NOT EXISTS directory_visits (
                path TEXT PRIMARY KEY,
                visit_count INTEGER DEFAULT 1,
                last_visited DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        )?;

        let db = Self { conn };
        let _ = db.seed_psreadline_history_if_needed();
        Ok(db)
    }

    pub fn seed_psreadline_history_if_needed(&self) -> Result<usize> {
        let already_seeded: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'psreadline_seeded'",
                [],
                |r| r.get(0),
            )
            .ok();
        if already_seeded.as_deref() == Some("1") {
            return Ok(0);
        }

        let hist_path = dirs::data_dir().map(|d| {
            d.join("Microsoft")
                .join("Windows")
                .join("PowerShell")
                .join("PSReadLine")
                .join("ConsoleHost_history.txt")
        });

        let mut imported = 0usize;
        if let Some(path) = hist_path {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // Keep unique commands in order of their latest appearance
                    let mut seen = std::collections::HashSet::new();
                    let mut ordered_rev = Vec::new();
                    for line in content.lines().rev() {
                        let trimmed = line.trim();
                        if trimmed.len() < 2 || trimmed.ends_with('`') {
                            continue;
                        }
                        if seen.insert(trimmed.to_string()) {
                            ordered_rev.push(trimmed.to_string());
                            if ordered_rev.len() >= 5000 {
                                break;
                            }
                        }
                    }
                    ordered_rev.reverse();

                    self.conn.execute_batch("BEGIN IMMEDIATE;")?;
                    {
                        let mut stmt = self.conn.prepare(
                            "INSERT INTO history (command, cwd, git_root, exit_code, duration_ms)
                             VALUES (?1, '', NULL, 0, NULL)",
                        )?;
                        for cmd in ordered_rev {
                            let _ = stmt.execute(params![cmd]);
                            imported += 1;
                        }
                    }
                    self.conn.execute(
                        "INSERT OR REPLACE INTO meta (key, value) VALUES ('psreadline_seeded', '1')",
                        [],
                    )?;
                    self.conn.execute_batch("COMMIT;")?;
                    return Ok(imported);
                }
            }
        }

        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('psreadline_seeded', '1')",
            [],
        );
        Ok(0)
    }

    pub fn get_data_version(&self) -> i64 {
        self.conn
            .query_row("PRAGMA data_version", [], |r| r.get(0))
            .unwrap_or(0)
    }

    pub fn record(
        &self,
        command: &str,
        cwd: &str,
        git_root: Option<&str>,
        exit_code: Option<i32>,
        duration_ms: Option<i64>,
    ) -> Result<i64> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return Ok(0);
        }

        self.conn.execute(
            "INSERT INTO history (command, cwd, git_root, exit_code, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![trimmed, cwd, git_root, exit_code, duration_ms],
        )?;
        let id = self.conn.last_insert_rowid();

        // Update directory visits
        self.conn.execute(
            "INSERT INTO directory_visits (path, visit_count, last_visited)
             VALUES (?1, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(path) DO UPDATE SET
                visit_count = visit_count + 1,
                last_visited = CURRENT_TIMESTAMP",
            params![cwd],
        )?;

        Ok(id)
    }

    #[allow(dead_code)]
    pub fn delete_history(&self, id: i64) -> Result<bool> {
        let rows = self.conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    pub fn delete_command(&self, cmd: &str) -> Result<bool> {
        let rows = self.conn.execute("DELETE FROM history WHERE command = ?1", params![cmd])?;
        Ok(rows > 0)
    }

    pub fn delete_last(&self) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT id, command FROM history ORDER BY id DESC LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let cmd: String = row.get(1)?;
            self.conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
            Ok(Some(cmd))
        } else {
            Ok(None)
        }
    }

    pub fn get_local_history(
        &self,
        cwd: &str,
        git_root: Option<&str>,
        limit: usize,
    ) -> Result<Vec<HistoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT MAX(id) as max_id, command, cwd, git_root, exit_code, duration_ms, MAX(created_at), COUNT(*) as freq
             FROM history
             WHERE cwd = ?1 OR (?2 IS NOT NULL AND git_root = ?2)
             GROUP BY command
             ORDER BY CASE WHEN cwd = ?1 THEN 1 ELSE 0 END DESC, max_id DESC
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(params![cwd, git_root, limit as i64], |row| {
            Ok(HistoryRecord {
                id: row.get(0)?,
                command: row.get(1)?,
                cwd: row.get(2)?,
                git_root: row.get(3)?,
                exit_code: row.get(4)?,
                duration_ms: row.get(5)?,
                created_at: row.get(6)?,
                freq: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_global_history(&self, exclude_cwd: &str, limit: usize) -> Result<Vec<HistoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT MAX(id) as max_id, command, cwd, git_root, exit_code, duration_ms, MAX(created_at), COUNT(*) as freq
             FROM history
             WHERE cwd != ?1
             GROUP BY command
             ORDER BY max_id DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![exclude_cwd, limit as i64], |row| {
            Ok(HistoryRecord {
                id: row.get(0)?,
                command: row.get(1)?,
                cwd: row.get(2)?,
                git_root: row.get(3)?,
                exit_code: row.get(4)?,
                duration_ms: row.get(5)?,
                created_at: row.get(6)?,
                freq: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_all_aliases(&self) -> Result<Vec<AliasRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, trigger, expansion, description, category
             FROM aliases
             ORDER BY trigger ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AliasRecord {
                id: row.get(0)?,
                trigger: row.get(1)?,
                expansion: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn add_alias(
        &self,
        trigger: &str,
        expansion: &str,
        description: Option<&str>,
        category: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO aliases (trigger, expansion, description, category)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(trigger) DO UPDATE SET
                expansion = excluded.expansion,
                description = excluded.description,
                category = excluded.category",
            params![trigger, expansion, description, category],
        )?;
        Ok(())
    }

    pub fn remove_alias(&self, trigger: &str) -> Result<bool> {
        let rows = self.conn.execute("DELETE FROM aliases WHERE trigger = ?1", params![trigger])?;
        Ok(rows > 0)
    }

    pub fn get_recent_directories(&self, limit: usize) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT path, visit_count FROM directory_visits
             ORDER BY visit_count DESC, last_visited DESC
             LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }
}
