import {
  Component,
  createResource,
  Show,
  For,
  createSignal,
  onMount,
  onCleanup,
  createEffect,
} from "solid-js";
import {
  apiClient,
  type Session,
  type LogEntry,
  type SessionEvent,
} from "../../lib/api.js";

interface TaskDetailsPaneProps {
  sessionId?: string;
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

export const TaskDetailsPane: Component<TaskDetailsPaneProps> = (props) => {
  const [activeTab, setActiveTab] = createSignal<
    "overview" | "logs" | "events"
  >("overview");
  const [logsRefreshTrigger, setLogsRefreshTrigger] = createSignal(0);
  const [realTimeLogs, setRealTimeLogs] = createSignal<LogEntry[]>([]);
  const [optimisticStatus, setOptimisticStatus] = createSignal<string | null>(
    null,
  );
  const [eventSource, setEventSource] = createSignal<EventSource | null>(null);
  const [connectionStatus, setConnectionStatus] = createSignal<
    "connected" | "connecting" | "disconnected" | "error"
  >("disconnected");
  const [reconnectAttempts, setReconnectAttempts] = createSignal(0);

  const [sessionData] = createResource(
    () => props.sessionId,
    async (sessionId) => {
      if (!sessionId) return null;
      try {
        return await apiClient.getSession(sessionId);
      } catch (error) {
        console.error("Failed to load session details:", error);
        return null;
      }
    },
  );

  const [logsData] = createResource(
    () => ({ sessionId: props.sessionId, refresh: logsRefreshTrigger() }),
    async ({ sessionId }) => {
      if (!sessionId) return null;
      try {
        return await apiClient.getSessionLogs(sessionId, 100);
      } catch (error) {
        console.error("Failed to load session logs:", error);
        return null;
      }
    },
  );

  // Function to establish SSE connection with error handling and reconnection
  const connectToSSE = (sessionId: string) => {
    setConnectionStatus("connecting");

    const es = apiClient.subscribeToSessionEvents(
      sessionId,
      (event: SessionEvent) => {
        if (event.type === "status") {
          // Update optimistic status if it matches our expected transition
          if (optimisticStatus() === event.status) {
            setOptimisticStatus(null);
          }
          // Refetch session data to get latest status
          sessionData.refetch();
        } else if (event.type === "log") {
          // Add new log entry to real-time logs
          setRealTimeLogs((prev) => [
            ...prev,
            {
              level: event.level!,
              message: event.message!,
              ts: event.ts,
            },
          ]);
        }
      },
    );

    // Handle connection opened
    es.onopen = () => {
      setConnectionStatus("connected");
      setReconnectAttempts(0);
    };

    // Handle connection errors
    es.onerror = (error) => {
      console.error("SSE connection error:", error);
      setConnectionStatus("error");

      // Close the connection
      es.close();

      // Attempt reconnection with exponential backoff (max 5 attempts)
      const attempts = reconnectAttempts();
      if (attempts < 5) {
        const delay = Math.min(1000 * Math.pow(2, attempts), 30000); // Max 30 seconds
        setTimeout(() => {
          setReconnectAttempts((prev) => prev + 1);
          connectToSSE(sessionId);
        }, delay);
      } else {
        setConnectionStatus("disconnected");
      }
    };

    setEventSource(es);
    return es;
  };

  // SSE subscription for real-time updates
  createEffect(() => {
    const sessionId = props.sessionId;

    // Clean up previous subscription
    if (eventSource()) {
      eventSource()!.close();
      setEventSource(null);
      setConnectionStatus("disconnected");
      setReconnectAttempts(0);
    }

    if (sessionId) {
      connectToSSE(sessionId);

      // Cleanup on effect re-run
      onCleanup(() => {
        if (eventSource()) {
          eventSource()!.close();
          setEventSource(null);
          setConnectionStatus("disconnected");
        }
      });
    }
  });

  // Auto-refresh logs for running sessions (fallback when SSE not available)
  onMount(() => {
    const interval = setInterval(() => {
      if (
        props.sessionId &&
        sessionData()?.status === "running" &&
        !eventSource()
      ) {
        setLogsRefreshTrigger((prev) => prev + 1);
      }
    }, 5000); // Refresh logs every 5 seconds for running sessions when no SSE

    return () => clearInterval(interval);
  });

  const session = () => sessionData();
  const currentStatus = () =>
    optimisticStatus() || session()?.status || "unknown";

  const canStop = () =>
    session() &&
    ["running", "queued", "provisioning", "paused"].includes(currentStatus());
  const canPause = () => session() && ["running"].includes(currentStatus());
  const canResume = () => session() && ["paused"].includes(currentStatus());

  const handleStop = async () => {
    if (!session()) return;
    try {
      // Optimistic UI update
      setOptimisticStatus("stopping");
      await apiClient.stopSession(session()!.id);
      // If SSE is available, status will be updated via events
      // Otherwise, refetch after a delay
      if (!eventSource()) {
        setTimeout(() => sessionData.refetch(), 1000);
      }
    } catch (error) {
      console.error("Failed to stop session:", error);
      setOptimisticStatus(null); // Revert optimistic update on error
    }
  };

  const handlePause = async () => {
    if (!session()) return;
    try {
      // Optimistic UI update
      setOptimisticStatus("pausing");
      await apiClient.pauseSession(session()!.id);
      // If SSE is available, status will be updated via events
      // Otherwise, refetch after a delay
      if (!eventSource()) {
        setTimeout(() => sessionData.refetch(), 1000);
      }
    } catch (error) {
      console.error("Failed to pause session:", error);
      setOptimisticStatus(null); // Revert optimistic update on error
    }
  };

  const handleResume = async () => {
    if (!session()) return;
    try {
      // Optimistic UI update
      setOptimisticStatus("resuming");
      await apiClient.resumeSession(session()!.id);
      // If SSE is available, status will be updated via events
      // Otherwise, refetch after a delay
      if (!eventSource()) {
        setTimeout(() => sessionData.refetch(), 1000);
      }
    } catch (error) {
      console.error("Failed to resume session:", error);
      setOptimisticStatus(null); // Revert optimistic update on error
    }
  };

  return (
    <div class="flex flex-col h-full">
      <div class="p-6 border-b border-slate-200/50">
        <div class="flex items-center space-x-3">
          <div class="w-10 h-10 bg-gradient-to-br from-orange-500 to-red-500 rounded-xl flex items-center justify-center">
            <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
          </div>
          <div>
            <h2 class="text-lg font-semibold text-slate-900">Task Details</h2>
            <p class="text-sm text-slate-500 mt-0.5">
              {props.sessionId
                ? `Session ${props.sessionId.slice(-8)}`
                : "Select a session to view details"}
            </p>
          </div>
        </div>

        {/* Session actions */}
        <Show when={session()}>
          <div class="mt-3 flex space-x-2">
            <Show when={canStop()}>
              <button
                onClick={handleStop}
                class="px-3 py-1 text-xs bg-red-600 text-white rounded hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500"
              >
                Stop
              </button>
            </Show>
            <Show when={canPause()}>
              <button
                onClick={handlePause}
                class="px-3 py-1 text-xs bg-orange-600 text-white rounded hover:bg-orange-700 focus:outline-none focus:ring-2 focus:ring-orange-500"
              >
                Pause
              </button>
            </Show>
            <Show when={canResume()}>
              <button
                onClick={handleResume}
                class="px-3 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                Resume
              </button>
            </Show>
          </div>
        </Show>
      </div>

      <div class="flex-1 overflow-y-auto">
        {!props.sessionId ? (
          <div class="p-4">
            <div class="p-4 bg-gray-50 rounded-lg border border-gray-200 text-center">
              <div class="text-sm text-gray-600">No session selected</div>
              <div class="text-xs text-gray-500 mt-1">
                Click on a session to view its details
              </div>
            </div>
          </div>
        ) : (
          <Show
            when={!sessionData.loading}
            fallback={
              <div class="p-4">
                <div class="p-4 bg-gray-50 rounded-lg border border-gray-200">
                  <div class="animate-pulse">
                    <div class="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
                    <div class="h-3 bg-gray-200 rounded w-1/2 mb-2"></div>
                    <div class="h-3 bg-gray-200 rounded w-full"></div>
                  </div>
                </div>
              </div>
            }
          >
            <Show
              when={session()}
              fallback={
                <div class="p-4">
                  <div class="p-4 bg-red-50 rounded-lg border border-red-200">
                    <div class="text-sm text-red-800">
                      Failed to load session details
                    </div>
                  </div>
                </div>
              }
            >
              {/* Tabs */}
              <div class="border-b border-gray-200">
                <nav class="flex">
                  <button
                    onClick={() => setActiveTab("overview")}
                    class={`px-4 py-2 text-sm font-medium border-b-2 ${
                      activeTab() === "overview"
                        ? "border-blue-500 text-blue-600"
                        : "border-transparent text-gray-500 hover:text-gray-700"
                    }`}
                  >
                    Overview
                  </button>
                  <button
                    onClick={() => setActiveTab("logs")}
                    class={`px-4 py-2 text-sm font-medium border-b-2 ${
                      activeTab() === "logs"
                        ? "border-blue-500 text-blue-600"
                        : "border-transparent text-gray-500 hover:text-gray-700"
                    }`}
                  >
                    Logs
                  </button>
                  <button
                    onClick={() => setActiveTab("events")}
                    class={`px-4 py-2 text-sm font-medium border-b-2 ${
                      activeTab() === "events"
                        ? "border-blue-500 text-blue-600"
                        : "border-transparent text-gray-500 hover:text-gray-700"
                    }`}
                  >
                    Events
                  </button>
                </nav>
              </div>

              {/* Tab Content */}
              <div class="p-4">
                {activeTab() === "overview" && (
                  <div class="space-y-4">
                    {/* Status and metadata */}
                    <div class="p-4 bg-white rounded-lg border border-gray-200">
                      <div class="flex items-center justify-between mb-3">
                        <span
                          class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(currentStatus())}`}
                        >
                          {currentStatus()}
                          {optimisticStatus() && (
                            <span class="ml-1 opacity-75">(updating...)</span>
                          )}
                        </span>
                        <span class="text-xs text-gray-500">
                          {session()!.id.slice(-8)}
                        </span>
                      </div>

                      <div class="space-y-2 text-sm">
                        <div class="flex justify-between">
                          <span class="text-gray-600">Created:</span>
                          <span class="font-medium">
                            {formatDate(session()!.createdAt)}
                          </span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-gray-600">Repository:</span>
                          <span class="font-medium">
                            {getRepoName(session()!.repo.url)}
                          </span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-gray-600">Branch:</span>
                          <span class="font-medium">
                            {session()!.repo.branch || "default"}
                          </span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-gray-600">Agent:</span>
                          <span class="font-medium">
                            {session()!.agent.type} ({session()!.agent.version})
                          </span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-gray-600">Runtime:</span>
                          <span class="font-medium">
                            {session()!.runtime.type}
                          </span>
                        </div>
                      </div>
                    </div>

                    {/* Prompt */}
                    <div class="p-4 bg-white rounded-lg border border-gray-200">
                      <h3 class="text-sm font-medium text-gray-900 mb-2">
                        Task Prompt
                      </h3>
                      <p class="text-sm text-gray-600 whitespace-pre-wrap">
                        {session()!.prompt}
                      </p>
                    </div>
                  </div>
                )}

                {activeTab() === "logs" && (
                  <div class="space-y-2">
                    <Show
                      when={!logsData.loading}
                      fallback={
                        <div class="text-center py-4">
                          <div class="text-sm text-gray-600">
                            Loading logs...
                          </div>
                        </div>
                      }
                    >
                      {(() => {
                        const fetchedLogs = logsData()?.logs || [];
                        const combinedLogs = [
                          ...fetchedLogs,
                          ...realTimeLogs(),
                        ].sort(
                          (a, b) =>
                            new Date(a.ts).getTime() - new Date(b.ts).getTime(),
                        );

                        return (
                          <Show
                            when={combinedLogs.length > 0}
                            fallback={
                              <div class="text-center py-8">
                                <div class="text-sm text-gray-600">
                                  No logs available
                                </div>
                              </div>
                            }
                          >
                            <div class="space-y-1 max-h-96 overflow-y-auto">
                              <For each={combinedLogs}>
                                {(log) => (
                                  <div class="text-xs font-mono p-2 bg-gray-50 rounded border">
                                    <span class="text-gray-500">
                                      {formatDate(log.ts)}
                                    </span>
                                    <span
                                      class={`ml-2 px-1 py-0.5 rounded text-xs ${
                                        log.level === "error"
                                          ? "bg-red-100 text-red-800"
                                          : log.level === "warn"
                                            ? "bg-yellow-100 text-yellow-800"
                                            : log.level === "info"
                                              ? "bg-blue-100 text-blue-800"
                                              : "bg-gray-100 text-gray-800"
                                      }`}
                                    >
                                      {log.level}
                                    </span>
                                    <span class="ml-2 text-gray-900">
                                      {log.message}
                                    </span>
                                  </div>
                                )}
                              </For>
                              {connectionStatus() === "connected" && (
                                <div class="text-xs text-green-600 p-2 bg-green-50 rounded border border-green-200">
                                  <div class="flex items-center">
                                    <div class="w-2 h-2 bg-green-500 rounded-full mr-2 animate-pulse"></div>
                                    Live: Receiving real-time log updates
                                  </div>
                                </div>
                              )}
                              {connectionStatus() === "connecting" && (
                                <div class="text-xs text-yellow-600 p-2 bg-yellow-50 rounded border border-yellow-200">
                                  <div class="flex items-center">
                                    <div class="w-2 h-2 bg-yellow-500 rounded-full mr-2 animate-pulse"></div>
                                    Connecting to live log stream...
                                  </div>
                                </div>
                              )}
                              {connectionStatus() === "error" && (
                                <div class="text-xs text-red-600 p-2 bg-red-50 rounded border border-red-200">
                                  <div class="flex items-center">
                                    <div class="w-2 h-2 bg-red-500 rounded-full mr-2"></div>
                                    Live log updates unavailable - using polling
                                    fallback
                                  </div>
                                </div>
                              )}
                            </div>
                          </Show>
                        );
                      })()}
                    </Show>
                  </div>
                )}

                {activeTab() === "events" && (
                  <div class="space-y-2">
                    {/* Connection status indicator */}
                    <div
                      class={`p-3 rounded-lg border mb-4 ${
                        connectionStatus() === "connected"
                          ? "bg-green-50 border-green-200"
                          : connectionStatus() === "connecting"
                            ? "bg-yellow-50 border-yellow-200"
                            : connectionStatus() === "error"
                              ? "bg-red-50 border-red-200"
                              : "bg-gray-50 border-gray-200"
                      }`}
                    >
                      <div class="flex items-center text-sm">
                        <div
                          class={`w-2 h-2 rounded-full mr-2 ${
                            connectionStatus() === "connected"
                              ? "bg-green-500 animate-pulse"
                              : connectionStatus() === "connecting"
                                ? "bg-yellow-500 animate-pulse"
                                : connectionStatus() === "error"
                                  ? "bg-red-500"
                                  : "bg-gray-500"
                          }`}
                        ></div>
                        <span
                          class={`${
                            connectionStatus() === "connected"
                              ? "text-green-800"
                              : connectionStatus() === "connecting"
                                ? "text-yellow-800"
                                : connectionStatus() === "error"
                                  ? "text-red-800"
                                  : "text-gray-800"
                          }`}
                        >
                          {connectionStatus() === "connected" &&
                            "Connected to real-time event stream"}
                          {connectionStatus() === "connecting" &&
                            "Connecting to event stream..."}
                          {connectionStatus() === "error" &&
                            `Connection error (attempt ${reconnectAttempts() + 1}/5)`}
                          {connectionStatus() === "disconnected" &&
                            "Disconnected from event stream"}
                        </span>
                      </div>
                      {connectionStatus() === "error" &&
                        reconnectAttempts() < 5 && (
                          <div class="text-xs text-red-600 mt-1">
                            Reconnecting in{" "}
                            {Math.min(
                              1000 * Math.pow(2, reconnectAttempts()),
                              30000,
                            ) / 1000}{" "}
                            seconds...
                          </div>
                        )}
                    </div>

                    <div class="text-center py-4">
                      <div class="text-sm text-gray-600">
                        {connectionStatus() === "connected"
                          ? "Real-time events will appear here as they happen"
                          : "Waiting for connection to display events"}
                      </div>
                      <div class="text-xs text-gray-500 mt-1">
                        Status changes, log entries, and progress updates
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </Show>
          </Show>
        )}
      </div>
    </div>
  );
};
