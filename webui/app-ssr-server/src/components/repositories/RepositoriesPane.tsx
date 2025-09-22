import { Component } from "solid-js";

interface RepositoriesPaneProps {
  collapsed?: boolean;
  onToggleCollapse?: () => void;
}

export const RepositoriesPane: Component<RepositoriesPaneProps> = (props) => {
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
          <div class="p-3 bg-gray-50 rounded-lg border border-gray-200">
            <div class="text-sm font-medium text-gray-900">
              Loading repositories...
            </div>
            <div class="text-xs text-gray-500 mt-1">
              This content will load with JavaScript enabled
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
