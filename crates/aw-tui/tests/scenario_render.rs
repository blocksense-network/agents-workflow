//! Scenario-based initial render test

use aw_test_scenarios::{Scenario, ScenarioTerminal};
use aw_tui::{create_test_terminal, app::AppState, ViewModel};
use ratatui::widgets::ListState;

#[test]
fn test_initial_render_from_minimal_scenario() -> anyhow::Result<()> {
    // Minimal scenario JSON (could be moved to a fixture file later)
    let json = r#"{
        "name": "minimal_initial",
        "terminal": { "width": 80, "height": 24 },
        "steps": []
    }"#;

    let scenario = Scenario::from_str(json)?;
    let (w, h) = scenario
        .terminal
        .map(|t| (t.width.unwrap_or(80), t.height.unwrap_or(24)))
        .unwrap_or((80, 24));

    let mut term = create_test_terminal(w, h);

    // Render the current dashboard with default state
    term.draw(|f| {
        let area = f.size();
        let mut project_state = ListState::default();
        let mut branch_state = ListState::default();
        let mut agent_state = ListState::default();

        // Build a default AppState via a lightweight path: reuse draw_dashboard with empty data
        let state = AppState::default();
        let view_model = ViewModel::from_state(&state);
        aw_tui::ui::draw_task_dashboard(
            f,
            area,
            &view_model,
            None,
            None,
        );
    })?;

    let buffer = term.backend().buffer();
    let all_text = buffer
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Expect the static section titles to be present
    assert!(all_text.contains("╔"), "Should render header with logo border");
    assert!(all_text.contains("New Task"));
    assert!(all_text.contains("Description"));

    Ok(())
}


