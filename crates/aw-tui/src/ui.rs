//! UI components for the TUI application

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

/// Draw the main dashboard layout
pub fn draw_dashboard(
    f: &mut ratatui::Frame,
    area: Rect,
    project_state: &mut ListState,
    branch_state: &mut ListState,
    agent_state: &mut ListState,
    task_description: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Project selector
            Constraint::Length(3), // Branch selector
            Constraint::Length(3), // Agent selector
            Constraint::Min(5),    // Task description editor
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Project selector
    draw_selector(
        f,
        chunks[0],
        "Project",
        &["project-1", "project-2", "project-3"],
        project_state,
    );

    // Branch selector
    draw_selector(
        f,
        chunks[1],
        "Branch",
        &["main", "feature/agent-workflow", "develop"],
        branch_state,
    );

    // Agent selector
    draw_selector(
        f,
        chunks[2],
        "Agent",
        &["claude-code", "openhands", "copilot"],
        agent_state,
    );

    // Task description editor
    draw_task_editor(f, chunks[3], task_description);

    // Footer with shortcuts
    draw_footer(f, chunks[4]);
}

/// Draw a selector widget
fn draw_selector(
    f: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    items: &[&str],
    state: &mut ListState,
) {
    let items: Vec<ListItem> = items
        .iter()
        .map(|item| ListItem::new(*item))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, state);
}

/// Draw the task description editor
fn draw_task_editor(f: &mut ratatui::Frame, area: Rect, description: &str) {
    let paragraph = Paragraph::new(description)
        .block(Block::default().borders(Borders::ALL).title("Task Description"))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Draw the footer with contextual shortcuts
fn draw_footer(f: &mut ratatui::Frame, area: Rect) {
    let shortcuts = vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" - Next Section  "),
        Span::styled("Shift+Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" - Previous Section  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" - Create Task  "),
        Span::styled("Ctrl+C", Style::default().fg(Color::Red)),
        Span::raw(" - Quit"),
    ];

    let line = Line::from(shortcuts);
    let paragraph = Paragraph::new(line).block(Block::default().borders(Borders::ALL));

    f.render_widget(paragraph, area);
}

/// Draw a loading overlay
pub fn draw_loading(f: &mut ratatui::Frame, area: Rect, message: &str) {
    let popup_area = centered_rect(60, 20, area);

    f.render_widget(Clear, popup_area);

    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Loading")
                .style(Style::default().fg(Color::Yellow)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, popup_area);
}

/// Draw an error overlay
pub fn draw_error(f: &mut ratatui::Frame, area: Rect, error: &str) {
    let popup_area = centered_rect(80, 30, area);

    f.render_widget(Clear, popup_area);

    let paragraph = Paragraph::new(error)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Error")
                .style(Style::default().fg(Color::Red)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, popup_area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
