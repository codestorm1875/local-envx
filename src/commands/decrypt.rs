use anyhow::{Context, Result, bail};
use std::fs;
use std::io::{self, IsTerminal, Write};

use crate::audit::append_event;
use crate::crypto;
use crate::store::env_path;

pub fn run(passphrase_mode: bool) -> Result<()> {
    let env_file = env_path();
    let encrypted_file = std::path::PathBuf::from(".env.age");

    if !encrypted_file.exists() {
        bail!("`.env.age` file not found. Run `envx encrypt` first.");
    }

    let encrypted = fs::read(&encrypted_file)?;

    let is_passphrase = crypto::is_passphrase_encrypted(&encrypted)?;
    let actual_passphrase_mode = passphrase_mode || is_passphrase;

    let decrypted = if actual_passphrase_mode {
        let passphrase = get_passphrase()?;
        crypto::decrypt_file_with_passphrase(&encrypted, &passphrase)?
    } else {
        let identity = crypto::load_or_generate_identity()?;
        crypto::decrypt_file(&encrypted, &identity)?
    };

    if env_file.exists() {
        if io::stdin().is_terminal() {
            eprint!("⚠ `.env` already exists. Overwrite? (yes/no): ");
            io::stderr().flush()?;

            let mut response = String::new();
            io::stdin().read_line(&mut response)?;

            if !response.trim().eq_ignore_ascii_case("yes") {
                println!("Aborted: `.env` was not overwritten.");
                return Ok(());
            }
        } else {
            bail!("`.env` already exists and cannot prompt in non-interactive mode")
        }
    }

    fs::write(&env_file, decrypted)?;
    println!(
        "✓ Decrypted {} → {}",
        encrypted_file.display(),
        env_file.display()
    );
    append_event("DECRYPT")?;

    Ok(())
}

fn get_passphrase() -> Result<String> {
    if io::stdin().is_terminal() {
        eprint!("Enter passphrase to decrypt: ");
        io::stderr().flush()?;
        rpassword::read_password().context("Failed to read passphrase")
    } else {
        bail!("Cannot read passphrase in non-interactive mode")
    }
}
