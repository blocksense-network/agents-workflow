//! Main TUI application logic

use aw_rest_client::RestClient;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use tokio::sync::Mutex;

use crate::error::TuiResult;
use crate::event::{Event, EventHandler};
use crate::ui;

/// Application state
#[derive(Debug)]
pub struct AppState {
    pub current_section: usize,
    pub project_index: usize,
    pub branch_index: usize,
    pub agent_index: usize,
    pub task_description: String,
    pub loading: bool,
    pub error: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_section: 0,
            project_index: 0,
            branch_index: 0,
            agent_index: 0,
            task_description: String::new(),
            loading: false,
            error: None,
        }
    }
}

/// Main TUI application
pub struct App {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    event_handler: EventHandler,
    rest_client: Option<RestClient>,
    state: Mutex<AppState>,
}

impl App {
    /// Create a new TUI application
    pub fn new(rest_client: Option<RestClient>) -> TuiResult<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let event_handler = EventHandler::new();

        Ok(Self {
            terminal,
            event_handler,
            rest_client,
            state: Mutex::new(AppState::default()),
        })
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> TuiResult<()> {
        // Start event handling
        self.event_handler.run().await;

        loop {
            // Draw the UI
            self.terminal.draw(|f| {
                let size = f.size();
                let state = self.state.try_lock().unwrap();

                if let Some(error) = &state.error {
                    ui::draw_error(f, size, error);
                } else if state.loading {
                    ui::draw_loading(f, size, "Creating task...");
                } else {
                    // Create list states for UI
                    let mut project_state = ratatui::widgets::ListState::default();
                    project_state.select(Some(state.project_index));

                    let mut branch_state = ratatui::widgets::ListState::default();
                    branch_state.select(Some(state.branch_index));

                    let mut agent_state = ratatui::widgets::ListState::default();
                    agent_state.select(Some(state.agent_index));

                    ui::draw_dashboard(
                        f,
                        size,
                        &mut project_state,
                        &mut branch_state,
                        &mut agent_state,
                        &state.task_description,
                    );
                }
            })?;

            // Handle events
            match self.event_handler.next().await {
                Some(Event::Quit) => break,
                Some(Event::Input(event)) => {
                    self.handle_input(event).await?;
                }
                Some(Event::Tick) => {
                    // Periodic updates can go here
                }
                Some(Event::Error(e)) => {
                    let mut state = self.state.lock().await;
                    state.error = Some(format!("Event error: {}", e));
                }
                None => break,
            }
        }

        Ok(())
    }

    /// Handle user input events
    async fn handle_input(&self, event: crossterm::event::Event) -> TuiResult<()> {
        use crossterm::event::{Event, KeyCode, KeyEvent};

        let mut state = self.state.lock().await;

        match event {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Tab => {
                    state.current_section = (state.current_section + 1) % 4;
                }
                KeyCode::BackTab => {
                    state.current_section = if state.current_section == 0 {
                        3
                    } else {
                        state.current_section - 1
                    };
                }
                KeyCode::Up => match state.current_section {
                    0 => {
                        if state.project_index > 0 {
                            state.project_index -= 1;
                        }
                    }
                    1 => {
                        if state.branch_index > 0 {
                            state.branch_index -= 1;
                        }
                    }
                    2 => {
                        if state.agent_index > 0 {
                            state.agent_index -= 1;
                        }
                    }
                    _ => {}
                },
                KeyCode::Down => match state.current_section {
                    0 => {
                        state.project_index += 1;
                    }
                    1 => {
                        state.branch_index += 1;
                    }
                    2 => {
                        state.agent_index += 1;
                    }
                    _ => {}
                },
                KeyCode::Char(c) => {
                    if state.current_section == 3 {
                        // Task description editing
                        if c == '\n' {
                            state.task_description.push('\n');
                        } else if c == '\x08' || c == '\x7f' {
                            // Backspace
                            state.task_description.pop();
                        } else {
                            state.task_description.push(c);
                        }
                    }
                }
                KeyCode::Enter => {
                    if state.current_section == 3 && !state.task_description.trim().is_empty() {
                        // Create task
                        self.create_task(&mut state).await?;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    /// Create a task using the REST client
    async fn create_task(&self, state: &mut AppState) -> TuiResult<()> {
        if self.rest_client.is_none() {
            state.error = Some("No REST client configured".to_string());
            return Ok(());
        }

        state.loading = true;
        state.error = None;

        // This would create the actual task - simplified for now
        // In a real implementation, this would use the REST client
        // to create a task with the selected project, branch, agent, and description

        // Simulate task creation delay
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        state.loading = false;
        state.task_description.clear();

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Cleanup terminal
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen
        );
        let _ = self.terminal.show_cursor();
    }
}
