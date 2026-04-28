use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use std::io;

use crate::store::EnvDocument;

pub struct TuiApp {
    entries: Vec<(String, String)>,
    selected: usize,
    reveal: bool,
    search_query: String,
    search_mode: bool,
    filtered_indices: Vec<usize>,
}

impl TuiApp {
    pub fn new(document: &EnvDocument) -> Self {
        let entries: Vec<(String, String)> = document
            .keys()
            .filter_map(|key| {
                document
                    .get(key)
                    .map(|value| (key.to_string(), value.to_string()))
            })
            .collect();

        let filtered_indices: Vec<usize> = (0..entries.len()).collect();

        Self {
            entries,
            selected: 0,
            reveal: false,
            search_query: String::new(),
            search_mode: false,
            filtered_indices,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let res = self.event_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        res
    }

    fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key(key) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.entries.len()).collect();
            return;
        }

        let matcher = SkimMatcherV2::default();
        self.filtered_indices = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(idx, (key, _))| matcher.fuzzy_match(key, &self.search_query).map(|_| idx))
            .collect();

        self.selected = 0;
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.search_mode {
            match key.code {
                KeyCode::Esc => {
                    self.search_mode = false;
                    self.search_query.clear();
                    self.apply_filter();
                    false
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.apply_filter();
                    false
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.apply_filter();
                    false
                }
                KeyCode::Enter => {
                    self.search_mode = false;
                    false
                }
                _ => false,
            }
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => true,
                KeyCode::Char('r') => {
                    self.reveal = !self.reveal;
                    false
                }
                KeyCode::Char('/') => {
                    self.search_mode = true;
                    self.search_query.clear();
                    false
                }
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    false
                }
                KeyCode::Down => {
                    let max = self.filtered_indices.len().saturating_sub(1);
                    if self.selected < max {
                        self.selected += 1;
                    }
                    false
                }
                KeyCode::Home => {
                    self.selected = 0;
                    false
                }
                KeyCode::End => {
                    self.selected = self.filtered_indices.len().saturating_sub(1);
                    false
                }
                KeyCode::PageUp => {
                    self.selected = self.selected.saturating_sub(10);
                    false
                }
                KeyCode::PageDown => {
                    let max = self.filtered_indices.len().saturating_sub(1);
                    self.selected = (self.selected + 10).min(max);
                    false
                }
                _ => false,
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let size = f.area();

        // Title and help
        let title_block = Block::default()
            .title(" envx • Local-first .env Manager ")
            .borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT | Borders::RIGHT);

        let help_text = if self.search_query.is_empty() {
            "Press [q] or [ESC] to quit • [r] to reveal values • [/] to search • [↑↓] to scroll"
        } else {
            "Searching... • Press [ENTER] or [ESC] to close search"
        };

        let title_paragraph = Paragraph::new(help_text)
            .block(title_block)
            .style(Style::default().fg(Color::Gray));

        let title_area = Rect {
            x: size.x,
            y: size.y,
            width: size.width,
            height: 3,
        };
        f.render_widget(title_paragraph, title_area);

        let list_height = if self.search_mode {
            size.height.saturating_sub(6)
        } else {
            size.height.saturating_sub(3)
        };
        let list_area = Rect {
            x: size.x,
            y: size.y + 3,
            width: size.width,
            height: list_height,
        };

        // Render filtered entries
        let items: Vec<ListItem> = self
            .filtered_indices
            .iter()
            .enumerate()
            .map(|(idx, &entry_idx)| {
                let (key, value) = &self.entries[entry_idx];
                let display_value = if self.reveal {
                    value.clone()
                } else {
                    "***".to_string()
                };
                let content = format!("{:<30} = {}", key, display_value);
                let style = if idx == self.selected {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };
                ListItem::new(content).style(style)
            })
            .collect();

        let filtered_count = self.filtered_indices.len();
        let visible_pct = if filtered_count == 0 {
            0
        } else {
            (self.selected + 1) * 100 / filtered_count
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(
                        " {} of {} entries • {} ({}% visible) ",
                        filtered_count,
                        self.entries.len(),
                        if self.reveal { "REVEALED" } else { "MASKED" },
                        visible_pct
                    ))
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(list, list_area);

        // Render search box at bottom if in search mode
        if self.search_mode {
            let search_area = Rect {
                x: size.x,
                y: size.y + size.height.saturating_sub(2),
                width: size.width,
                height: 2,
            };

            let search_text = format!("Search: {}", self.search_query);
            let search_box = Paragraph::new(search_text)
                .block(Block::default().borders(Borders::ALL).title(" Find "))
                .style(Style::default().fg(Color::Cyan));

            f.render_widget(search_box, search_area);
        }
    }
}

pub fn run(document: &EnvDocument) -> Result<()> {
    let mut app = TuiApp::new(document);
    app.run()
}
