use crate::error::EnvxError;
use anyhow::{Context, Result, bail};
use std::fmt::{self, Write as _};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Blank,
    Comment(String),
    Entry { key: String, value: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvDocument {
    lines: Vec<Line>,
}

impl EnvDocument {
    pub fn parse(input: &str) -> Result<Self> {
        let mut lines = Vec::new();

        for raw_line in input.lines() {
            let line = raw_line.trim_end();

            if line.trim().is_empty() {
                lines.push(Line::Blank);
                continue;
            }

            if line.trim_start().starts_with('#') {
                lines.push(Line::Comment(line.to_owned()));
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .with_context(|| format!("invalid env line `{line}`"))?;
            let key = key.trim();
            let value = value.trim_start().to_owned();

            validate_key(key)?;
            lines.push(Line::Entry {
                key: key.to_owned(),
                value,
            });
        }

        Ok(Self { lines })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.lines.iter().rev().find_map(|line| match line {
            Line::Entry {
                key: current_key,
                value,
            } if current_key == key => Some(value.as_str()),
            _ => None,
        })
    }

    pub fn set(&mut self, key: &str, value: impl Into<String>) -> Result<()> {
        validate_key(key)?;
        let value = value.into();

        if let Some(existing) = self.lines.iter_mut().find_map(|line| match line {
            Line::Entry {
                key: current_key,
                value: current_value,
            } if current_key == key => Some(current_value),
            _ => None,
        }) {
            *existing = value;
            self.remove_duplicates(key);
            return Ok(());
        }

        self.lines.push(Line::Entry {
            key: key.to_owned(),
            value,
        });
        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let original_len = self.lines.len();
        self.lines.retain(|line| match line {
            Line::Entry {
                key: current_key, ..
            } => current_key != key,
            _ => true,
        });
        original_len != self.lines.len()
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().filter_map(|line| match line {
            Line::Entry { key, .. } => Some(key.as_str()),
            _ => None,
        })
    }

    /// Merge another document into this one, with the other document taking precedence.
    pub fn merge(&mut self, other: &EnvDocument) {
        for key in other.keys() {
            if let Some(value) = other.get(key) {
                let _ = self.set(key, value);
            }
        }
    }

    /// Expand variables in values using the format ${VAR_NAME}.
    /// Falls back to $VAR_NAME for simple variable names.
    pub fn expand_variables(&self) -> Result<EnvDocument> {
        let mut expanded = self.clone();

        for line in &mut expanded.lines {
            if let Line::Entry { key, value } = line {
                let resolved = self.expand_value(value, &mut vec![key.clone()])?;
                *value = resolved;
            }
        }

        Ok(expanded)
    }

    fn expand_value(&self, value: &str, stack: &mut Vec<String>) -> Result<String> {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume '{'
                    let mut var_name = String::new();
                    let mut closed = false;

                    while let Some(&ch) = chars.peek() {
                        if ch == '}' {
                            chars.next();
                            closed = true;
                            break;
                        }

                        var_name.push(ch);
                        chars.next();
                    }

                    if !closed {
                        bail!("unterminated variable reference: ${{{var_name}}}");
                    }

                    result.push_str(&self.resolve_reference(&var_name, stack)?);
                } else {
                    let mut var_name = String::new();

                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            var_name.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    if !var_name.is_empty() {
                        result.push_str(&self.resolve_reference(&var_name, stack)?);
                    } else {
                        result.push('$');
                    }
                }
            } else {
                result.push(c);
            }
        }

        Ok(result)
    }

    fn resolve_reference(&self, name: &str, stack: &mut Vec<String>) -> Result<String> {
        if stack.iter().any(|current| current == name) {
            stack.push(name.to_owned());
            bail!("cyclic variable reference detected: {}", stack.join(" -> "));
        }

        let value = self
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("undefined variable: {name}"))?;

        stack.push(name.to_owned());
        let expanded = self.expand_value(value, stack);
        stack.pop();
        expanded
    }

    pub fn to_text(&self) -> String {
        let mut output = String::new();

        for (index, line) in self.lines.iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }

            match line {
                Line::Blank => {}
                Line::Comment(text) => output.push_str(text),
                Line::Entry { key, value } => {
                    let _ = write!(&mut output, "{key}={value}");
                }
            }
        }

        output
    }

    fn remove_duplicates(&mut self, key: &str) {
        let mut seen = false;

        self.lines.retain(|line| match line {
            Line::Entry {
                key: current_key, ..
            } if current_key == key => {
                if seen {
                    false
                } else {
                    seen = true;
                    true
                }
            }
            _ => true,
        });
    }
}

pub fn env_path() -> PathBuf {
    PathBuf::from(".env")
}

pub fn load_env(path: impl AsRef<Path>) -> Result<EnvDocument> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(EnvxError::MissingEnvFile.into());
    }

    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    EnvDocument::parse(&contents)
}

/// Smartly load an environment file.
/// If the target file doesn't exist, it looks for an encrypted `.age` version
/// of the file, automatically decrypts it in-memory (prompting if necessary),
/// and returns the parsed document.
pub fn load_env_smart_read(path: impl AsRef<Path>) -> Result<EnvDocument> {
    let path = path.as_ref();

    if path.exists() {
        return load_env(path);
    }

    // Determine the path to the .age equivalent.
    // If path is ".env", this becomes ".env.age".
    // If path is ".env.production", this becomes ".env.production.age" (since rust sees 'production' as the extension).
    // The safest way to just append .age is to use string format:
    let mut path_str = path.to_string_lossy().to_string();
    path_str.push_str(".age");
    let age_path = PathBuf::from(path_str);

    if age_path.exists() {
        eprintln!(
            "⚠ `{}` not found, but `{}` exists. Decrypting in memory...",
            path.display(),
            age_path.display()
        );

        let encrypted = fs::read(&age_path)?;
        let is_passphrase = crate::crypto::is_passphrase_encrypted(&encrypted)?;

        let decrypted_bytes = if is_passphrase {
            let passphrase = crate::crypto::get_passphrase_for_read()?;
            crate::crypto::decrypt_file_with_passphrase(&encrypted, &passphrase)?
        } else {
            let identity = crate::crypto::load_or_generate_identity()?;
            crate::crypto::decrypt_file(&encrypted, &identity)?
        };

        let contents = String::from_utf8(decrypted_bytes)
            .map_err(|_| anyhow::anyhow!("Decrypted data is not valid UTF-8"))?;
        return EnvDocument::parse(&contents);
    }

    Err(EnvxError::MissingEnvFile.into())
}

/// Guard function for write operations (set, delete, merge).
/// Checks if we're trying to write to a missing `.env` when `.env.age` exists.
pub fn check_env_for_write(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    if path.exists() {
        return Ok(());
    }

    let mut path_str = path.to_string_lossy().to_string();
    path_str.push_str(".age");
    let age_path = PathBuf::from(path_str);

    if age_path.exists() {
        bail!(
            "Cannot modify `{}` because it is missing and `{}` exists.\nPlease run `envx decrypt` first before modifying the file.",
            path.display(),
            age_path.display()
        );
    }

    Ok(())
}

pub fn save_env(path: impl AsRef<Path>, document: &EnvDocument) -> Result<()> {
    let path = path.as_ref();
    fs::write(path, document.to_text())
        .with_context(|| format!("failed to write {}", path.display()))
}

pub fn ensure_env_file(path: impl AsRef<Path>) -> Result<bool> {
    let path = path.as_ref();

    if path.exists() {
        return Ok(false);
    }

    fs::write(path, "").with_context(|| format!("failed to create {}", path.display()))?;
    Ok(true)
}

pub fn ensure_gitignore_has_env(path: impl AsRef<Path>) -> Result<bool> {
    let path = path.as_ref();
    let mut contents = if path.exists() {
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?
    } else {
        String::new()
    };

    if contents.lines().any(|line| line.trim() == ".env") {
        return Ok(false);
    }

    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(".env\n");

    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(true)
}

pub fn validate_key(key: &str) -> Result<()> {
    let mut chars = key.chars();

    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => bail!(EnvxError::InvalidKey(key.to_owned())),
    }

    if chars.all(|character| character.is_ascii_alphanumeric() || character == '_') {
        Ok(())
    } else {
        bail!(EnvxError::InvalidKey(key.to_owned()))
    }
}

impl fmt::Display for EnvDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let doc = EnvDocument::parse("").unwrap();
        assert_eq!(doc.keys().count(), 0);
    }

    #[test]
    fn test_parse_single_key_value() {
        let doc = EnvDocument::parse("KEY=value").unwrap();
        assert_eq!(doc.get("KEY"), Some("value"));
    }

    #[test]
    fn test_parse_multiple_entries() {
        let content = "A=1\nB=2\nC=3";
        let doc = EnvDocument::parse(content).unwrap();
        assert_eq!(doc.get("A"), Some("1"));
        assert_eq!(doc.get("B"), Some("2"));
        assert_eq!(doc.get("C"), Some("3"));
    }

    #[test]
    fn test_parse_with_comments() {
        let content = "# Comment\nKEY=value";
        let doc = EnvDocument::parse(content).unwrap();
        assert_eq!(doc.get("KEY"), Some("value"));
        assert_eq!(doc.keys().count(), 1);
    }

    #[test]
    fn test_parse_with_blank_lines() {
        let content = "KEY1=val1\n\nKEY2=val2";
        let doc = EnvDocument::parse(content).unwrap();
        assert_eq!(doc.get("KEY1"), Some("val1"));
        assert_eq!(doc.get("KEY2"), Some("val2"));
    }

    #[test]
    fn test_parse_empty_value() {
        let doc = EnvDocument::parse("KEY=").unwrap();
        assert_eq!(doc.get("KEY"), Some(""));
    }

    #[test]
    fn test_parse_value_with_spaces() {
        let doc = EnvDocument::parse("KEY= value with spaces").unwrap();
        assert_eq!(doc.get("KEY"), Some("value with spaces"));
    }

    #[test]
    fn test_parse_invalid_key_no_equals() {
        let result = EnvDocument::parse("INVALID_LINE");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_key_starts_with_number() {
        let result = EnvDocument::parse("9KEY=value");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_valid_keys_with_underscore() {
        let doc = EnvDocument::parse("_KEY=value\nKEY_NAME=value2").unwrap();
        assert_eq!(doc.get("_KEY"), Some("value"));
        assert_eq!(doc.get("KEY_NAME"), Some("value2"));
    }

    #[test]
    fn test_set_new_key() {
        let mut doc = EnvDocument::default();
        doc.set("KEY", "value").unwrap();
        assert_eq!(doc.get("KEY"), Some("value"));
    }

    #[test]
    fn test_set_overwrites_existing() {
        let mut doc = EnvDocument::parse("KEY=old").unwrap();
        doc.set("KEY", "new").unwrap();
        assert_eq!(doc.get("KEY"), Some("new"));
    }

    #[test]
    fn test_set_removes_duplicates() {
        let mut doc = EnvDocument::parse("KEY=1\nKEY=2\nKEY=3").unwrap();
        doc.set("KEY", "final").unwrap();
        assert_eq!(doc.keys().filter(|k| *k == "KEY").count(), 1);
        assert_eq!(doc.get("KEY"), Some("final"));
    }

    #[test]
    fn test_remove_existing_key() {
        let mut doc = EnvDocument::parse("KEY=value\nOTHER=data").unwrap();
        assert!(doc.remove("KEY"));
        assert_eq!(doc.get("KEY"), None);
        assert_eq!(doc.get("OTHER"), Some("data"));
    }

    #[test]
    fn test_remove_nonexistent_key() {
        let mut doc = EnvDocument::parse("KEY=value").unwrap();
        assert!(!doc.remove("NOTFOUND"));
    }

    #[test]
    fn test_get_returns_last_value_on_duplicate() {
        let doc = EnvDocument::parse("KEY=first\nKEY=second").unwrap();
        assert_eq!(doc.get("KEY"), Some("second"));
    }

    #[test]
    fn test_to_text_preserves_format() {
        let content = "# Header\nKEY1=val1\n\nKEY2=val2";
        let doc = EnvDocument::parse(content).unwrap();
        let text = doc.to_text();
        assert!(text.contains("# Header"));
        assert!(text.contains("KEY1=val1"));
        assert!(text.contains("KEY2=val2"));
    }

    #[test]
    fn test_expand_variables_resolves_nested_references() {
        let doc = EnvDocument::parse("A=hello\nB=${A} world\nC=$B!").unwrap();
        let expanded = doc.expand_variables().unwrap();

        assert_eq!(expanded.get("A"), Some("hello"));
        assert_eq!(expanded.get("B"), Some("hello world"));
        assert_eq!(expanded.get("C"), Some("hello world!"));
    }

    #[test]
    fn test_expand_variables_reports_cycles() {
        let doc = EnvDocument::parse("A=$B\nB=$A").unwrap();
        let result = doc.expand_variables();

        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        assert!(message.contains("cyclic variable reference"));
    }

    #[test]
    fn test_validate_key_valid() {
        assert!(validate_key("KEY").is_ok());
        assert!(validate_key("_KEY").is_ok());
        assert!(validate_key("KEY_NAME").is_ok());
        assert!(validate_key("KEY123").is_ok());
    }

    #[test]
    fn test_validate_key_invalid_starts_with_number() {
        assert!(validate_key("9KEY").is_err());
    }

    #[test]
    fn test_validate_key_invalid_special_chars() {
        assert!(validate_key("KEY-NAME").is_err());
        assert!(validate_key("KEY.NAME").is_err());
        assert!(validate_key("KEY NAME").is_err());
    }

    #[test]
    fn test_validate_key_empty() {
        assert!(validate_key("").is_err());
    }
}
