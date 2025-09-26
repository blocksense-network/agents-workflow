//! ViewModel layer - derived presentation state shaped for rendering
//!
//! The ViewModel transforms domain state into presentation-ready data
//! (strings, selection flags, focus indicators) that the View can consume.

use crate::app::AppState;
use aw_rest_api_contract::{AgentCapability, Project, Repository};

/// ViewModel represents the presentation state derived from the Model
/// This is what the UI rendering code consumes - pure data, no business logic
#[derive(Debug, Clone, PartialEq)]
pub struct ViewModel {
    /// Current focused section (0=projects, 1=repositories, 2=agents, 3=task description)
    pub focus: Section,
    /// Selected project index (within filtered list)
    pub selected_project: usize,
    /// Selected repository index (within filtered list)
    pub selected_repository: usize,
    /// Selected agent index (within filtered list)
    pub selected_agent: usize,
    /// Current filter text for projects
    pub project_filter: String,
    /// Current filter text for repositories
    pub repository_filter: String,
    /// Current filter text for agents
    pub agent_filter: String,
    /// Filtered list of projects for display
    pub visible_projects: Vec<ProjectItem>,
    /// Filtered list of repositories for display
    pub visible_repositories: Vec<RepositoryItem>,
    /// Filtered list of agents for display
    pub visible_agents: Vec<AgentItem>,
    /// Current task description text
    pub task_description: String,
    /// Editor height for task description
    pub editor_height: usize,
    /// Current loading state
    pub loading: bool,
    /// Current error message (if any)
    pub error_message: Option<String>,
    /// Loading states for each data source
    pub projects_loading: bool,
    pub repositories_loading: bool,
    pub agents_loading: bool,
}

/// Represents a section in the UI
#[derive(Debug, Clone, PartialEq)]
pub enum Section {
    Projects,
    Repositories,
    Agents,
    TaskDescription,
}

/// Presentation-ready project item
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectItem {
    pub display_name: String,
    pub is_selected: bool,
}

/// Presentation-ready repository item
#[derive(Debug, Clone, PartialEq)]
pub struct RepositoryItem {
    pub display_name: String,
    pub scm_provider: String,
    pub is_selected: bool,
}

/// Presentation-ready agent item
#[derive(Debug, Clone, PartialEq)]
pub struct AgentItem {
    pub agent_type: String,
    pub versions: Vec<String>,
    pub is_selected: bool,
}

impl ViewModel {
    /// Create a ViewModel from the current AppState
    pub fn from_state(state: &AppState) -> Self {
        let focus = match state.current_section {
            0 => Section::Projects,
            1 => Section::Repositories,
            2 => Section::Agents,
            3 => Section::TaskDescription,
            _ => Section::Projects,
        };

        // Filter and prepare projects
        let filtered_projects = Self::filter_projects(&state.projects, &state.project_filter);
        let visible_projects = filtered_projects
            .iter()
            .enumerate()
            .map(|(idx, project)| ProjectItem {
                display_name: project.display_name.clone(),
                is_selected: idx == state.project_index,
            })
            .collect();

        // Filter and prepare repositories
        let filtered_repositories = Self::filter_repositories(&state.repositories, &state.branch_filter);
        let visible_repositories = filtered_repositories
            .iter()
            .enumerate()
            .map(|(idx, repo)| RepositoryItem {
                display_name: repo.display_name.clone(),
                scm_provider: repo.scm_provider.clone(),
                is_selected: idx == state.branch_index,
            })
            .collect();

        // Filter and prepare agents
        let filtered_agents = Self::filter_agents(&state.agents, &state.agent_filter);
        let visible_agents = filtered_agents
            .iter()
            .enumerate()
            .map(|(idx, agent)| AgentItem {
                agent_type: agent.agent_type.clone(),
                versions: agent.versions.clone(),
                is_selected: idx == state.agent_index,
            })
            .collect();

        Self {
            focus,
            selected_project: state.project_index,
            selected_repository: state.branch_index,
            selected_agent: state.agent_index,
            project_filter: state.project_filter.clone(),
            repository_filter: state.branch_filter.clone(),
            agent_filter: state.agent_filter.clone(),
            visible_projects,
            visible_repositories,
            visible_agents,
            task_description: state.task_description.clone(),
            editor_height: state.editor_height,
            loading: state.loading,
            error_message: state.error.clone(),
            projects_loading: state.projects_loading,
            repositories_loading: state.repositories_loading,
            agents_loading: state.agents_loading,
        }
    }

    /// Filter projects based on the current filter text
    fn filter_projects<'a>(projects: &'a [Project], filter: &str) -> Vec<&'a Project> {
        if filter.is_empty() {
            projects.iter().collect()
        } else {
            projects
                .iter()
                .filter(|p| p.display_name.to_lowercase().contains(&filter.to_lowercase()))
                .collect()
        }
    }

    /// Filter repositories based on the current filter text
    fn filter_repositories<'a>(repositories: &'a [Repository], filter: &str) -> Vec<&'a Repository> {
        if filter.is_empty() {
            repositories.iter().collect()
        } else {
            repositories
                .iter()
                .filter(|r| r.display_name.to_lowercase().contains(&filter.to_lowercase()))
                .collect()
        }
    }

    /// Filter agents based on the current filter text
    fn filter_agents<'a>(agents: &'a [AgentCapability], filter: &str) -> Vec<&'a AgentCapability> {
        if filter.is_empty() {
            agents.iter().collect()
        } else {
            agents
                .iter()
                .filter(|a| a.agent_type.to_lowercase().contains(&filter.to_lowercase()))
                .collect()
        }
    }

    /// Get the current focus as a string (useful for assertions)
    pub fn focus_string(&self) -> &str {
        match self.focus {
            Section::Projects => "projects",
            Section::Repositories => "repositories",
            Section::Agents => "agents",
            Section::TaskDescription => "task_description",
        }
    }

    /// Get the selected item name for the current focus (useful for assertions)
    pub fn selected_item_name(&self) -> Option<&str> {
        match self.focus {
            Section::Projects => self.visible_projects.get(self.selected_project)
                .map(|p| p.display_name.as_str()),
            Section::Repositories => self.visible_repositories.get(self.selected_repository)
                .map(|r| r.display_name.as_str()),
            Section::Agents => self.visible_agents.get(self.selected_agent)
                .map(|a| a.agent_type.as_str()),
            Section::TaskDescription => None,
        }
    }

    /// Check if a specific item is selected in the current focus section
    pub fn is_selected(&self, name: &str) -> bool {
        match self.focus {
            Section::Projects => self.visible_projects.get(self.selected_project)
                .map(|p| p.display_name == name).unwrap_or(false),
            Section::Repositories => self.visible_repositories.get(self.selected_repository)
                .map(|r| r.display_name == name).unwrap_or(false),
            Section::Agents => self.visible_agents.get(self.selected_agent)
                .map(|a| a.agent_type == name).unwrap_or(false),
            Section::TaskDescription => false,
        }
    }

    /// Get the current filter text for the focused section
    pub fn current_filter(&self) -> Option<&str> {
        match self.focus {
            Section::Projects => Some(""), // Would need to be passed from state
            Section::Repositories => Some(""),
            Section::Agents => Some(""),
            Section::TaskDescription => None,
        }
    }
}
