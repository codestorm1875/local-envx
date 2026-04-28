use crate::audit::append_event;
use crate::store::{env_path, load_env, save_env};
use anyhow::Result;

pub fn run(key: &str) -> Result<()> {
    let path = env_path();
    crate::store::check_env_for_write(&path)?;
    let mut document = load_env(&path)?;

    if !document.remove(key) {
        eprintln!("Warning: key {key} was not present");
        return Ok(());
    }

    save_env(path, &document)?;
    append_event(&format!("DELETE {key}"))?;
    println!("Deleted {key}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::store::EnvDocument;

    #[test]
    fn test_delete_existing_key() {
        let mut doc = EnvDocument::parse("A=1\nB=2\nC=3").unwrap();
        assert!(doc.remove("B"));
        assert_eq!(doc.get("B"), None);
        assert_eq!(doc.get("A"), Some("1"));
        assert_eq!(doc.get("C"), Some("3"));
    }

    #[test]
    fn test_delete_nonexistent_key_returns_false() {
        let mut doc = EnvDocument::parse("A=1").unwrap();
        assert!(!doc.remove("MISSING"));
    }

    #[test]
    fn test_delete_removes_all_duplicates() {
        let mut doc = EnvDocument::parse("KEY=1\nKEY=2\nKEY=3").unwrap();
        assert!(doc.remove("KEY"));
        assert_eq!(doc.get("KEY"), None);
        assert_eq!(doc.keys().filter(|k| *k == "KEY").count(), 0);
    }

    #[test]
    fn test_delete_last_key_leaves_empty() {
        let mut doc = EnvDocument::parse("ONLY=value").unwrap();
        assert!(doc.remove("ONLY"));
        assert_eq!(doc.keys().count(), 0);
    }
}
