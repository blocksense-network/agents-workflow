import { Component, createSignal, createEffect } from "solid-js";
import { RepositoriesPane } from "../repositories/RepositoriesPane.js";
import { SessionsPane } from "../sessions/SessionsPane.js";
import { TaskDetailsPane } from "../tasks/TaskDetailsPane.js";

interface ThreePaneLayoutProps {
  selectedSessionId?: string;
}

export const ThreePaneLayout: Component<ThreePaneLayoutProps> = (props) => {
  // State for pane collapse/expand
  const [repositoriesCollapsed, setRepositoriesCollapsed] = createSignal(false);
  const [sessionsCollapsed, setSessionsCollapsed] = createSignal(false);

  // Load preferences from localStorage
  createEffect(() => {
    const saved = localStorage.getItem("webui-layout-prefs");
    if (saved) {
      try {
        const prefs = JSON.parse(saved);
        setRepositoriesCollapsed(prefs.repositoriesCollapsed || false);
        setSessionsCollapsed(prefs.sessionsCollapsed || false);
      } catch (e) {
        console.warn("Failed to parse layout preferences:", e);
      }
    }
  });

  // Save preferences to localStorage
  const savePreferences = () => {
    const prefs = {
      repositoriesCollapsed: repositoriesCollapsed(),
      sessionsCollapsed: sessionsCollapsed(),
    };
    localStorage.setItem("webui-layout-prefs", JSON.stringify(prefs));
  };

  const toggleRepositories = () => {
    setRepositoriesCollapsed(!repositoriesCollapsed());
    savePreferences();
  };

  const toggleSessions = () => {
    setSessionsCollapsed(!sessionsCollapsed());
    savePreferences();
  };

  return (
    <div class="flex h-full overflow-hidden">
      {/* Left Pane - Repositories */}
      <div
        class={`bg-white border-r border-gray-200 flex flex-col transition-all duration-200 ${
          repositoriesCollapsed() ? "w-12" : "w-80"
        }`}
      >
        <RepositoriesPane
          collapsed={repositoriesCollapsed()}
          onToggleCollapse={toggleRepositories}
        />
      </div>

      {/* Center Pane - Sessions Feed */}
      <div
        class={`bg-white border-r border-gray-200 flex flex-col transition-all duration-200 ${
          sessionsCollapsed() ? "w-12" : "flex-1 min-w-0"
        }`}
      >
        <SessionsPane
          selectedSessionId={props.selectedSessionId}
          collapsed={sessionsCollapsed()}
          onToggleCollapse={toggleSessions}
        />
      </div>

      {/* Right Pane - Task Details */}
      <div class="w-96 bg-white flex flex-col min-w-0">
        <TaskDetailsPane sessionId={props.selectedSessionId} />
      </div>
    </div>
  );
};
