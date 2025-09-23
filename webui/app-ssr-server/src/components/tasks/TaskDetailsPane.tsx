import { Component, createResource, Show, For, createSignal, onMount } from "solid-js";
import { apiClient, type Session, type LogEntry } from "../../lib/api.js";

interface TaskDetailsPaneProps {
  sessionId?: string;
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'running':
      return 'bg-green-100 text-green-800';
    case 'queued':
      return 'bg-yellow-100 text-yellow-800';
    case 'provisioning':
      return 'bg-blue-100 text-blue-800';
    case 'pausing':
    case 'paused':
      return 'bg-orange-100 text-orange-800';
    case 'resuming':
      return 'bg-blue-100 text-blue-800';
    case 'stopping':
      return 'bg-red-100 text-red-800';
    case 'stopped':
    case 'completed':
      return 'bg-gray-100 text-gray-800';
    case 'failed':
    case 'cancelled':
      return 'bg-red-100 text-red-800';
    default:
      return 'bg-gray-100 text-gray-800';
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
  if (!url) return 'Unknown';
  try {
    const match = url.match(/\/([^\/]+)\.git$/);
    return match ? match[1] : url.split('/').pop() || 'Unknown';
  } catch {
    return 'Unknown';
  }
};

export const TaskDetailsPane: Component<TaskDetailsPaneProps> = (props) => {
  const [activeTab, setActiveTab] = createSignal<'overview' | 'logs' | 'events'>('overview');
  const [logsRefreshTrigger, setLogsRefreshTrigger] = createSignal(0);

  const [sessionData] = createResource(
    () => props.sessionId,
    async (sessionId) => {
      if (!sessionId) return null;
      try {
        return await apiClient.getSession(sessionId);
      } catch (error) {
        console.error('Failed to load session details:', error);
        return null;
      }
    }
  );

  const [logsData] = createResource(
    () => ({ sessionId: props.sessionId, refresh: logsRefreshTrigger() }),
    async ({ sessionId }) => {
      if (!sessionId) return null;
      try {
        return await apiClient.getSessionLogs(sessionId, 100);
      } catch (error) {
        console.error('Failed to load session logs:', error);
        return null;
      }
    }
  );

  // Auto-refresh logs for running sessions
  onMount(() => {
    const interval = setInterval(() => {
      if (props.sessionId && sessionData()?.status === 'running') {
        setLogsRefreshTrigger(prev => prev + 1);
      }
    }, 5000); // Refresh logs every 5 seconds for running sessions

    return () => clearInterval(interval);
  });

  const session = () => sessionData();

  const canStop = () => session() && ['running', 'queued', 'provisioning', 'paused'].includes(session()!.status);
  const canPause = () => session() && ['running'].includes(session()!.status);
  const canResume = () => session() && ['paused'].includes(session()!.status);

  const handleStop = async () => {
    if (!session()) return;
    try {
      await apiClient.stopSession(session()!.id);
      sessionData.refetch(); // Refresh session data
    } catch (error) {
      console.error('Failed to stop session:', error);
    }
  };

  const handlePause = async () => {
    if (!session()) return;
    try {
      await apiClient.pauseSession(session()!.id);
      sessionData.refetch(); // Refresh session data
    } catch (error) {
      console.error('Failed to pause session:', error);
    }
  };

  const handleResume = async () => {
    if (!session()) return;
    try {
      await apiClient.resumeSession(session()!.id);
      sessionData.refetch(); // Refresh session data
    } catch (error) {
      console.error('Failed to resume session:', error);
    }
  };

  return (
    <div class="flex flex-col h-full">
      <div class="p-4 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Task Details</h2>
        <p class="text-sm text-gray-600 mt-1">
          {props.sessionId
            ? `Session ${props.sessionId.slice(-8)}`
            : "Select a session to view details"}
        </p>

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
                    <div class="text-sm text-red-800">Failed to load session details</div>
                  </div>
                </div>
              }
            >
              {/* Tabs */}
              <div class="border-b border-gray-200">
                <nav class="flex">
                  <button
                    onClick={() => setActiveTab('overview')}
                    class={`px-4 py-2 text-sm font-medium border-b-2 ${
                      activeTab() === 'overview'
                        ? 'border-blue-500 text-blue-600'
                        : 'border-transparent text-gray-500 hover:text-gray-700'
                    }`}
                  >
                    Overview
                  </button>
                  <button
                    onClick={() => setActiveTab('logs')}
                    class={`px-4 py-2 text-sm font-medium border-b-2 ${
                      activeTab() === 'logs'
                        ? 'border-blue-500 text-blue-600'
                        : 'border-transparent text-gray-500 hover:text-gray-700'
                    }`}
                  >
                    Logs
                  </button>
                  <button
                    onClick={() => setActiveTab('events')}
                    class={`px-4 py-2 text-sm font-medium border-b-2 ${
                      activeTab() === 'events'
                        ? 'border-blue-500 text-blue-600'
                        : 'border-transparent text-gray-500 hover:text-gray-700'
                    }`}
                  >
                    Events
                  </button>
                </nav>
              </div>

              {/* Tab Content */}
              <div class="p-4">
                {activeTab() === 'overview' && (
                  <div class="space-y-4">
                    {/* Status and metadata */}
                    <div class="p-4 bg-white rounded-lg border border-gray-200">
                      <div class="flex items-center justify-between mb-3">
                        <span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(session()!.status)}`}>
                          {session()!.status}
                        </span>
                        <span class="text-xs text-gray-500">
                          {session()!.id.slice(-8)}
                        </span>
                      </div>

                      <div class="space-y-2 text-sm">
                        <div class="flex justify-between">
                          <span class="text-gray-600">Created:</span>
                          <span class="font-medium">{formatDate(session()!.createdAt)}</span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-gray-600">Repository:</span>
                          <span class="font-medium">{getRepoName(session()!.repo.url)}</span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-gray-600">Branch:</span>
                          <span class="font-medium">{session()!.repo.branch || 'default'}</span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-gray-600">Agent:</span>
                          <span class="font-medium">{session()!.agent.type} ({session()!.agent.version})</span>
                        </div>
                        <div class="flex justify-between">
                          <span class="text-gray-600">Runtime:</span>
                          <span class="font-medium">{session()!.runtime.type}</span>
                        </div>
                      </div>
                    </div>

                    {/* Prompt */}
                    <div class="p-4 bg-white rounded-lg border border-gray-200">
                      <h3 class="text-sm font-medium text-gray-900 mb-2">Task Prompt</h3>
                      <p class="text-sm text-gray-600 whitespace-pre-wrap">{session()!.prompt}</p>
                    </div>
                  </div>
                )}

                {activeTab() === 'logs' && (
                  <div class="space-y-2">
                    <Show
                      when={!logsData.loading}
                      fallback={
                        <div class="text-center py-4">
                          <div class="text-sm text-gray-600">Loading logs...</div>
                        </div>
                      }
                    >
                      <Show
                        when={logsData()?.logs.length > 0}
                        fallback={
                          <div class="text-center py-8">
                            <div class="text-sm text-gray-600">No logs available</div>
                          </div>
                        }
                      >
                        <div class="space-y-1 max-h-96 overflow-y-auto">
                          <For each={logsData()?.logs}>
                            {(log) => (
                              <div class="text-xs font-mono p-2 bg-gray-50 rounded border">
                                <span class="text-gray-500">{formatDate(log.ts)}</span>
                                <span class={`ml-2 px-1 py-0.5 rounded text-xs ${
                                  log.level === 'error' ? 'bg-red-100 text-red-800' :
                                  log.level === 'warn' ? 'bg-yellow-100 text-yellow-800' :
                                  log.level === 'info' ? 'bg-blue-100 text-blue-800' :
                                  'bg-gray-100 text-gray-800'
                                }`}>
                                  {log.level}
                                </span>
                                <span class="ml-2 text-gray-900">{log.message}</span>
                              </div>
                            )}
                          </For>
                        </div>
                      </Show>
                    </Show>
                  </div>
                )}

                {activeTab() === 'events' && (
                  <div class="space-y-2">
                    <div class="text-center py-8">
                      <div class="text-sm text-gray-600">Events will be displayed here</div>
                      <div class="text-xs text-gray-500 mt-1">Real-time event streaming coming in W4</div>
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
