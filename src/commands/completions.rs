use crate::cli::{Cli, CompletionShell};
use anyhow::{anyhow, Result};
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::{env, fs, io};

pub fn run(shell_opt: Option<CompletionShell>, install: bool) -> Result<()> {
    let mut command = Cli::command();

    let completion_shell = match shell_opt {
        Some(s) => s,
        None => detect_shell()?,
    };

    let clap_shell = match completion_shell {
        CompletionShell::Bash => Shell::Bash,
        CompletionShell::Zsh => Shell::Zsh,
        CompletionShell::Fish => Shell::Fish,
    };

    if install {
        install_completions(&mut command, clap_shell, completion_shell)?;
    } else {
        let mut stdout = io::stdout();
        generate(clap_shell, &mut command, "envx", &mut stdout);
    }

    Ok(())
}

fn detect_shell() -> Result<CompletionShell> {
    let shell_path = env::var("SHELL").unwrap_or_default();
    if shell_path.ends_with("bash") {
        Ok(CompletionShell::Bash)
    } else if shell_path.ends_with("zsh") {
        Ok(CompletionShell::Zsh)
    } else if shell_path.ends_with("fish") {
        Ok(CompletionShell::Fish)
    } else {
        Err(anyhow!(
            "Could not auto-detect shell from $SHELL ({}). Please specify the shell explicitly.",
            if shell_path.is_empty() { "not set" } else { &shell_path }
        ))
    }
}

fn install_completions(
    command: &mut clap::Command,
    clap_shell: Shell,
    completion_shell: CompletionShell,
) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    
    let path = match completion_shell {
        CompletionShell::Bash => {
            home.join(".local/share/bash-completion/completions/envx")
        }
        CompletionShell::Zsh => {
            home.join(".local/share/zsh/site-functions/_envx")
        }
        CompletionShell::Fish => {
            home.join(".config/fish/completions/envx.fish")
        }
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(&path)?;
    generate(clap_shell, command, "envx", &mut file);

    println!("Successfully installed {} completions to: {}", clap_shell, path.display());
    
    if matches!(completion_shell, CompletionShell::Zsh) {
        println!("\nNote: For Zsh, ensure that the following is in your ~/.zshrc before compinit:");
        println!("fpath=(~/.local/share/zsh/site-functions $fpath)");
    }

    Ok(())
}
