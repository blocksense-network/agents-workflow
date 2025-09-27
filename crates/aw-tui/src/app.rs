//! Main TUI application logic

use crate::{ButtonFocus, ModalState, ModelSelection};
use crate::task::create_default_models;
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
use crate::task::{Task, TaskState};
use crate::ui;

/// Application state
#[derive(Debug)]
pub struct AppState {
    // Task list and selection
    pub tasks: Vec<Task>,
    pub selected_task_index: usize,

    // New task input
    pub new_task_description: String,

    // UI state
    pub loading: bool,
    pub error: Option<String>,

    // Draft task handling
    pub has_unsaved_draft: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let mut tasks = crate::task::create_sample_tasks();
        // Add the new task at the end
        tasks.push(Task::new());
        let total_tasks = tasks.len();

        Self {
            // Previous tasks + new task creation interface
            tasks,
            selected_task_index: total_tasks - 1, // Focus on new task

            // New task input
            new_task_description: String::new(),

            // UI state
            loading: false,
            error: None,

            // Draft task handling
            has_unsaved_draft: false,
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
        // For now, we'll use sample data. In the future, this would load
        // real tasks from the REST API
            let mut state = self.state.lock().await;
        state.tasks = crate::task::create_sample_tasks();
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

                    ui::draw_task_dashboard(f, size, &view_model);
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
                KeyCode::Up => {
                    // Navigate up in task list or within modal
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: Some(ref mut modal), .. } = current_task.state {
                            // Navigate within modal
                            match modal {
                                ModalState::RepositorySelect { ref mut selected_index, options, .. } |
                                ModalState::BranchSelect { ref mut selected_index, options, .. } => {
                                    if *selected_index > 0 {
                                        *selected_index -= 1;
                                    }
                                }
                                ModalState::ModelSelect { ref mut selected_index, options, .. } => {
                                    if *selected_index > 0 {
                                        *selected_index -= 1;
                                    }
                                }
                            }
                        } else if matches!(current_task.state, TaskState::New { .. }) {
                            // Regular task navigation
                            if selected_index > 0 {
                                state.selected_task_index -= 1;
                            }
                        } else {
                            // Regular task navigation
                            if selected_index > 0 {
                                state.selected_task_index -= 1;
                            }
                        }
                    }
                }
                KeyCode::Down => {
                    // Navigate down in task list or within modal
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: Some(ref mut modal), .. } = current_task.state {
                            // Navigate within modal
                            match modal {
                                ModalState::RepositorySelect { ref mut selected_index, options, .. } => {
                                    let max_index = options.len().saturating_sub(1);
                                    if *selected_index < max_index {
                                        *selected_index += 1;
                                    }
                                }
                                ModalState::BranchSelect { ref mut selected_index, options, .. } => {
                                    let max_index = options.len().saturating_sub(1);
                                    if *selected_index < max_index {
                                        *selected_index += 1;
                                    }
                                }
                                ModalState::ModelSelect { ref mut selected_index, options, .. } => {
                                    let max_index = options.len().saturating_sub(1);
                                    if *selected_index < max_index {
                                        *selected_index += 1;
                                    }
                                }
                            }
                                } else {
                            // Regular task navigation
                            let max_index = state.tasks.len().saturating_sub(1);
                            if selected_index < max_index {
                                state.selected_task_index += 1;
                            }
                        }
                    }
                }
                KeyCode::Left => {
                    // Decrease model instance count in model selection modal
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: Some(ModalState::ModelSelect { selected_index: model_idx, options, .. }), .. } = &mut current_task.state {
                            if let Some(model) = options.get_mut(*model_idx) {
                                if model.instance_count > 0 {
                                    model.instance_count -= 1;
                                }
                            }
                        }
                    }
                }
                KeyCode::Right => {
                    // Increase model instance count in model selection modal
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: Some(ModalState::ModelSelect { selected_index: model_idx, options, .. }), .. } = &mut current_task.state {
                            if let Some(model) = options.get_mut(*model_idx) {
                                model.instance_count += 1;
                            }
                        }
                    }
                }
                KeyCode::Tab => {
                    // Cycle between buttons in task creation
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: None, ref mut focused_button, .. } = current_task.state {
                            *focused_button = match focused_button {
                                ButtonFocus::Description => ButtonFocus::Repository,
                                ButtonFocus::Repository => ButtonFocus::Branch,
                                ButtonFocus::Branch => ButtonFocus::Models,
                                ButtonFocus::Models => ButtonFocus::Go,
                                ButtonFocus::Go => ButtonFocus::Description,
                            };
                        }
                    }
                }
                KeyCode::BackTab => {
                    // Reverse cycle between buttons
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: None, ref mut focused_button, .. } = current_task.state {
                            *focused_button = match focused_button {
                                ButtonFocus::Description => ButtonFocus::Go,
                                ButtonFocus::Repository => ButtonFocus::Description,
                                ButtonFocus::Branch => ButtonFocus::Repository,
                                ButtonFocus::Models => ButtonFocus::Branch,
                                ButtonFocus::Go => ButtonFocus::Models,
                            };
                        }
                    }
                }
                KeyCode::Char('+') => {
                    // Alternative way to increase model count in modal
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: Some(ModalState::ModelSelect { selected_index: model_idx, options, .. }), .. } = &mut current_task.state {
                            if let Some(model) = options.get_mut(*model_idx) {
                                model.instance_count += 1;
                            }
                        }
                    }
                }
                KeyCode::Char('-') => {
                    // Alternative way to decrease model count in modal
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: Some(ModalState::ModelSelect { selected_index: model_idx, options, .. }), .. } = &mut current_task.state {
                            if let Some(model) = options.get_mut(*model_idx) {
                                if model.instance_count > 0 {
                                    model.instance_count -= 1;
                                }
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    // Handle text input when description is focused
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { focused_button, description, .. } = &mut current_task.state {
                            if matches!(focused_button, ButtonFocus::Description) {
                                description.push(c);
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    // Handle backspace when description is focused
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { focused_button, description, .. } = &mut current_task.state {
                            if matches!(focused_button, ButtonFocus::Description) {
                                description.pop();
                            }
                        }
                    }
                }
                KeyCode::Enter => {
                    // Handle Enter key based on context
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: Some(ref modal), focused_button, selected_repo, selected_branch, selected_models, description, .. } = &mut current_task.state {
                            // In modal: select current item
                            match modal {
                                ModalState::RepositorySelect { selected_index: idx, options, .. } => {
                                    if let Some(selected_repo_name) = options.get(*idx).cloned() {
                                        *selected_repo = selected_repo_name;
                                        current_task.state = TaskState::New {
                                            description: String::new(),
                                            selected_repo: selected_repo.clone(),
                                            selected_branch: selected_branch.clone(),
                                            selected_models: selected_models.clone(),
                                            focused_button: focused_button.clone(),
                                            modal_state: None,
                                        };
                                    }
                                }
                                ModalState::BranchSelect { selected_index: idx, options, .. } => {
                                    if let Some(selected_branch_name) = options.get(*idx).cloned() {
                                        *selected_branch = selected_branch_name;
                                        current_task.state = TaskState::New {
                                            description: String::new(),
                                            selected_repo: selected_repo.clone(),
                                            selected_branch: selected_branch.clone(),
                                            selected_models: selected_models.clone(),
                                            focused_button: focused_button.clone(),
                                            modal_state: None,
                                        };
                                    }
                                }
                                ModalState::ModelSelect { options, .. } => {
                                    let selected_models_new: Vec<ModelSelection> = options.iter()
                                        .filter(|m| m.instance_count > 0)
                                        .cloned()
                                        .collect();
                                    current_task.state = TaskState::New {
                                        description: String::new(),
                                        selected_repo: selected_repo.clone(),
                                        selected_branch: selected_branch.clone(),
                                        selected_models: selected_models_new,
                                        focused_button: focused_button.clone(),
                                        modal_state: None,
                                    };
                                }
                            }
                        } else if let TaskState::New { focused_button, selected_repo, selected_branch, selected_models, description, .. } = &mut current_task.state {
                            // Not in modal: activate focused button
                            match focused_button {
                                ButtonFocus::Repository => {
                                    // Open repository selection modal
                                    let options = crate::task::create_sample_repos();
                                    current_task.state = TaskState::New {
                                        description: String::new(),
                                        selected_repo: selected_repo.clone(),
                                        selected_branch: selected_branch.clone(),
                                        selected_models: selected_models.clone(),
                                        focused_button: focused_button.clone(),
                                        modal_state: Some(ModalState::RepositorySelect {
                                            query: String::new(),
                                            options,
                                            selected_index: 0,
                                        }),
                                    };
                                }
                                ButtonFocus::Branch => {
                                    // Open branch selection modal
                                    let options = crate::task::create_sample_branches();
                                    current_task.state = TaskState::New {
                                        description: String::new(),
                                        selected_repo: selected_repo.clone(),
                                        selected_branch: selected_branch.clone(),
                                        selected_models: selected_models.clone(),
                                        focused_button: focused_button.clone(),
                                        modal_state: Some(ModalState::BranchSelect {
                                            query: String::new(),
                                            options,
                                            selected_index: 0,
                                        }),
                                    };
                                }
                                ButtonFocus::Models => {
                                    // Open model selection modal
                                    let options = crate::task::create_default_models();
                                    current_task.state = TaskState::New {
                                        description: String::new(),
                                        selected_repo: selected_repo.clone(),
                                        selected_branch: selected_branch.clone(),
                                        selected_models: selected_models.clone(),
                                        focused_button: focused_button.clone(),
                                        modal_state: Some(ModalState::ModelSelect {
                                            query: String::new(),
                                            options,
                                            selected_index: 0,
                                        }),
                                    };
                                }
                                ButtonFocus::Go => {
                                    // Launch task
                        self.create_task(&mut state).await?;
                                }
                                ButtonFocus::Description => {
                                    // Description is handled by direct typing, not button activation
                                    // When focused, user can type directly into the description area
                                }
                            }
                        } else {
                            // Regular task selection
                            // Could implement task selection/activation here
                        }
                    }
                }
                KeyCode::Esc => {
                    // Handle Escape key based on context
                    let selected_index = state.selected_task_index;
                    if let Some(current_task) = state.tasks.get_mut(selected_index) {
                        if let TaskState::New { modal_state: Some(_), focused_button, selected_repo, selected_branch, selected_models, description, .. } = &current_task.state {
                            // Close modal
                            current_task.state = TaskState::New {
                                description: String::new(),
                                selected_repo: selected_repo.clone(),
                                selected_branch: selected_branch.clone(),
                                selected_models: selected_models.clone(),
                                focused_button: focused_button.clone(),
                                modal_state: None,
                            };
                        }
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

        // Get the task creation configuration from the New task state
        let (selected_repo, selected_branch, selected_models) = if let Some(new_task) = state.tasks.last() {
            if let TaskState::New { selected_repo, selected_branch, selected_models, .. } = &new_task.state {
                let models = selected_models.iter()
                    .filter(|m| m.instance_count > 0)
                    .map(|m| format!("{} (x{})", m.model_name, m.instance_count))
                    .collect::<Vec<_>>();
                (selected_repo.clone(), selected_branch.clone(), models)
            } else {
                ("Unknown".to_string(), "Unknown".to_string(), vec!["GPT-4 (x1)".to_string()])
            }
        } else {
            ("Unknown".to_string(), "Unknown".to_string(), vec!["GPT-4 (x1)".to_string()])
        };

        // Create a descriptive task title
        let task_title = format!("Task on {}:{} with {}", selected_repo, selected_branch, selected_models.join(", "));

        // Create a new active task
        let new_task = Task::active(
            format!("task_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
            task_title,
            std::time::SystemTime::now(),
            "Starting task".to_string(),
            "Initializing agent environment".to_string(),
            None,
        );

        // Add the new task to the list (before the New task input)
        let new_task_index = state.tasks.len().saturating_sub(1);
        state.tasks.insert(new_task_index, new_task);
        state.selected_task_index = new_task_index;

        // Reset the task creation state
        if let Some(last_task) = state.tasks.last_mut() {
            if let TaskState::New { ref mut selected_repo, ref mut selected_branch, ref mut selected_models, ref mut focused_button, ref mut modal_state, ref mut description } = last_task.state {
                *selected_repo = "agent-workflow".to_string();
                *selected_branch = "main".to_string();
                *selected_models = create_default_models().into_iter().filter(|m| m.selected).collect();
                *focused_button = ButtonFocus::Description;
                *modal_state = None;
                *description = String::new();
            }
        }

        // This would create the actual task - simplified for now
        // In a real implementation, this would use the REST client
        // to create a task with the selected repo, branch, and models

        // Simulate task creation delay
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        state.loading = false;

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

