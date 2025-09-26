//! Main TUI application logic

use aw_rest_api_contract::{AgentCapability, Project, Repository};
use aw_rest_client::RestClient;
use aw_client_api::ClientApi;
use crossterm::{
    event::KeyModifiers,
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

    // REST API data
    pub projects: Vec<Project>,
    pub repositories: Vec<Repository>,
    pub agents: Vec<AgentCapability>,

    // Loading states for each data source
    pub projects_loading: bool,
    pub repositories_loading: bool,
    pub agents_loading: bool,

    // Filter text for each selector
    pub project_filter: String,
    pub branch_filter: String,
    pub agent_filter: String,

    // Editor height for resizing
    pub editor_height: usize,
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

            // REST API data
            projects: Vec::new(),
            repositories: Vec::new(),
            agents: Vec::new(),

            // Loading states
            projects_loading: false,
            repositories_loading: false,
            agents_loading: false,

            // Filter text
            project_filter: String::new(),
            branch_filter: String::new(),
            agent_filter: String::new(),

            // Editor height
            editor_height: 5,
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

    /// Load initial data from REST API
    pub async fn load_initial_data(&self) -> TuiResult<()> {
        if let Some(client) = &self.rest_client {
            let mut state = self.state.lock().await;

            // Load projects
            state.projects_loading = true;
            match ClientApi::list_projects(client, None).await {
                Ok(projects) => {
                    state.projects = projects;
                    state.projects_loading = false;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to load projects: {}", e));
                    state.projects_loading = false;
                }
            }

            // Load repositories
            state.repositories_loading = true;
            match ClientApi::list_repositories(client, None, None).await {
                Ok(repositories) => {
                    state.repositories = repositories;
                    state.repositories_loading = false;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to load repositories: {}", e));
                    state.repositories_loading = false;
                }
            }

            // Load agents
            state.agents_loading = true;
            match ClientApi::list_agents(client).await {
                Ok(agents) => {
                    state.agents = agents;
                    state.agents_loading = false;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to load agents: {}", e));
                    state.agents_loading = false;
                }
            }
        }

        Ok(())
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> TuiResult<()> {
        // Start event handling
        self.event_handler.run().await;

        // Load initial data
        if let Err(e) = self.load_initial_data().await {
            let mut state = self.state.lock().await;
            state.error = Some(format!("Failed to load initial data: {}", e));
        }

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
                    // Create ViewModel from current state
                    let view_model = crate::ViewModel::from_state(&state);

                    // Create list states for UI
                    let mut project_state = ratatui::widgets::ListState::default();
                    let mut branch_state = ratatui::widgets::ListState::default();
                    let mut agent_state = ratatui::widgets::ListState::default();

                    ui::draw_dashboard(
                        f,
                        size,
                        &view_model,
                        &mut project_state,
                        &mut branch_state,
                        &mut agent_state,
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
            Event::Key(KeyEvent { code, modifiers, .. }) => match code {
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
                KeyCode::Up => {
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        // Ctrl+Up: resize editor smaller
                        if state.current_section == 3 && state.editor_height > 3 {
                            state.editor_height -= 1;
                        }
                    } else {
                        // Regular Up: navigate in current section
                        match state.current_section {
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
                        }
                    }
                }
                KeyCode::Down => {
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        // Ctrl+Down: resize editor larger
                        if state.current_section == 3 && state.editor_height < 20 {
                            state.editor_height += 1;
                        }
                    } else {
                        // Regular Down: navigate in current section
                        match state.current_section {
                            0 => {
                                let max_index = if state.project_filter.is_empty() {
                                    state.projects.len().saturating_sub(1)
                                } else {
                                    state.projects.iter()
                                        .filter(|p| p.display_name.to_lowercase().contains(&state.project_filter.to_lowercase()))
                                        .count()
                                        .saturating_sub(1)
                                };
                                if state.project_index < max_index {
                                    state.project_index += 1;
                                }
                            }
                            1 => {
                                let max_index = if state.branch_filter.is_empty() {
                                    state.repositories.len().saturating_sub(1)
                                } else {
                                    state.repositories.iter()
                                        .filter(|r| r.display_name.to_lowercase().contains(&state.branch_filter.to_lowercase()))
                                        .count()
                                        .saturating_sub(1)
                                };
                                if state.branch_index < max_index {
                                    state.branch_index += 1;
                                }
                            }
                            2 => {
                                let max_index = if state.agent_filter.is_empty() {
                                    state.agents.len().saturating_sub(1)
                                } else {
                                    state.agents.iter()
                                        .filter(|a| a.agent_type.to_lowercase().contains(&state.agent_filter.to_lowercase()))
                                        .count()
                                        .saturating_sub(1)
                                };
                                if state.agent_index < max_index {
                                    state.agent_index += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Char(c) => {
                    match state.current_section {
                        0 => {
                            // Project filter
                            if c == '\x08' || c == '\x7f' {
                                state.project_filter.pop();
                                state.project_index = 0;
                            } else if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                                state.project_filter.push(c);
                                state.project_index = 0;
                            }
                        }
                        1 => {
                            // Branch/repository filter
                            if c == '\x08' || c == '\x7f' {
                                state.branch_filter.pop();
                                state.branch_index = 0;
                            } else if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '/' {
                                state.branch_filter.push(c);
                                state.branch_index = 0;
                            }
                        }
                        2 => {
                            // Agent filter
                            if c == '\x08' || c == '\x7f' {
                                state.agent_filter.pop();
                                state.agent_index = 0;
                            } else if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                                state.agent_filter.push(c);
                                state.agent_index = 0;
                            }
                        }
                        3 => {
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
                        _ => {}
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
