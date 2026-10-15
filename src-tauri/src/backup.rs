// The WebView never supplies paths. Only native dialogs call these service methods.
use crate::{domain::*, repository::Repository, service::Service};
use rusqlite::{backup::Backup, Connection, OpenFlags};
use std::{path::Path, time::Duration};

fn io(error: std::io::Error) -> AppError {
    eprintln!("Backup I/O: {error}");
    AppError {
        code: "backup",
        message: "Could not access the backup file. Check its location and free space.".into(),
    }
}
fn copy(source: &Connection, destination: &mut Connection) -> Result<()> {
    Backup::new(source, destination)?.run_to_completion(128, Duration::from_millis(1), None)?;
    Ok(())
}
fn validate(conn: &Connection) -> Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version != 3 && version != 4 {
        return Err(AppError::validation(
            "Backup schema is not supported by this Forma version.",
        ));
    }
    // Compare the full released schema, rejecting extra triggers/views and missing constraints.
    let reference = Connection::open_in_memory()?;
    reference.execute_batch(include_str!("../migrations/001_initial.sql"))?;
    reference.execute_batch(include_str!("../migrations/002_issues.sql"))?;
    reference.execute_batch(include_str!("../migrations/003_productivity.sql"))?;
    if version == 4 {
        reference.execute_batch(include_str!("../migrations/004_git.sql"))?;
    }
    fn schema(c: &Connection) -> Result<Vec<(String, String)>> {
        Ok(c.prepare(
            "SELECT name,sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY name",
        )?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<std::result::Result<_, _>>()?)
    }
    if schema(conn)? != schema(&reference)? {
        return Err(AppError::validation(
            "Backup schema does not match the supported Forma format.",
        ));
    }
    let format: i64 = conn.query_row(
        "SELECT format_version FROM backup_metadata WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    if format != (if version == 3 { 1 } else { 2 }) {
        return Err(AppError::validation("Unsupported backup format version."));
    }
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity != "ok" || conn.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(AppError::validation(
            "Backup integrity or foreign-key validation failed.",
        ));
    }
    Ok(())
}
impl Service {
    pub fn export_backup(&self, path: &Path) -> Result<()> {
        if let Some(live) = self.repo.conn.path().filter(|p| !p.is_empty()) {
            let selected = path.canonicalize().unwrap_or_else(|_| path.to_owned());
            let live = Path::new(live).canonicalize().map_err(io)?;
            if selected == live
                || ["-wal", "-shm"].iter().any(|suffix| {
                    let mut sidecar = live.as_os_str().to_os_string();
                    sidecar.push(suffix);
                    selected.as_os_str() == sidecar
                })
            {
                return Err(AppError::validation(
                    "Choose a destination outside the active database files.",
                ));
            }
        }
        let parent = path
            .parent()
            .ok_or_else(|| AppError::validation("Choose a backup destination."))?;
        let staging = tempfile::NamedTempFile::new_in(parent).map_err(io)?;
        let mut destination = Connection::open(staging.path())?;
        copy(&self.repo.conn, &mut destination)?;
        destination.execute("INSERT OR REPLACE INTO backup_metadata(singleton,format_version,created_at) VALUES(1,2,?1)", [chrono::Utc::now().to_rfc3339()])?;
        destination.pragma_update(None, "journal_mode", "DELETE")?;
        validate(&destination)?;
        destination.close().map_err(|(_, e)| AppError::from(e))?;
        staging.as_file().sync_all().map_err(io)?;
        staging.persist(path).map_err(|e| io(e.error))?;
        Ok(())
    }
    /// Stage a private snapshot so changing the selected file after confirmation cannot
    /// change the data restored. No archive extraction or filenames inside the backup.
    pub fn prepare_restore(&mut self, path: &Path) -> Result<()> {
        self.pending_restore = None;
        let source = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        source.execute_batch("BEGIN")?;
        validate(&source)?;
        let staging = tempfile::NamedTempFile::new().map_err(io)?;
        let mut destination = Connection::open(staging.path())?;
        copy(&source, &mut destination)?;
        destination.pragma_update(None, "journal_mode", "DELETE")?;
        let version: i64 = destination.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version == 3 {
            let tx = destination.transaction()?;
            tx.execute_batch(include_str!("../migrations/004_git.sql"))?;
            tx.pragma_update(None, "user_version", 4)?;
            tx.execute("UPDATE backup_metadata SET format_version=2", [])?;
            tx.commit()?;
        }
        // Paths in imported data never authorize filesystem access, even when a
        // malicious backup sets trusted=1. Only native folder selection reauthorizes.
        destination.execute("UPDATE git_repositories SET trusted=0", [])?;
        validate(&destination)?;
        drop(destination);
        self.pending_restore = Some(staging);
        Ok(())
    }
    pub fn cancel_restore(&mut self) {
        self.pending_restore = None;
    }
    pub fn restore_backup(&mut self, confirmed: bool) -> Result<String> {
        if !confirmed {
            return Err(AppError::validation(
                "Confirm replacement of all local data.",
            ));
        }
        let staging = self
            .pending_restore
            .take()
            .ok_or_else(|| AppError::validation("Select and validate a backup first."))?;
        let source = Connection::open_with_flags(staging.path(), OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        validate(&source)?;
        let live_path = self
            .repo
            .conn
            .path()
            .filter(|p| !p.is_empty())
            .ok_or_else(|| AppError::validation("Restore requires a file database."))?
            .to_owned();
        let recovery = Path::new(&live_path).with_file_name(format!(
            "forma-recovery-{}.forma-backup",
            uuid::Uuid::new_v4()
        ));
        self.export_backup(&recovery)?;
        // SQLite backup destination writes are a single transaction: interrupted or
        // failed copying rolls back; a crash recovers the previous WAL transaction.
        copy(&source, &mut self.repo.conn)?;
        match Repository::open(Path::new(&live_path)) {
            Ok(repository) => self.repo = repository,
            Err(error) => {
                let previous =
                    Connection::open_with_flags(&recovery, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
                copy(&previous, &mut self.repo.conn)?;
                return Err(error);
            }
        }
        Ok(recovery.to_string_lossy().into_owned())
    }
}
