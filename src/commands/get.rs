use crate::error::EnvxError;
use crate::store::env_path;
use anyhow::{Result, bail};

pub fn run(key: &str) -> Result<()> {
    let document = crate::store::load_env_smart_read(env_path())?;

    if let Some(value) = document.get(key) {
        println!("{value}");
        return Ok(());
    }

    bail!(EnvxError::KeyNotFound(key.to_owned()))
}

#[cfg(test)]
mod tests {
    use crate::store::EnvDocument;

    #[test]
    fn test_get_existing_key() {
        let doc = EnvDocument::parse("DATABASE_URL=postgres://localhost/app").unwrap();
        assert_eq!(doc.get("DATABASE_URL"), Some("postgres://localhost/app"));
    }

    #[test]
    fn test_get_missing_key() {
        let doc = EnvDocument::parse("KEY=value").unwrap();
        assert_eq!(doc.get("MISSING"), None);
    }

    #[test]
    fn test_get_empty_value() {
        let doc = EnvDocument::parse("KEY=").unwrap();
        assert_eq!(doc.get("KEY"), Some(""));
    }

    #[test]
    fn test_get_last_duplicate_wins() {
        let doc = EnvDocument::parse("KEY=first\nKEY=second").unwrap();
        assert_eq!(doc.get("KEY"), Some("second"));
    }
}
