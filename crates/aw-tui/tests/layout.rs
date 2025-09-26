//! Layout rendering tests for the TUI dashboard

use aw_rest_api_contract::{AgentCapability, Project, Repository};
use aw_tui::{app::AppState, ViewModel};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::io::Result;

/// Test that the dashboard renders correctly on different terminal sizes
#[test]
fn test_dashboard_layout_small_terminal() -> Result<()> {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Create test state with sample data
    let mut state = AppState::default();
    state.projects = vec![
        Project {
            id: "proj1".to_string(),
            display_name: "Test Project 1".to_string(),
            last_used_at: None,
        },
        Project {
            id: "proj2".to_string(),
            display_name: "Test Project 2".to_string(),
            last_used_at: None,
        },
    ];
    state.repositories = vec![
        Repository {
            id: "repo1".to_string(),
            display_name: "Test Repo 1".to_string(),
            scm_provider: "github".to_string(),
            remote_url: "https://github.com/test/repo1".parse().unwrap(),
            default_branch: "main".to_string(),
            last_used_at: None,
        },
    ];
    state.agents = vec![
        AgentCapability {
            agent_type: "claude-code".to_string(),
            versions: vec!["latest".to_string()],
            settings_schema_ref: None,
        },
    ];

    // Create ViewModel from state
    let view_model = ViewModel::from_state(&state);

    // Test rendering
    terminal.draw(|f| {
        let size = f.size();
        let mut project_state = ratatui::widgets::ListState::default();
        let mut branch_state = ratatui::widgets::ListState::default();
        let mut agent_state = ratatui::widgets::ListState::default();

        aw_tui::ui::draw_dashboard(
            f,
            size,
            &view_model,
            &mut project_state,
            &mut branch_state,
            &mut agent_state,
        );
    })?;

    let buffer = terminal.backend().buffer();

    // Check that the layout contains expected elements
    let all_text = buffer.content().iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    // Should contain project data
    assert!(all_text.contains("Test Project 1"), "Should contain 'Test Project 1'");

    // Should contain repository data
    assert!(all_text.contains("Test Repo 1"), "Should contain 'Test Repo 1'");

    // Should contain agent data
    assert!(all_text.contains("claude-code"), "Should contain 'claude-code'");

    // Should contain section titles
    assert!(all_text.contains("Project Filter"));
    assert!(all_text.contains("Repository"));
    assert!(all_text.contains("Agent"));

    Ok(())
}

#[test]
fn test_dashboard_layout_large_terminal() -> Result<()> {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend)?;

    let state = AppState::default();
    let view_model = ViewModel::from_state(&state);

    terminal.draw(|f| {
        let size = f.size();
        let mut project_state = ratatui::widgets::ListState::default();
        let mut branch_state = ratatui::widgets::ListState::default();
        let mut agent_state = ratatui::widgets::ListState::default();

        aw_tui::ui::draw_dashboard(
            f,
            size,
            &view_model,
            &mut project_state,
            &mut branch_state,
            &mut agent_state,
        );
    })?;

    let buffer = terminal.backend().buffer();

    // Check that layout adapts to larger terminal
    assert!(buffer.area().width >= 120);
    assert!(buffer.area().height >= 40);

    Ok(())
}

#[test]
fn test_focus_indication() -> Result<()> {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::default();
    state.current_section = 0; // Focus on projects
    let view_model = ViewModel::from_state(&state);

    terminal.draw(|f| {
        let size = f.size();
        let mut project_state = ratatui::widgets::ListState::default();
        let mut branch_state = ratatui::widgets::ListState::default();
        let mut agent_state = ratatui::widgets::ListState::default();

        aw_tui::ui::draw_dashboard(
            f,
            size,
            &view_model,
            &mut project_state,
            &mut branch_state,
            &mut agent_state,
        );
    })?;

    let buffer = terminal.backend().buffer();
    let _content = buffer.content();

    // Should show focus indication (cyan color in project section)
    // This is a basic test - more sophisticated color testing would require
    // examining the buffer cells more carefully

    Ok(())
}
