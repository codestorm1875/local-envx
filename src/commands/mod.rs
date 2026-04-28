use crate::cli::Command;
use anyhow::Result;

mod check;
mod completions;
mod decrypt;
mod delete;
mod diff;
mod encrypt;
mod expand;
mod get;
mod init;
mod list;
mod merge;
mod set;
mod view;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Init => init::run(),
        Command::List { reveal } => list::run(reveal),
        Command::Get { key } => get::run(&key),
        Command::Set { key, value } => set::run(&key, &value),
        Command::Delete { key } => delete::run(&key),
        Command::Check => check::run(),
        Command::Diff { file1, file2 } => diff::run(&file1, &file2),
        Command::View => view::run(),
        Command::Encrypt { recipient, delete } => encrypt::run(recipient, delete),
        Command::Decrypt { passphrase } => decrypt::run(passphrase),
        Command::Merge { file, output } => merge::run(file, output),
        Command::Expand { file, output } => expand::run(file, output),
        Command::Completions { shell } => completions::run(shell),
    }
}
