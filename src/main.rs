use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::{error::Error, io};

enum InputMode {
    Normal,
    CommitMessage,
    PRTitle,
}

#[derive(Default)]
struct FileItem {
    name: String,
    selected: bool,
}

struct App {
    files: Vec<FileItem>,
    files_state: ListState,
    commit_message: String,
    pr_title: String,
    input_mode: InputMode,
    should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        // Mock data - in real app, this would come from Git
        let files = vec![
            FileItem {
                name: "untracked.rs".to_string(),
                selected: false,
            },
            FileItem {
                name: "modified.rs".to_string(),
                selected: false,
            },
            FileItem {
                name: "staged.rs".to_string(),
                selected: false,
            },
        ];

        App {
            files,
            files_state: ListState::default(),
            commit_message: String::new(),
            pr_title: String::new(),
            input_mode: InputMode::Normal,
            should_quit: false,
        }
    }
}

impl App {
    fn next(&mut self) {
        let i = match self.files_state.selected() {
            Some(i) => (i + 1) % self.files.len(),
            None => 0,
        };
        self.files_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.files_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.files_state.select(Some(i));
    }

    fn toggle_selected(&mut self) {
        if let Some(i) = self.files_state.selected() {
            self.files[i].selected = !self.files[i].selected;
        }
    }

    fn create_commit_and_pr(&self) {
        // Stub function - would implement actual Git operations here
        println!("Would create commit with message: {}", self.commit_message);
        println!("Would create PR with title: {}", self.pr_title);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::default();
    app.files_state.select(Some(0));

    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                return Ok(());
            }
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('j') => app.next(),
                    KeyCode::Char('k') => app.previous(),
                    KeyCode::Char(' ') => app.toggle_selected(),
                    KeyCode::Char('c') => {
                        app.input_mode = InputMode::CommitMessage;
                    }
                    KeyCode::Char('p') => {
                        app.input_mode = InputMode::PRTitle;
                    }
                    KeyCode::Enter => {
                        app.create_commit_and_pr();
                    }
                    _ => {}
                },
                InputMode::CommitMessage => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        app.commit_message.push(c);
                    }
                    KeyCode::Backspace => {
                        app.commit_message.pop();
                    }
                    _ => {}
                },
                InputMode::PRTitle => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        app.pr_title.push(c);
                    }
                    KeyCode::Backspace => {
                        app.pr_title.pop();
                    }
                    _ => {}
                },
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Files list
    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|file| {
            let prefix = if file.selected { "[x] " } else { "[ ] " };
            ListItem::new(format!("{}{}", prefix, file.name))
        })
        .collect();

    let files_list = List::new(items)
        .block(Block::default().title("Files").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(files_list, chunks[0], &mut app.files_state);

    // Commit message input
    let commit_block = Block::default()
        .title("Commit Message (press 'c' to edit)")
        .borders(Borders::ALL);
    let commit_text = Paragraph::new(app.commit_message.as_str())
        .block(commit_block)
        .style(match app.input_mode {
            InputMode::CommitMessage => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(commit_text, chunks[1]);

    // PR title input
    let pr_block = Block::default()
        .title("PR Title (press 'p' to edit)")
        .borders(Borders::ALL);
    let pr_text =
        Paragraph::new(app.pr_title.as_str())
            .block(pr_block)
            .style(match app.input_mode {
                InputMode::PRTitle => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            });
    f.render_widget(pr_text, chunks[2]);
}
