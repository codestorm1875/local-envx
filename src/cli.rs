use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Parser)]
#[command(name = "envx", version, about = "Local-first .env manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new .env file and supporting project metadata.
    Init,
    /// List keys from the current .env file.
    List {
        /// Reveal values instead of masking them.
        #[arg(long)]
        reveal: bool,
    },
    /// Read the value for a single key.
    Get {
        /// The key to look up.
        key: String,
    },
    /// Add or update a key in .env.
    Set {
        /// The key to write.
        key: String,
        /// The value to store.
        value: String,
    },
    /// Remove a key from .env.
    Delete {
        /// The key to remove.
        key: String,
    },
    /// Validate .env against .env.example.
    Check,
    /// Compare two .env files and show differences (keys only).
    Diff {
        /// Path to the first .env file.
        file1: String,
        /// Path to the second .env file.
        file2: String,
    },
    /// Interactive TUI viewer for .env files.
    View,
    /// Encrypt .env to .env.age using age encryption.
    Encrypt {
        /// Public key for recipient-based encryption (for team sharing).
        #[arg(long)]
        recipient: Option<String>,
        /// Delete plaintext .env after encryption.
        #[arg(long)]
        delete: bool,
    },
    /// Decrypt .env.age back to .env.
    Decrypt {
        /// Use passphrase-based decryption instead of identity key.
        #[arg(long)]
        passphrase: bool,
    },
    /// Merge .env with .env.local (or another file) — local values take precedence.
    Merge {
        /// Path to the file to merge in (default: .env.local).
        #[arg(default_value = ".env.local")]
        file: String,
        /// Output file (if not specified, updates .env in-place).
        #[arg(long)]
        output: Option<String>,
    },
    /// Expand variables in .env values (supports ${VAR} and $VAR syntax).
    Expand {
        /// Input file (default: .env).
        #[arg(default_value = ".env")]
        file: String,
        /// Output file (if not specified, prints to stdout).
        #[arg(long)]
        output: Option<String>,
    },
    /// Generate or install shell completion scripts.
    Completions {
        /// Shell to generate completions for. If omitted, attempts to auto-detect from $SHELL.
        #[arg(value_enum)]
        shell: Option<CompletionShell>,

        /// Automatically install the completion script for the detected shell.
        #[arg(long)]
        install: bool,
    },
}
