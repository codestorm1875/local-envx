use anyhow::{Context, Result, bail};
use std::fs;
use std::io::{self, IsTerminal, Write};

use crate::audit::append_event;
use crate::crypto;
use crate::store::env_path;

pub fn run(recipient_key: Option<String>, delete_after: bool) -> Result<()> {
    let env_file = env_path();

    if !env_file.exists() {
        bail!("`.env` file not found. Run `envx init` first.");
    }

    let encrypted = if let Some(pubkey_str) = recipient_key {
        let recipient: age::x25519::Recipient = pubkey_str
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("Failed to parse recipient public key"))?;

        crypto::encrypt_file(env_file.to_str().context("Invalid path")?, &recipient)?
    } else {
        eprintln!("No recipient specified. Using passphrase encryption.");
        let passphrase = get_passphrase()?;
        crypto::encrypt_file_with_passphrase(
            env_file.to_str().context("Invalid path")?,
            &passphrase,
        )?
    };

    let encrypted_file = std::path::PathBuf::from(".env.age");
    fs::write(&encrypted_file, encrypted)?;

    println!(
        "✓ Encrypted {} → {}",
        env_file.display(),
        encrypted_file.display()
    );

    if delete_after {
        fs::remove_file(&env_file)?;
        println!("✓ Deleted plaintext {}", env_file.display());
        append_event("ENCRYPT (deleted plaintext)")?;
    } else {
        println!("⚠ Plaintext .env still exists. Use `rm .env` to delete it, or run `envx encrypt --delete` next time.");
        append_event("ENCRYPT")?;
    }

    Ok(())
}

fn get_passphrase() -> Result<String> {
    if io::stdin().is_terminal() {
        eprint!("Enter passphrase (will not echo): ");
        io::stderr().flush()?;
        let passphrase = rpassword::read_password()?;

        eprint!("Confirm passphrase: ");
        io::stderr().flush()?;
        let confirm = rpassword::read_password()?;

        if passphrase != confirm {
            bail!("Passphrases do not match");
        }

        Ok(passphrase)
    } else {
        bail!("Cannot read passphrase in non-interactive mode")
    }
}
