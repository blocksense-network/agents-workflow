import {
  Component,
  createResource,
  createSignal,
  For,
  Show,
  onMount,
} from "solid-js";
import { apiClient, type Session } from "../../lib/api.js";
import { SessionCard } from "./SessionCard.js";

interface TaskFeedPaneProps {
  selectedSessionId?: string;
  collapsed?: boolean;
  onToggleCollapse?: () => void;
  onSessionSelect?: (sessionId: string) => void;
}

export const TaskFeedPane: Component<TaskFeedPaneProps> = (props) => {
  const [statusFilter, setStatusFilter] = createSignal<string>("");
  const [refreshTrigger, setRefreshTrigger] = createSignal(0);

  const [sessionsData] = createResource(
    () => ({ filter: statusFilter(), refresh: refreshTrigger() }),
    async ({ filter }) => {
      try {
        const params: any = {};
        if (filter) params.status = filter;
        params.perPage = 50; // Load more sessions
        return await apiClient.listSessions(params);
      } catch (error) {
        console.error("Failed to load sessions:", error);
        return {
          items: [],
          pagination: { page: 1, perPage: 20, total: 0, totalPages: 1 },
        };
      }
    },
  );

  // Auto-refresh sessions every 30 seconds
  onMount(() => {
    const interval = setInterval(() => {
      setRefreshTrigger((prev) => prev + 1);
    }, 30000);

    return () => clearInterval(interval);
  });

  const handleStopSession = async (sessionId: string) => {
    try {
      await apiClient.stopSession(sessionId);
      setRefreshTrigger((prev) => prev + 1); // Refresh the list
    } catch (error) {
      console.error("Failed to stop session:", error);
      // TODO: Show error notification
    }
  };

  const handleCancelSession = async (sessionId: string) => {
    if (!confirm("Are you sure you want to cancel this session?")) {
      return;
    }

    try {
      await apiClient.cancelSession(sessionId);
      setRefreshTrigger((prev) => prev + 1); // Refresh the list
    } catch (error) {
      console.error("Failed to cancel session:", error);
      // TODO: Show error notification
    }
  };

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
            Task Feed ({sessionsData()?.pagination.total || 0})
          </div>
        </div>
      </div>
    );
  }

  return (
    <div class="flex flex-col h-full">
      <div class="p-4 border-b border-gray-200">
        <div class="flex items-center justify-between mb-3">
          <div>
            <h2 class="text-lg font-semibold text-gray-900">Task Feed</h2>
            <p class="text-sm text-gray-600 mt-1">
              Chronological task feed with live status
            </p>
          </div>
          <button
            onClick={props.onToggleCollapse}
            class="p-1 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded"
            title="Collapse Task Feed"
            aria-label="Collapse Task Feed pane"
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

        {/* Status filter */}
        <div>
          <label
            for="status-filter"
            class="block text-xs font-medium text-gray-700 mb-1"
          >
            Filter by Status
          </label>
          <select
            id="status-filter"
            value={statusFilter()}
            onChange={(e) => setStatusFilter(e.currentTarget.value)}
            class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            <option value="">All Sessions</option>
            <option value="running">Running</option>
            <option value="queued">Queued</option>
            <option value="provisioning">Provisioning</option>
            <option value="paused">Paused</option>
            <option value="completed">Completed</option>
            <option value="failed">Failed</option>
            <option value="cancelled">Cancelled</option>
          </select>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto">
        <Show
          when={!sessionsData.loading}
          fallback={
            <div class="p-4">
              <div class="space-y-3">
                {Array.from({ length: 3 }).map(() => (
                  <div class="p-4 bg-gray-50 rounded-lg border border-gray-200 animate-pulse">
                    <div class="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
                    <div class="h-3 bg-gray-200 rounded w-1/2 mb-2"></div>
                    <div class="h-3 bg-gray-200 rounded w-full"></div>
                  </div>
                ))}
              </div>
            </div>
          }
        >
          <div class="p-4">
            <Show
              when={sessionsData()?.items.length > 0}
              fallback={
                <div class="text-center py-8">
                  <svg
                    class="mx-auto h-12 w-12 text-gray-400"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                    />
                  </svg>
                  <h3 class="mt-2 text-sm font-medium text-gray-900">
                    No sessions
                  </h3>
                  <p class="mt-1 text-sm text-gray-500">
                    {statusFilter()
                      ? `No sessions with status "${statusFilter()}"`
                      : "Get started by creating a new task."}
                  </p>
                  <Show when={statusFilter()}>
                    <button
                      onClick={() => setStatusFilter("")}
                      class="mt-3 text-sm text-blue-600 hover:text-blue-500"
                    >
                      Clear filter
                    </button>
                  </Show>
                </div>
              }
            >
              <div class="space-y-3">
                <For each={sessionsData()?.items}>
                  {(session) => (
                    <SessionCard
                      session={session}
                      isSelected={props.selectedSessionId === session.id}
                      onClick={() => props.onSessionSelect?.(session.id)}
                      onStop={() => handleStopSession(session.id)}
                      onCancel={() => handleCancelSession(session.id)}
                    />
                  )}
                </For>
              </div>

              <Show when={sessionsData()?.pagination.totalPages > 1}>
                <div class="mt-4 text-center text-sm text-gray-500">
                  Showing {sessionsData()?.items.length} of{" "}
                  {sessionsData()?.pagination.total} sessions
                </div>
              </Show>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
};
