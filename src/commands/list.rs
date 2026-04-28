use crate::store::env_path;
use anyhow::Result;
use std::io::{self, IsTerminal, Write};

pub fn run(reveal: bool) -> Result<()> {
    if reveal && io::stdin().is_terminal() {
        if !confirm_reveal()? {
            println!("Aborted: values remain masked.");
            return Ok(());
        }
    }

    let document = crate::store::load_env_smart_read(env_path())?;

    if io::stdout().is_terminal() {
        render_tui_table(&document, reveal)?;
    } else {
        for key in document.keys() {
            if reveal {
                if let Some(value) = document.get(key) {
                    println!("{key}={value}");
                }
            } else {
                println!("{key}=***");
            }
        }
    }

    Ok(())
}

fn confirm_reveal() -> Result<bool> {
    print!("Reveal all values? Type 'yes' to continue: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim() == "yes")
}

fn render_tui_table(
    document: &crate::store::EnvDocument,
    reveal: bool,
) -> Result<()> {
    use crossterm::execute;
    use crossterm::terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    };
    use crossterm::event::{self, Event, KeyCode};
    use ratatui::prelude::*;
    use ratatui::widgets::{Block, Borders, Cell, Row, Table};

    let rows: Vec<Row> = document
        .keys()
        .filter_map(|key| {
            let display_value = if reveal {
                document.get(key).unwrap_or("").to_string()
            } else {
                "***".to_string()
            };
            Some(Row::new(vec![
                Cell::from(key.to_string()).style(Style::default().fg(Color::Cyan)),
                Cell::from(display_value),
            ]))
        })
        .collect();

    let entry_count = rows.len();

    let table = Table::new(
        rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .header(
        Row::new(vec![
            Cell::from("Key").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Cell::from("Value").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ])
        .bottom_margin(1),
    )
    .block(
        Block::default()
            .title(format!(
                " envx • {} entr{} • {} ",
                entry_count,
                if entry_count == 1 { "y" } else { "ies" },
                if reveal { "REVEALED" } else { "MASKED" },
            ))
            .borders(Borders::ALL),
    );

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            f.render_widget(&table, f.area());
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
