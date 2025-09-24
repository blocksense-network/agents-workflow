import { Component, createSignal, For } from "solid-js";

interface Repository {
  id: string;
  name: string;
  branch: string;
  lastCommit: string;
}

interface RepositoriesPaneProps {
  collapsed?: boolean;
  onToggleCollapse?: () => void;
  onRepositorySelect?: (repo: Repository) => void;
}

// Mock repository data for demo
const mockRepositories: Repository[] = [
  {
    id: "1",
    name: "agents-workflow-webui",
    branch: "main",
    lastCommit: "feat: Add real-time session updates"
  },
  {
    id: "2",
    name: "agents-workflow-core",
    branch: "develop",
    lastCommit: "refactor: Improve API error handling"
  },
  {
    id: "3",
    name: "agents-workflow-cli",
    branch: "main",
    lastCommit: "fix: Resolve path resolution issues"
  },
  {
    id: "4",
    name: "agents-workflow-docs",
    branch: "main",
    lastCommit: "docs: Update API documentation"
  }
];

export const RepositoriesPane: Component<RepositoriesPaneProps> = (props) => {
  const [selectedRepoId, setSelectedRepoId] = createSignal<string | null>(null);

  const handleRepoSelect = (repo: Repository) => {
    setSelectedRepoId(repo.id);
    props.onRepositorySelect?.(repo);
  };

  if (props.collapsed) {
    return (
      <div class="flex flex-col h-full">
        <div class="p-2 border-b border-gray-200 flex justify-center">
          <button
            onClick={props.onToggleCollapse}
            class="p-1 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded"
            title="Expand Repositories"
            aria-label="Expand Repositories pane"
          >
            <svg
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 5l7 7-7 7"
              ></path>
            </svg>
          </button>
        </div>
        <div class="flex-1 flex items-center justify-center">
          <div class="transform -rotate-90 whitespace-nowrap text-xs text-gray-500 font-medium">
            Repositories
          </div>
        </div>
      </div>
    );
  }

  return (
    <div class="flex flex-col h-full">
      <div class="p-4 border-b border-gray-200">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-lg font-semibold text-gray-900">Repositories</h2>
            <p class="text-sm text-gray-600 mt-1">
              Select a repository to create tasks
            </p>
          </div>
          <button
            onClick={props.onToggleCollapse}
            class="p-1 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded"
            title="Collapse Repositories"
            aria-label="Collapse Repositories pane"
          >
            <svg
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M15 19l-7-7 7-7"
              ></path>
            </svg>
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-4">
        <div class="space-y-2">
          <For each={mockRepositories}>
            {(repo) => (
              <div
                class={`p-3 rounded-lg border cursor-pointer transition-colors ${
                  selectedRepoId() === repo.id
                    ? "bg-blue-50 border-blue-200"
                    : "bg-white border-gray-200 hover:bg-gray-50"
                }`}
                onClick={() => handleRepoSelect(repo)}
              >
                <div class="flex items-center space-x-2 mb-2">
                  <div class="w-6 h-6 bg-gradient-to-br from-emerald-500 to-teal-500 rounded-md flex items-center justify-center">
                    <svg class="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M3 3h18v18H3V3zm16 16V5H5v14h14zM9 7h6v2H9V7zm0 4h6v2H9v-2zm0 4h4v2H9v-2z"/>
                    </svg>
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="text-sm font-medium text-gray-900 truncate">
                      {repo.name}
                    </div>
                  </div>
                </div>
                <div class="flex items-center space-x-1 text-xs text-gray-500 mb-1">
                  <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M7 6a1 1 0 011-1h8a1 1 0 011 1v1h3a1 1 0 011 1v10a1 1 0 01-1 1H4a1 1 0 01-1-1V8a1 1 0 011-1h3V6zM6 8H4v10h16V8h-2v1a1 1 0 01-1 1H7a1 1 0 01-1-1V8zm2-2v1h8V6H8z"/>
                  </svg>
                  <span>{repo.branch}</span>
                </div>
                <div class="text-xs text-gray-600 truncate">
                  {repo.lastCommit}
                </div>
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
};
