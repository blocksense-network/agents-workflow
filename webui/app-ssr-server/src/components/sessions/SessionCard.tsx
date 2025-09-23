import { Component, Show } from "solid-js";
import { Session } from "../../lib/api.js";

interface SessionCardProps {
  session: Session;
  isSelected?: boolean;
  onClick?: () => void;
  onStop?: () => void;
  onCancel?: () => void;
}

const getStatusColor = (status: string) => {
  switch (status) {
    case "running":
      return "bg-green-100 text-green-800";
    case "queued":
      return "bg-yellow-100 text-yellow-800";
    case "provisioning":
      return "bg-blue-100 text-blue-800";
    case "pausing":
    case "paused":
      return "bg-orange-100 text-orange-800";
    case "resuming":
      return "bg-blue-100 text-blue-800";
    case "stopping":
      return "bg-red-100 text-red-800";
    case "stopped":
    case "completed":
      return "bg-gray-100 text-gray-800";
    case "failed":
    case "cancelled":
      return "bg-red-100 text-red-800";
    default:
      return "bg-gray-100 text-gray-800";
  }
};

const formatDate = (dateString: string) => {
  try {
    return new Date(dateString).toLocaleString();
  } catch {
    return dateString;
  }
};

const getRepoName = (url?: string) => {
  if (!url) return "Unknown";
  try {
    const match = url.match(/\/([^\/]+)\.git$/);
    return match ? match[1] : url.split("/").pop() || "Unknown";
  } catch {
    return "Unknown";
  }
};

export const SessionCard: Component<SessionCardProps> = (props) => {
  const session = () => props.session;

  const canStop = () =>
    ["running", "queued", "provisioning", "paused"].includes(session().status);
  const canCancel = () =>
    ["queued", "provisioning", "running", "paused"].includes(session().status);

  return (
    <div
      class={`bg-white border rounded-lg shadow-sm hover:shadow-md transition-shadow cursor-pointer ${
        props.isSelected
          ? "ring-2 ring-blue-500 border-blue-500"
          : "border-gray-200"
      }`}
      onClick={props.onClick}
    >
      <div class="p-4">
        {/* Header with status and actions */}
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center space-x-2">
            <span
              class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(session().status)}`}
            >
              {session().status}
            </span>
            <span class="text-xs text-gray-500">{session().id.slice(-8)}</span>
          </div>

          <div class="flex space-x-1">
            <Show when={canStop()}>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  props.onStop?.();
                }}
                class="p-1 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded"
                title="Stop session"
                aria-label="Stop session"
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
                    d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 10h6m-6 4h6m-6 4h6"
                  />
                </svg>
              </button>
            </Show>

            <Show when={canCancel()}>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  props.onCancel?.();
                }}
                class="p-1 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded"
                title="Cancel session"
                aria-label="Cancel session"
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
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </Show>
          </div>
        </div>

        {/* Repository and agent info */}
        <div class="space-y-2 mb-3">
          <div class="flex items-center space-x-2">
            <svg
              class="w-4 h-4 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z"
              />
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M8 5a2 2 0 012-2h4a2 2 0 012 2v0M9 9h6"
              />
            </svg>
            <span class="text-sm font-medium text-gray-900 truncate">
              {getRepoName(session().repo.url)}
            </span>
            <Show when={session().repo.branch}>
              <span class="text-xs text-gray-500 bg-gray-100 px-1.5 py-0.5 rounded">
                {session().repo.branch}
              </span>
            </Show>
          </div>

          <div class="flex items-center space-x-2">
            <svg
              class="w-4 h-4 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
              />
            </svg>
            <span class="text-sm text-gray-600">
              {session().agent.type} ({session().agent.version})
            </span>
          </div>
        </div>

        {/* Prompt preview */}
        <div class="mb-3">
          <p class="text-sm text-gray-600 line-clamp-2">
            {session().prompt.length > 100
              ? `${session().prompt.slice(0, 100)}...`
              : session().prompt}
          </p>
        </div>

        {/* Footer with timestamp */}
        <div class="flex items-center justify-between text-xs text-gray-500">
          <span>{formatDate(session().createdAt)}</span>
          <span>{session().runtime.type}</span>
        </div>
      </div>
    </div>
  );
};
