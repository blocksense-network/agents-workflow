//! The Model layer - domain state and rules (no I/O, no Ratatui)
//!
//! This implements the state machine core that processes messages
//! and updates the application state.

use crate::app::AppState;
use crate::msg::{Msg, NetMsg, RestSuccessMsg};
use aw_client_api::ClientApi;
use aw_rest_api_contract::{AgentCapability, CreateTaskRequest, Project, Repository};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;

/// The Model represents the domain state and business logic
/// It processes messages and updates state deterministically
pub struct Model<C: ClientApi> {
    pub state: AppState,
    client: Arc<C>,
}

impl<C: ClientApi> Model<C> {
    /// Create a new model with initial state
    pub fn new(client: Arc<C>) -> Self {
        Self {
            state: AppState::default(),
            client,
        }
    }

    /// Process a single message and update state
    ///
    /// This is the core state machine method that handles all external stimuli.
    /// It's designed to be deterministic and side-effect free (no I/O).
    pub async fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Key(key_event) => self.handle_key(key_event),
            Msg::Tick => self.handle_tick(),
            Msg::Net(net_msg) => self.handle_net_msg(net_msg).await,
            Msg::Quit => {
                // Quit is handled at the application level
            }
        }
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Tab => {
                self.state.current_section = (self.state.current_section + 1) % 4;
            }
            KeyCode::BackTab => {
                self.state.current_section = if self.state.current_section == 0 {
                    3
                } else {
                    self.state.current_section - 1
                };
            }
            KeyCode::Up => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+Up: resize editor smaller
                    if self.state.current_section == 3 && self.state.editor_height > 3 {
                        self.state.editor_height -= 1;
                    }
                } else {
                    // Regular Up: navigate in current section
                    self.handle_navigation_up();
                }
            }
            KeyCode::Down => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+Down: resize editor larger
                    if self.state.current_section == 3 && self.state.editor_height < 20 {
                        self.state.editor_height += 1;
                    }
                } else {
                    // Regular Down: navigate in current section
                    self.handle_navigation_down();
                }
            }
            KeyCode::Char(c) => {
                self.handle_char_input(c, key_event.modifiers);
            }
            KeyCode::Enter => {
                if self.state.current_section == 3 && !self.state.task_description.trim().is_empty() {
                    // Create task - this would trigger a Net message in real implementation
                    // For now, just simulate the loading state
                    self.state.loading = true;
                    self.state.error = None;
                }
            }
            KeyCode::Backspace => {
                self.handle_backspace();
            }
            _ => {}
        }
    }

    /// Handle time tick
    fn handle_tick(&mut self) {
        // Periodic updates can be handled here
        // For example, polling for status updates, etc.
    }

    /// Handle network messages
    async fn handle_net_msg(&mut self, net_msg: NetMsg) {
        match net_msg {
            NetMsg::Sse(session_event) => {
                // Handle SSE events (status updates, logs, etc.)
                self.handle_sse_event(session_event);
            }
            NetMsg::RestSuccess(success_msg) => {
                // Handle successful REST responses
                self.handle_rest_success(success_msg);
            }
            NetMsg::RestError(error) => {
                // Handle REST errors
                self.state.error = Some(error);
                self.state.loading = false;
                self.reset_loading_states();
            }
        }
    }

    /// Handle SSE events
    fn handle_sse_event(&mut self, _event: aw_rest_api_contract::SessionEvent) {
        // Handle different types of SSE events
        // This would update status, logs, progress, etc.
    }

    /// Handle successful REST responses
    fn handle_rest_success(&mut self, success_msg: RestSuccessMsg) {
        match success_msg {
            RestSuccessMsg::ProjectsList(projects) => {
                self.state.projects = projects;
                self.state.projects_loading = false;
            }
            RestSuccessMsg::RepositoriesList(repositories) => {
                self.state.repositories = repositories;
                self.state.repositories_loading = false;
            }
            RestSuccessMsg::AgentsList(agents) => {
                self.state.agents = agents;
                self.state.agents_loading = false;
            }
            RestSuccessMsg::TaskCreated(_response) => {
                // Task created successfully
                self.state.loading = false;
                self.state.task_description.clear();
                // Could show success message or navigate to monitoring view
            }
        }
    }

    /// Handle navigation up in current section
    fn handle_navigation_up(&mut self) {
        match self.state.current_section {
            0 => {
                // Projects section
                if self.state.project_index > 0 {
                    self.state.project_index -= 1;
                }
            }
            1 => {
                // Repositories section
                if self.state.branch_index > 0 {
                    self.state.branch_index -= 1;
                }
            }
            2 => {
                // Agents section
                if self.state.agent_index > 0 {
                    self.state.agent_index -= 1;
                }
            }
            _ => {}
        }
    }

    /// Handle navigation down in current section
    fn handle_navigation_down(&mut self) {
        match self.state.current_section {
            0 => {
                // Projects section
                let max_index = self.filtered_projects().len().saturating_sub(1);
                if self.state.project_index < max_index {
                    self.state.project_index += 1;
                }
            }
            1 => {
                // Repositories section
                let max_index = self.filtered_repositories().len().saturating_sub(1);
                if self.state.branch_index < max_index {
                    self.state.branch_index += 1;
                }
            }
            2 => {
                // Agents section
                let max_index = self.filtered_agents().len().saturating_sub(1);
                if self.state.agent_index < max_index {
                    self.state.agent_index += 1;
                }
            }
            _ => {}
        }
    }

    /// Handle character input
    fn handle_char_input(&mut self, c: char, modifiers: KeyModifiers) {
        if modifiers.contains(KeyModifiers::CONTROL) {
            return; // Ctrl+char combinations are handled separately
        }

        match self.state.current_section {
            0 => {
                // Project filter
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                    self.state.project_filter.push(c);
                    self.state.project_index = 0;
                }
            }
            1 => {
                // Repository filter
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '/' {
                    self.state.branch_filter.push(c);
                    self.state.branch_index = 0;
                }
            }
            2 => {
                // Agent filter
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                    self.state.agent_filter.push(c);
                    self.state.agent_index = 0;
                }
            }
            3 => {
                // Task description editing
                if c == '\n' {
                    self.state.task_description.push('\n');
                } else {
                    self.state.task_description.push(c);
                }
            }
            _ => {}
        }
    }

    /// Handle backspace
    fn handle_backspace(&mut self) {
        match self.state.current_section {
            0 => {
                self.state.project_filter.pop();
                self.state.project_index = 0;
            }
            1 => {
                self.state.branch_filter.pop();
                self.state.branch_index = 0;
            }
            2 => {
                self.state.agent_filter.pop();
                self.state.agent_index = 0;
            }
            3 => {
                self.state.task_description.pop();
            }
            _ => {}
        }
    }

    /// Get filtered projects based on current filter
    fn filtered_projects(&self) -> Vec<&Project> {
        if self.state.project_filter.is_empty() {
            self.state.projects.iter().collect()
        } else {
            self.state.projects.iter()
                .filter(|p| p.display_name.to_lowercase().contains(&self.state.project_filter.to_lowercase()))
                .collect()
        }
    }

    /// Get filtered repositories based on current filter
    fn filtered_repositories(&self) -> Vec<&Repository> {
        if self.state.branch_filter.is_empty() {
            self.state.repositories.iter().collect()
        } else {
            self.state.repositories.iter()
                .filter(|r| r.display_name.to_lowercase().contains(&self.state.branch_filter.to_lowercase()))
                .collect()
        }
    }

    /// Get filtered agents based on current filter
    fn filtered_agents(&self) -> Vec<&AgentCapability> {
        if self.state.agent_filter.is_empty() {
            self.state.agents.iter().collect()
        } else {
            self.state.agents.iter()
                .filter(|a| a.agent_type.to_lowercase().contains(&self.state.agent_filter.to_lowercase()))
                .collect()
        }
    }

    /// Reset all loading states
    fn reset_loading_states(&mut self) {
        self.state.projects_loading = false;
        self.state.repositories_loading = false;
        self.state.agents_loading = false;
    }

    /// Load initial data by triggering REST calls
    /// This would typically be called once at startup
    pub async fn load_initial_data(&mut self) -> Result<(), String> {
        // In a real implementation, this would spawn async tasks to load data
        // and send Net messages when complete. For testing purposes,
        // we'll simulate immediate responses.

        // Load projects
        self.state.projects_loading = true;
        match self.client.list_projects(None).await {
            Ok(projects) => {
                self.update(Msg::Net(NetMsg::RestSuccess(RestSuccessMsg::ProjectsList(projects)))).await;
            }
            Err(e) => {
                self.update(Msg::Net(NetMsg::RestError(e.to_string()))).await;
                return Err(e.to_string());
            }
        }

        // Load repositories
        self.state.repositories_loading = true;
        match self.client.list_repositories(None, None).await {
            Ok(repositories) => {
                self.update(Msg::Net(NetMsg::RestSuccess(RestSuccessMsg::RepositoriesList(repositories)))).await;
            }
            Err(e) => {
                self.update(Msg::Net(NetMsg::RestError(e.to_string()))).await;
                return Err(e.to_string());
            }
        }

        // Load agents
        self.state.agents_loading = true;
        match self.client.list_agents().await {
            Ok(agents) => {
                self.update(Msg::Net(NetMsg::RestSuccess(RestSuccessMsg::AgentsList(agents)))).await;
            }
            Err(e) => {
                self.update(Msg::Net(NetMsg::RestError(e.to_string()))).await;
                return Err(e.to_string());
            }
        }

        Ok(())
    }
}
