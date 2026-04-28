use crate::cli::{Cli, CompletionShell};
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use std::io;

pub fn run(shell: CompletionShell) -> Result<()> {
    let mut command = Cli::command();
    let mut stdout = io::stdout();
    let shell = match shell {
        CompletionShell::Bash => Shell::Bash,
        CompletionShell::Zsh => Shell::Zsh,
        CompletionShell::Fish => Shell::Fish,
    };

    generate(shell, &mut command, "envx", &mut stdout);
    Ok(())
}
