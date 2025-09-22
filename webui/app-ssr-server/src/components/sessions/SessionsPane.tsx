import { Component } from "solid-js";

interface SessionsPaneProps {
  selectedSessionId?: string;
  collapsed?: boolean;
  onToggleCollapse?: () => void;
}

export const SessionsPane: Component<SessionsPaneProps> = (props) => {
  if (props.collapsed) {
    return (
      <div class="flex flex-col h-full">
        <div class="p-2 border-b border-gray-200 flex justify-center">
          <button
            onClick={props.onToggleCollapse}
            class="p-1 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded"
            title="Expand Sessions"
            aria-label="Expand Sessions pane"
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
            Sessions
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
            <h2 class="text-lg font-semibold text-gray-900">Sessions</h2>
            <p class="text-sm text-gray-600 mt-1">
              Active and recent agent sessions
            </p>
          </div>
          <button
            onClick={props.onToggleCollapse}
            class="p-1 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded"
            title="Collapse Sessions"
            aria-label="Collapse Sessions pane"
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
        <div class="space-y-3">
          <div class="p-4 bg-gray-50 rounded-lg border border-gray-200">
            <div class="text-sm font-medium text-gray-900">
              Loading sessions...
            </div>
            <div class="text-xs text-gray-500 mt-1">
              Real-time session data will appear here
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
