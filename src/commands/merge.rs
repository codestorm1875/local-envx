use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::audit::append_event;
use crate::store::{env_path, load_env, save_env};

pub fn run(file: String, output: Option<String>) -> Result<()> {
    let env_file = env_path();
    let merge_file = PathBuf::from(&file);

    crate::store::check_env_for_write(&env_file)?;
    if !env_file.exists() {
        bail!("`.env` file not found. Run `envx init` first.");
    }

    if !merge_file.exists() {
        bail!("File to merge not found: {}", merge_file.display());
    }

    let mut env_doc = load_env(&env_file)?;
    let merge_doc = crate::store::load_env_smart_read(&merge_file)?;

    env_doc.merge(&merge_doc);

    let output_path = if let Some(out) = output {
        PathBuf::from(out)
    } else {
        env_file
    };

    save_env(&output_path, &env_doc)?;

    let merged_count = merge_doc.keys().count();
    println!(
        "✓ Merged {} entries from {} into {}",
        merged_count,
        merge_file.display(),
        output_path.display()
    );

    append_event(&format!("MERGE {}", merge_file.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::store::EnvDocument;

    #[test]
    fn test_merge_adds_new_keys() {
        let mut base = EnvDocument::parse("A=1").unwrap();
        let overlay = EnvDocument::parse("B=2").unwrap();
        base.merge(&overlay);
        assert_eq!(base.get("A"), Some("1"));
        assert_eq!(base.get("B"), Some("2"));
    }

    #[test]
    fn test_merge_overlay_takes_precedence() {
        let mut base = EnvDocument::parse("A=old\nB=keep").unwrap();
        let overlay = EnvDocument::parse("A=new").unwrap();
        base.merge(&overlay);
        assert_eq!(base.get("A"), Some("new"));
        assert_eq!(base.get("B"), Some("keep"));
    }

    #[test]
    fn test_merge_with_empty_overlay() {
        let mut base = EnvDocument::parse("A=1\nB=2").unwrap();
        let overlay = EnvDocument::parse("").unwrap();
        base.merge(&overlay);
        assert_eq!(base.get("A"), Some("1"));
        assert_eq!(base.get("B"), Some("2"));
    }

    #[test]
    fn test_merge_into_empty_base() {
        let mut base = EnvDocument::parse("").unwrap();
        let overlay = EnvDocument::parse("A=1\nB=2").unwrap();
        base.merge(&overlay);
        assert_eq!(base.get("A"), Some("1"));
        assert_eq!(base.get("B"), Some("2"));
    }

    #[test]
    fn test_merge_multiple_overlapping_keys() {
        let mut base = EnvDocument::parse("A=1\nB=2\nC=3").unwrap();
        let overlay = EnvDocument::parse("B=new_b\nC=new_c\nD=4").unwrap();
        base.merge(&overlay);
        assert_eq!(base.get("A"), Some("1"));
        assert_eq!(base.get("B"), Some("new_b"));
        assert_eq!(base.get("C"), Some("new_c"));
        assert_eq!(base.get("D"), Some("4"));
    }
}
