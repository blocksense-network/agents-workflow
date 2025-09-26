//! Test runtime for deterministic scenario execution
//!
//! This module provides the core testing infrastructure that can execute
//! scenarios step-by-step in a deterministic manner using fake time.

use aw_client_api::ClientApi;
use aw_test_scenarios::{Scenario, Step};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::sync::Arc;
use tokio::time;

use crate::{
    create_test_terminal,
    model::Model,
    msg::{Msg, NetMsg, RestSuccessMsg},
    viewmodel::ViewModel,
    ui,
};

/// Test runtime that executes scenarios deterministically
pub struct TestRuntime<C: ClientApi> {
    terminal: Terminal<TestBackend>,
    model: Model<C>,
    current_view_model: ViewModel,
}

impl<C: ClientApi> TestRuntime<C> {
    /// Create a new test runtime
    pub fn new(client: Arc<C>, width: u16, height: u16) -> Self {
        let terminal = create_test_terminal(width, height);
        let model = Model::new(client);

        // Get initial view model
        let current_view_model = ViewModel::from_state(&model.state);

        Self {
            terminal,
            model,
            current_view_model,
        }
    }

    /// Execute a single step from the scenario
    ///
    /// This is the core deterministic execution method that handles
    /// exactly one message and renders once.
    pub async fn execute_step(&mut self, step: &Step) -> Result<(), String> {
        match step {
            Step::AdvanceMs { ms } => {
                // Advance fake time
                time::advance(time::Duration::from_millis(*ms as u64)).await;
                self.model.update(Msg::Tick).await;
            }
            Step::Key { key } => {
                // Convert string key to crossterm KeyEvent
                let key_event = self.parse_key(key)?;
                self.model.update(Msg::Key(key_event)).await;
            }
            Step::Sse { event } => {
                // Inject SSE event
                self.model.update(Msg::Net(NetMsg::Sse(event.clone()))).await;
            }
            Step::AssertVm { focus, selected } => {
                // Run ViewModel assertion
                self.assert_view_model(focus, *selected)?;
            }
            Step::Snapshot { name: _ } => {
                // Take snapshot (implemented later)
                // For now, just render to ensure consistency
                self.render()?;
            }
        }

        // Update view model after state change
        self.current_view_model = ViewModel::from_state(&self.model.state);

        // Always render after each step for consistency
        self.render()?;

        Ok(())
    }

    /// Load initial data (projects, repos, agents)
    pub async fn load_initial_data(&mut self) -> Result<(), String> {
        self.model.load_initial_data().await?;
        self.current_view_model = ViewModel::from_state(&self.model.state);
        self.render()?;
        Ok(())
    }

    /// Get the current ViewModel for assertions
    pub fn view_model(&self) -> &ViewModel {
        &self.current_view_model
    }

    /// Get the terminal buffer as a string for snapshot comparisons
    pub fn buffer_content(&self) -> String {
        self.terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    /// Assert ViewModel state
    fn assert_view_model(&self, expected_focus: &str, expected_selected: Option<usize>) -> Result<(), String> {
        let vm = &self.current_view_model;

        // Check focus
        let actual_focus = vm.focus_string();
        if actual_focus != expected_focus {
            return Err(format!(
                "Focus assertion failed: expected '{}', got '{}'",
                expected_focus, actual_focus
            ));
        }

        // Check selection if specified
        if let Some(expected_idx) = expected_selected {
            let actual_selected = match expected_focus {
                "projects" => vm.selected_project,
                "repositories" => vm.selected_repository,
                "agents" => vm.selected_agent,
                _ => {
                    return Err(format!("Cannot check selection for focus '{}'", expected_focus));
                }
            };

            if actual_selected != expected_idx {
                return Err(format!(
                    "Selection assertion failed for {}: expected {}, got {}",
                    expected_focus, expected_idx, actual_selected
                ));
            }
        }

        Ok(())
    }

    /// Parse a key string into a KeyEvent
    fn parse_key(&self, key: &str) -> Result<KeyEvent, String> {
        match key {
            "Tab" => Ok(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            "Down" => Ok(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            "Up" => Ok(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            "Enter" => Ok(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            "Esc" => Ok(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            "Backspace" => Ok(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            // Add more key mappings as needed
            other => {
                // Try to parse as a single character
                if other.len() == 1 {
                    let c = other.chars().next().unwrap();
                    Ok(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
                } else {
                    Err(format!("Unsupported key: {}", other))
                }
            }
        }
    }

    /// Render the current state to the test terminal
    fn render(&mut self) -> Result<(), String> {
        self.terminal
            .draw(|f| {
                let area = f.size();
                let mut project_state = ratatui::widgets::ListState::default();
                let mut branch_state = ratatui::widgets::ListState::default();
                let mut agent_state = ratatui::widgets::ListState::default();

                ui::draw_dashboard(f, area, &self.current_view_model, &mut project_state, &mut branch_state, &mut agent_state);
            })
            .map_err(|e| format!("Render error: {}", e))?;

        Ok(())
    }
}

/// Execute a complete scenario
pub async fn execute_scenario<C: ClientApi>(
    client: Arc<C>,
    scenario: &Scenario,
) -> Result<(), String> {
    // Set up fake time
    time::pause();

    let (width, height) = scenario
        .terminal
        .as_ref()
        .map(|t| (t.width.unwrap_or(80), t.height.unwrap_or(24)))
        .unwrap_or((80, 24));

    let mut runtime = TestRuntime::new(client, width, height);

    // Load initial data
    runtime.load_initial_data().await?;

    // Execute each step
    for step in &scenario.steps {
        runtime.execute_step(step).await?;
    }

    Ok(())
}
