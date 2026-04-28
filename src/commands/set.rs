use crate::audit::append_event;
use crate::store::{env_path, load_env, save_env, validate_key};
use anyhow::Result;

pub fn run(key: &str, value: &str) -> Result<()> {
    validate_key(key)?;

    let path = env_path();
    crate::store::check_env_for_write(&path)?;

    let mut document = if path.exists() {
        load_env(&path)?
    } else {
        crate::store::EnvDocument::default()
    };

    let duplicates_before = document.keys().filter(|existing| *existing == key).count();

    document.set(key, value.to_owned())?;
    save_env(path, &document)?;

    if duplicates_before > 1 {
        let removed = duplicates_before - 1;
        eprintln!(
            "Warning: removed {removed} duplicate entr{suffix} for {key}",
            suffix = if removed == 1 { "y" } else { "ies" }
        );
    }

    append_event(&format!("SET {key}"))?;

    println!("Set {key}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::store::EnvDocument;

    #[test]
    fn test_set_new_key_in_document() {
        let mut doc = EnvDocument::default();
        doc.set("NEW_KEY", "value").unwrap();
        assert_eq!(doc.get("NEW_KEY"), Some("value"));
    }

    #[test]
    fn test_set_updates_existing_key() {
        let mut doc = EnvDocument::parse("KEY=old").unwrap();
        doc.set("KEY", "new").unwrap();
        assert_eq!(doc.get("KEY"), Some("new"));
    }

    #[test]
    fn test_set_deduplicates() {
        let mut doc = EnvDocument::parse("KEY=1\nKEY=2\nKEY=3").unwrap();
        doc.set("KEY", "final").unwrap();
        assert_eq!(doc.keys().filter(|k| *k == "KEY").count(), 1);
        assert_eq!(doc.get("KEY"), Some("final"));
    }

    #[test]
    fn test_set_invalid_key_rejected() {
        use crate::store::validate_key;
        assert!(validate_key("9INVALID").is_err());
        assert!(validate_key("VALID_KEY").is_ok());
    }

    #[test]
    fn test_set_empty_value() {
        let mut doc = EnvDocument::default();
        doc.set("KEY", "").unwrap();
        assert_eq!(doc.get("KEY"), Some(""));
    }
}
