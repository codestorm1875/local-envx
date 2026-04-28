use age::x25519;
use anyhow::{Context, Result, bail};
use secrecy::{ExposeSecret, Secret};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

pub fn keys_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("envx")
        .join("keys");

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

pub fn identity_path() -> Result<PathBuf> {
    Ok(keys_dir()?.join("identity.txt"))
}

pub fn generate_and_save_identity() -> Result<x25519::Identity> {
    let identity = x25519::Identity::generate();
    let dir = keys_dir()?;

    let identity_path = dir.join("identity.txt");
    let secret_str = identity.to_string();
    fs::write(&identity_path, secret_str.expose_secret().as_bytes())?;

    let public_path = dir.join("public.txt");
    fs::write(&public_path, identity.to_public().to_string())?;

    Ok(identity)
}

pub fn load_or_generate_identity() -> Result<x25519::Identity> {
    let path = identity_path()?;

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        x25519::Identity::from_str(content.trim())
            .map_err(|_| anyhow::anyhow!("Failed to parse stored identity"))
    } else {
        generate_and_save_identity()
    }
}

pub fn encrypt_file(plaintext_path: &str, recipient: &x25519::Recipient) -> Result<Vec<u8>> {
    let plaintext = fs::read(plaintext_path)?;

    let encryptor = age::Encryptor::with_recipients(vec![Box::new(recipient.clone())])
        .context("Failed to create encryptor")?;

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(&plaintext)?;
    writer.finish()?;

    Ok(encrypted)
}

pub fn encrypt_file_with_passphrase(plaintext_path: &str, passphrase: &str) -> Result<Vec<u8>> {
    let plaintext = fs::read(plaintext_path)?;

    let encryptor = age::Encryptor::with_user_passphrase(Secret::new(passphrase.to_owned()));

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(&plaintext)?;
    writer.finish()?;

    Ok(encrypted)
}

pub fn decrypt_file(encrypted: &[u8], identity: &x25519::Identity) -> Result<Vec<u8>> {
    let decryptor =
        age::Decryptor::new_buffered(encrypted).context("Failed to read encrypted data")?;

    let decrypted = match decryptor {
        age::Decryptor::Recipients(rd) => {
            let mut reader = rd.decrypt(std::iter::once(identity as &dyn age::Identity))?;
            let mut decrypted = vec![];
            reader.read_to_end(&mut decrypted)?;
            decrypted
        }
        age::Decryptor::Passphrase(_) => {
            bail!("Encrypted file requires passphrase, but identity was provided")
        }
    };

    Ok(decrypted)
}

pub fn decrypt_file_with_passphrase(encrypted: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let decryptor =
        age::Decryptor::new_buffered(encrypted).context("Failed to read encrypted data")?;

    let decrypted = match decryptor {
        age::Decryptor::Passphrase(pd) => {
            let mut reader = pd.decrypt(&Secret::new(passphrase.to_owned()), None)?;
            let mut decrypted = vec![];
            reader.read_to_end(&mut decrypted)?;
            decrypted
        }
        age::Decryptor::Recipients(_) => {
            bail!("Encrypted file requires identity key, but passphrase was provided")
        }
    };

    Ok(decrypted)
}

pub fn is_passphrase_encrypted(encrypted: &[u8]) -> Result<bool> {
    let decryptor =
        age::Decryptor::new_buffered(encrypted).context("Failed to read encrypted data")?;
    Ok(matches!(decryptor, age::Decryptor::Passphrase(_)))
}

pub fn get_passphrase_for_read() -> Result<String> {
    use std::io::{self, IsTerminal, Write};
    if io::stdin().is_terminal() {
        eprint!("Enter passphrase to decrypt: ");
        io::stderr().flush()?;
        rpassword::read_password().context("Failed to read passphrase")
    } else {
        bail!("Cannot read passphrase in non-interactive mode")
    }
}

/// Encrypt raw bytes using a recipient public key (in-memory, for testing).
fn encrypt_bytes(plaintext: &[u8], recipient: &x25519::Recipient) -> Result<Vec<u8>> {
    let encryptor = age::Encryptor::with_recipients(vec![Box::new(recipient.clone())])
        .context("Failed to create encryptor")?;

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(plaintext)?;
    writer.finish()?;

    Ok(encrypted)
}

/// Encrypt raw bytes using a passphrase (in-memory, for testing).
fn encrypt_bytes_with_passphrase(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let encryptor = age::Encryptor::with_user_passphrase(Secret::new(passphrase.to_owned()));

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(plaintext)?;
    writer.finish()?;

    Ok(encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipient_encrypt_decrypt_roundtrip() {
        let identity = x25519::Identity::generate();
        let recipient = identity.to_public();

        let plaintext = b"DATABASE_URL=postgres://localhost/app\nAPI_KEY=secret123";
        let encrypted = encrypt_bytes(plaintext, &recipient).unwrap();

        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt_file(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_passphrase_encrypt_decrypt_roundtrip() {
        let passphrase = "test-passphrase-123";
        let plaintext = b"SECRET=hello_world";

        let encrypted = encrypt_bytes_with_passphrase(plaintext, passphrase).unwrap();
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt_file_with_passphrase(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_recipient_with_passphrase_fails() {
        let identity = x25519::Identity::generate();
        let recipient = identity.to_public();

        let plaintext = b"KEY=value";
        let encrypted = encrypt_bytes(plaintext, &recipient).unwrap();

        let result = decrypt_file_with_passphrase(&encrypted, "any-passphrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_passphrase_with_identity_fails() {
        let passphrase = "test-pass";
        let plaintext = b"KEY=value";
        let encrypted = encrypt_bytes_with_passphrase(plaintext, passphrase).unwrap();

        let identity = x25519::Identity::generate();
        let result = decrypt_file(&encrypted, &identity);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_corrupted_data_fails() {
        let identity = x25519::Identity::generate();
        let result = decrypt_file(b"this is not encrypted data", &identity);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_empty_data_fails() {
        let identity = x25519::Identity::generate();
        let result = decrypt_file(b"", &identity);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_plaintext() {
        let identity = x25519::Identity::generate();
        let recipient = identity.to_public();

        let encrypted = encrypt_bytes(b"", &recipient).unwrap();
        let decrypted = decrypt_file(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_wrong_recipient_cannot_decrypt() {
        let identity1 = x25519::Identity::generate();
        let identity2 = x25519::Identity::generate();
        let recipient1 = identity1.to_public();

        let plaintext = b"SECRET=data";
        let encrypted = encrypt_bytes(plaintext, &recipient1).unwrap();

        let result = decrypt_file(&encrypted, &identity2);
        assert!(result.is_err());
    }
}
