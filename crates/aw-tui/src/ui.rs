//! UI components for the TUI application

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::viewmodel::{ViewModel, Section};

/// Draw the main dashboard layout
pub fn draw_dashboard(
    f: &mut ratatui::Frame,
    area: Rect,
    view_model: &ViewModel,
    project_state: &mut ListState,
    branch_state: &mut ListState,
    agent_state: &mut ListState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Project selector (with filter)
            Constraint::Length(4), // Branch selector (with filter)
            Constraint::Length(4), // Agent selector (with filter)
            Constraint::Length(view_model.editor_height as u16), // Task description editor (resizable)
            Constraint::Length(4), // Footer (with more shortcuts)
        ])
        .split(area);

    // Set list states based on ViewModel
    project_state.select(Some(view_model.selected_project));
    branch_state.select(Some(view_model.selected_repository));
    agent_state.select(Some(view_model.selected_agent));

    // Project selector
    let project_items: Vec<&str> = view_model.visible_projects.iter()
        .map(|p| p.display_name.as_str())
        .collect();
    draw_selector_with_filter(
        f,
        chunks[0],
        "Project",
        &project_items,
        &view_model.project_filter,
        matches!(view_model.focus, Section::Projects),
        view_model.projects_loading,
        project_state,
    );

    // Branch selector
    let branch_items: Vec<&str> = view_model.visible_repositories.iter()
        .map(|r| r.display_name.as_str())
        .collect();
    draw_selector_with_filter(
        f,
        chunks[1],
        "Repository",
        &branch_items,
        &view_model.repository_filter,
        matches!(view_model.focus, Section::Repositories),
        view_model.repositories_loading,
        branch_state,
    );

    // Agent selector
    let agent_items: Vec<&str> = view_model.visible_agents.iter()
        .map(|a| a.agent_type.as_str())
        .collect();
    draw_selector_with_filter(
        f,
        chunks[2],
        "Agent",
        &agent_items,
        &view_model.agent_filter,
        matches!(view_model.focus, Section::Agents),
        view_model.agents_loading,
        agent_state,
    );

    // Task description editor
    draw_task_editor(f, chunks[3], &view_model.task_description, matches!(view_model.focus, Section::TaskDescription));

    // Footer with contextual shortcuts
    let section_index = match view_model.focus {
        Section::Projects => 0,
        Section::Repositories => 1,
        Section::Agents => 2,
        Section::TaskDescription => 3,
    };
    draw_footer(f, chunks[4], section_index);
}

/// Draw a selector widget with filtering
fn draw_selector_with_filter(
    f: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    items: &[&str],
    filter: &str,
    is_focused: bool,
    is_loading: bool,
    state: &mut ListState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Filter input
            Constraint::Min(1),    // List
        ])
        .split(area);

    // Filter input
    let filter_text = if filter.is_empty() {
        format!("Filter {}...", title.to_lowercase())
    } else {
        filter.to_string()
    };

    let filter_style = if is_focused && filter.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else if is_focused {
        Style::default().fg(Color::White).bg(Color::Blue)
    } else {
        Style::default().fg(Color::White)
    };

    let filter_paragraph = Paragraph::new(filter_text)
        .style(filter_style)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT).title(format!("{} Filter", title)));

    f.render_widget(filter_paragraph, chunks[0]);

    // Filtered items
    let filtered_items: Vec<&str> = if filter.is_empty() {
        items.to_vec()
    } else {
        items.iter()
            .filter(|item| item.to_lowercase().contains(&filter.to_lowercase()))
            .copied()
            .collect()
    };

    let list_items: Vec<ListItem> = if is_loading {
        vec![ListItem::new("Loading...")]
    } else if filtered_items.is_empty() && !filter.is_empty() {
        vec![ListItem::new("No matches found")]
    } else {
        filtered_items
            .iter()
            .map(|item| ListItem::new(*item))
            .collect()
    };

    let block_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT).style(block_style).title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], state);
}


/// Draw the task description editor
fn draw_task_editor(f: &mut ratatui::Frame, area: Rect, description: &str, is_focused: bool) {
    let block_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let paragraph = Paragraph::new(description)
        .block(Block::default().borders(Borders::ALL).style(block_style).title("Task Description (Ctrl+Up/Down to resize)"))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Draw the footer with contextual shortcuts
fn draw_footer(f: &mut ratatui::Frame, area: Rect, current_section: usize) {
    let shortcuts = match current_section {
        0 => vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" Next • "),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate • "),
            Span::styled("Type", Style::default().fg(Color::Green)),
            Span::raw(" Filter • "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red)),
            Span::raw(" Quit"),
        ],
        1 => vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" Next • "),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate • "),
            Span::styled("Type", Style::default().fg(Color::Green)),
            Span::raw(" Filter • "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red)),
            Span::raw(" Quit"),
        ],
        2 => vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" Next • "),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate • "),
            Span::styled("Type", Style::default().fg(Color::Green)),
            Span::raw(" Filter • "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red)),
            Span::raw(" Quit"),
        ],
        3 => vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" Next • "),
            Span::styled("Ctrl+↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Resize • "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" Create Task • "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red)),
            Span::raw(" Quit"),
        ],
        _ => vec![
            Span::styled("Ctrl+C", Style::default().fg(Color::Red)),
            Span::raw(" Quit"),
        ],
    };

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
