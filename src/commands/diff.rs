use crate::store::EnvDocument;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

pub fn run(file1_path: &str, file2_path: &str) -> Result<()> {
    let file1 = crate::store::load_env_smart_read(Path::new(file1_path))?;
    let file2 = crate::store::load_env_smart_read(Path::new(file2_path))?;

    let result = compute_diff(&file1, &file2);
    print_diff(&result);

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct DiffResult {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
}

pub fn compute_diff(file1: &EnvDocument, file2: &EnvDocument) -> DiffResult {
    let keys1: BTreeSet<&str> = file1.keys().collect();
    let keys2: BTreeSet<&str> = file2.keys().collect();

    let added: Vec<String> = keys2.difference(&keys1).map(|k| k.to_string()).collect();
    let removed: Vec<String> = keys1.difference(&keys2).map(|k| k.to_string()).collect();
    let common: BTreeSet<&str> = keys1.intersection(&keys2).copied().collect();

    let mut changed = Vec::new();
    for key in common {
        if file1.get(key) != file2.get(key) {
            changed.push(key.to_string());
        }
    }

    DiffResult {
        added,
        removed,
        changed,
    }
}

fn print_diff(result: &DiffResult) {
    let mut has_output = false;

    if !result.added.is_empty() {
        println!("Added keys:");
        for key in &result.added {
            println!("  + {key}");
        }
        has_output = true;
    }

    if !result.removed.is_empty() {
        if has_output {
            println!();
        }
        println!("Removed keys:");
        for key in &result.removed {
            println!("  - {key}");
        }
        has_output = true;
    }

    if !result.changed.is_empty() {
        if has_output {
            println!();
        }
        println!("Changed keys:");
        for key in &result.changed {
            println!("  ~ {key}");
        }
    }

    if result.added.is_empty() && result.removed.is_empty() && result.changed.is_empty() {
        println!("No differences found");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identical_files() {
        let doc1 = EnvDocument::parse("A=1\nB=2").unwrap();
        let doc2 = EnvDocument::parse("A=1\nB=2").unwrap();
        let result = compute_diff(&doc1, &doc2);
        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert!(result.changed.is_empty());
    }

    #[test]
    fn test_diff_added_keys() {
        let doc1 = EnvDocument::parse("A=1").unwrap();
        let doc2 = EnvDocument::parse("A=1\nB=2").unwrap();
        let result = compute_diff(&doc1, &doc2);
        assert_eq!(result.added, vec!["B"]);
        assert!(result.removed.is_empty());
        assert!(result.changed.is_empty());
    }

    #[test]
    fn test_diff_removed_keys() {
        let doc1 = EnvDocument::parse("A=1\nB=2").unwrap();
        let doc2 = EnvDocument::parse("A=1").unwrap();
        let result = compute_diff(&doc1, &doc2);
        assert!(result.added.is_empty());
        assert_eq!(result.removed, vec!["B"]);
        assert!(result.changed.is_empty());
    }

    #[test]
    fn test_diff_changed_keys() {
        let doc1 = EnvDocument::parse("A=1\nB=old").unwrap();
        let doc2 = EnvDocument::parse("A=1\nB=new").unwrap();
        let result = compute_diff(&doc1, &doc2);
        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert_eq!(result.changed, vec!["B"]);
    }

    #[test]
    fn test_diff_mixed_changes() {
        let doc1 = EnvDocument::parse("A=1\nB=2\nC=3").unwrap();
        let doc2 = EnvDocument::parse("B=changed\nC=3\nD=4").unwrap();
        let result = compute_diff(&doc1, &doc2);
        assert_eq!(result.added, vec!["D"]);
        assert_eq!(result.removed, vec!["A"]);
        assert_eq!(result.changed, vec!["B"]);
    }

    #[test]
    fn test_diff_both_empty() {
        let doc1 = EnvDocument::parse("").unwrap();
        let doc2 = EnvDocument::parse("").unwrap();
        let result = compute_diff(&doc1, &doc2);
        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert!(result.changed.is_empty());
    }

    #[test]
    fn test_diff_never_exposes_values() {
        // The DiffResult only contains key names, never values
        let doc1 = EnvDocument::parse("SECRET=password123").unwrap();
        let doc2 = EnvDocument::parse("SECRET=newpassword456").unwrap();
        let result = compute_diff(&doc1, &doc2);
        assert_eq!(result.changed, vec!["SECRET"]);
        let debug_output = format!("{:?}", result);
        assert!(!debug_output.contains("password123"));
        assert!(!debug_output.contains("newpassword456"));
    }
}
