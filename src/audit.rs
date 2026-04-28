use anyhow::{Context, Result};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub fn append_event(event: &str) -> Result<()> {
    let path = audit_log_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let timestamp = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("failed to format audit timestamp")?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;

    writeln!(file, "[{timestamp}] {event}")
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

fn audit_log_path() -> Result<PathBuf> {
    let home = env::var_os("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("envx")
        .join("audit.log"))
}
