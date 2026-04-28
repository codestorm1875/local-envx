use crate::store::EnvDocument;
use anyhow::Result;
use std::path::Path;

pub fn run() -> Result<()> {
    let example_path = Path::new(".env.example");
    let env_path = Path::new(".env");

    let example = crate::store::load_env_smart_read(example_path)?;
    let current = crate::store::load_env_smart_read(env_path)?;

    check_keys(&example, &current)
}

fn check_keys(example: &EnvDocument, current: &EnvDocument) -> Result<()> {
    let missing: Vec<&str> = example
        .keys()
        .filter(|key| current.get(key).is_none())
        .collect();

    if missing.is_empty() {
        println!("✓ All required keys are present in .env");
        Ok(())
    } else {
        eprintln!(
            "✗ Missing {} key{}:",
            missing.len(),
            if missing.len() == 1 { "" } else { "s" }
        );
        for key in &missing {
            eprintln!("  - {key}");
        }
        Err(anyhow::anyhow!(
            "validation failed: {} missing key{}",
            missing.len(),
            if missing.len() == 1 { "" } else { "s" }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_all_keys_present() {
        let example = EnvDocument::parse("A=\nB=\nC=").unwrap();
        let current = EnvDocument::parse("A=1\nB=2\nC=3").unwrap();
        assert!(check_keys(&example, &current).is_ok());
    }

    #[test]
    fn test_check_missing_single_key() {
        let example = EnvDocument::parse("A=\nB=\nC=").unwrap();
        let current = EnvDocument::parse("A=1\nB=2").unwrap();
        let result = check_keys(&example, &current);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("1 missing key"));
    }

    #[test]
    fn test_check_missing_multiple_keys() {
        let example = EnvDocument::parse("A=\nB=\nC=").unwrap();
        let current = EnvDocument::parse("A=1").unwrap();
        let result = check_keys(&example, &current);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("2 missing keys"));
    }

    #[test]
    fn test_check_extra_keys_in_current_ok() {
        let example = EnvDocument::parse("A=").unwrap();
        let current = EnvDocument::parse("A=1\nEXTRA=2").unwrap();
        assert!(check_keys(&example, &current).is_ok());
    }

    #[test]
    fn test_check_both_empty() {
        let example = EnvDocument::parse("").unwrap();
        let current = EnvDocument::parse("").unwrap();
        assert!(check_keys(&example, &current).is_ok());
    }
}
