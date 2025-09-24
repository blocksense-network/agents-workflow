import { Component, createSignal, createEffect, onMount } from "solid-js";
import { RepositoriesPane } from "../repositories/RepositoriesPane.js";
import { TaskFeedPane } from "../sessions/TaskFeedPane.js";
import { TaskDetailsPane } from "../tasks/TaskDetailsPane.js";

interface ThreePaneLayoutProps {
  selectedSessionId?: string;
  onSessionSelect?: (sessionId: string) => void;
  onRepositorySelect?: (repo: { id: string; name: string; branch: string; lastCommit: string }) => void;
  onCreateTaskForRepo?: (repo: { id: string; name: string; branch: string; lastCommit: string }) => void;
  inlineTaskCreationRepo?: { id: string; name: string; branch: string; lastCommit: string } | null;
  onInlineTaskCreated?: (taskId: string) => void;
  onCancelInlineTaskCreation?: () => void;
}

export const ThreePaneLayout: Component<ThreePaneLayoutProps> = (props) => {
  console.log("ThreePaneLayout component function called, inlineTaskCreationRepo:", props.inlineTaskCreationRepo);
  // State for pane collapse/expand - start collapsed to show the new expand buttons
  const [repositoriesCollapsed, setRepositoriesCollapsed] = createSignal(true);
  const [sessionsCollapsed, setSessionsCollapsed] = createSignal(true);

  // Handle session selection
  const handleSessionSelect = (sessionId: string) => {
    props.onSessionSelect?.(sessionId);
  };

  // Load and save preferences from/to localStorage (client-side only)
  onMount(() => {
    // Load preferences from localStorage (only available on client)
    if (typeof window !== "undefined") {
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
    }
  });

  // Save preferences to localStorage
  const savePreferences = () => {
    if (typeof window !== "undefined") {
      const prefs = {
        repositoriesCollapsed: repositoriesCollapsed(),
        sessionsCollapsed: sessionsCollapsed(),
      };
      localStorage.setItem("webui-layout-prefs", JSON.stringify(prefs));
    }
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
    <div class="flex h-full overflow-hidden w-full">
      {/* Left Pane - Repositories */}
      <div
        class={`bg-white border-r border-gray-200 flex flex-col transition-all duration-200 ${
          repositoriesCollapsed() ? "w-12 flex-shrink-0" : "w-1/5 flex-shrink-0"
        }`}
      >
        <RepositoriesPane
          collapsed={repositoriesCollapsed()}
          onToggleCollapse={toggleRepositories}
          onRepositorySelect={props.onRepositorySelect}
          onCreateTaskForRepo={props.onCreateTaskForRepo}
        />
      </div>

      {/* Center Pane - Task Feed */}
      <div
        class={`bg-white border-r border-gray-200 flex flex-col transition-all duration-200 ${
          sessionsCollapsed() ? "w-12 flex-shrink-0" : "w-2/5 flex-shrink-0"
        }`}
      >
        <TaskFeedPane
          selectedSessionId={props.selectedSessionId}
          collapsed={sessionsCollapsed()}
          onToggleCollapse={toggleSessions}
          onSessionSelect={handleSessionSelect}
          inlineTaskCreationRepo={props.inlineTaskCreationRepo}
          onInlineTaskCreated={props.onInlineTaskCreated}
          onCancelInlineTaskCreation={props.onCancelInlineTaskCreation}
        />
      </div>

      {/* Right Pane - Task Details */}
      <div class="w-2/5 flex-shrink-0 bg-white flex flex-col min-w-0">
        <TaskDetailsPane sessionId={props.selectedSessionId} />
      </div>
    </div>
  );
};
