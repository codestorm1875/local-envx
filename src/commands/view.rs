use anyhow::Result;

use crate::store::env_path;
use crate::tui;

pub fn run() -> Result<()> {
    let document = crate::store::load_env_smart_read(env_path())?;
    tui::run(&document)
}
