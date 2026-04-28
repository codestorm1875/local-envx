use anyhow::Result;
use std::fs;
use std::path::PathBuf;


pub fn run(file: String, output: Option<String>) -> Result<()> {
    let env_path = PathBuf::from(&file);
    let env_doc = crate::store::load_env_smart_read(&env_path)?;

    let expanded = env_doc.expand_variables()?;
    let text = expanded.to_text();

    if let Some(out) = output {
        fs::write(&out, &text)?;
        println!("✓ Variables expanded and written to {}", out);
    } else {
        println!("{}", text);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::store::EnvDocument;

    #[test]
    fn test_expand_simple_reference() {
        let doc = EnvDocument::parse("HOST=localhost\nURL=http://$HOST").unwrap();
        let expanded = doc.expand_variables().unwrap();
        assert_eq!(expanded.get("URL"), Some("http://localhost"));
    }

    #[test]
    fn test_expand_braced_reference() {
        let doc = EnvDocument::parse("HOST=localhost\nPORT=5432\nURL=postgres://${HOST}:${PORT}/db").unwrap();
        let expanded = doc.expand_variables().unwrap();
        assert_eq!(expanded.get("URL"), Some("postgres://localhost:5432/db"));
    }

    #[test]
    fn test_expand_chained_references() {
        let doc = EnvDocument::parse("A=hello\nB=${A}_world\nC=${B}_foo").unwrap();
        let expanded = doc.expand_variables().unwrap();
        assert_eq!(expanded.get("C"), Some("hello_world_foo"));
    }

    #[test]
    fn test_expand_no_references() {
        let doc = EnvDocument::parse("KEY=plain_value").unwrap();
        let expanded = doc.expand_variables().unwrap();
        assert_eq!(expanded.get("KEY"), Some("plain_value"));
    }

    #[test]
    fn test_expand_cyclic_reference_fails() {
        let doc = EnvDocument::parse("A=${B}\nB=${A}").unwrap();
        let result = doc.expand_variables();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cyclic"));
    }

    #[test]
    fn test_expand_undefined_variable_fails() {
        let doc = EnvDocument::parse("URL=http://${MISSING}").unwrap();
        let result = doc.expand_variables();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("undefined"));
    }
}
