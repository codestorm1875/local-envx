use crate::audit::append_event;
use crate::crypto;
use crate::store::{ensure_env_file, ensure_gitignore_has_env, env_path};
use anyhow::{Context, Result};
use std::io::{self, IsTerminal, Write};
use std::path::Path;

pub fn run() -> Result<()> {
    let env_created = ensure_env_file(env_path())?;
    let gitignore_updated = ensure_gitignore_has_env(Path::new(".gitignore"))?;

    if env_created {
        println!("Created .env");
    } else {
        println!(".env already exists");
    }

    if gitignore_updated {
        println!("Added .env to .gitignore");
    }

    let identity_path = crypto::identity_path()?;
    if identity_path.exists() {
        println!("Age keypair already exists at {}", identity_path.display());
    } else {
        let identity = crypto::generate_and_save_identity()?;
        let public_key = identity.to_public().to_string();
        let keys_dir = identity_path.parent().context("identity path has no parent")?;
        println!("Generated age keypair at {}", keys_dir.display());
        println!("  Public key: {public_key}");

        if io::stdin().is_terminal() {
            println!();
            println!("How would you like to encrypt .env files?");
            println!("  1) Keypair-based (recommended for personal use)");
            println!("  2) Passphrase-based (recommended for team sharing)");
            print!("Choose [1/2] (default: 1): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim() {
                "2" => println!("Passphrase mode selected. Use `envx encrypt` without --recipient."),
                _ => println!("Keypair mode selected. Use `envx encrypt --recipient <pubkey>`."),
            }
        }
    }

    append_event("INIT")?;

    Ok(())
}
