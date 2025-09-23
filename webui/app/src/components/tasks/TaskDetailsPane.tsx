import { Component, Show } from 'solid-js';

interface TaskDetailsPaneProps {
  sessionId?: string;
}

export const TaskDetailsPane: Component<TaskDetailsPaneProps> = (props) => {
  // Mock session details
  const sessionDetails = {
    id: '01HVZ6K9T1N8S6M3V3Q3F0X5B7',
    status: 'running',
    task: 'Fix flaky tests in checkout service and improve logging.',
    agent: 'claude-code',
    repository: 'storefront',
    branch: 'main',
    createdAt: '2025-01-01T12:00:00Z',
    logs: [
      { timestamp: '2025-01-01T12:00:00Z', level: 'info', message: 'Session started' },
      { timestamp: '2025-01-01T12:00:05Z', level: 'info', message: 'Agent initialized' },
      { timestamp: '2025-01-01T12:00:10Z', level: 'info', message: 'Running tests...' },
      { timestamp: '2025-01-01T12:00:15Z', level: 'info', message: 'Found 3 failing tests' },
    ],
  };

  return (
    <div class="flex flex-col h-full">
      <Show
        when={props.sessionId}
        fallback={
          <div class="flex items-center justify-center h-full text-gray-500">
            <div class="text-center">
              <div class="text-4xl mb-4">📋</div>
              <p>Select a session to view details</p>
            </div>
          </div>
        }
      >
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-medium text-gray-900">Session Details</h2>
        </div>

        <div class="flex-1 overflow-y-auto p-4">
          <div class="space-y-4">
            <div>
              <h3 class="font-medium text-gray-900 mb-2">Task</h3>
              <p class="text-gray-700">{sessionDetails.task}</p>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div>
                <h4 class="font-medium text-gray-900 mb-1">Agent</h4>
                <p class="text-gray-700">{sessionDetails.agent}</p>
              </div>
              <div>
                <h4 class="font-medium text-gray-900 mb-1">Status</h4>
                <span class="px-2 py-1 bg-green-100 text-green-800 rounded-full text-sm">
                  {sessionDetails.status}
                </span>
              </div>
              <div>
                <h4 class="font-medium text-gray-900 mb-1">Repository</h4>
                <p class="text-gray-700">{sessionDetails.repository}</p>
              </div>
              <div>
                <h4 class="font-medium text-gray-900 mb-1">Branch</h4>
                <p class="text-gray-700">{sessionDetails.branch}</p>
              </div>
            </div>

            <div>
              <h3 class="font-medium text-gray-900 mb-2">Live Logs</h3>
              <div class="bg-gray-900 text-green-400 p-4 rounded-lg font-mono text-sm max-h-64 overflow-y-auto">
                {sessionDetails.logs.map((log) => (
                  <div key={log.timestamp} class="mb-1">
                    <span class="text-gray-400">
                      [{new Date(log.timestamp).toLocaleTimeString()}]
                    </span>
                    <span
                      class={`ml-2 ${
                        log.level === 'error'
                          ? 'text-red-400'
                          : log.level === 'warn'
                            ? 'text-yellow-400'
                            : 'text-green-400'
                      }`}
                    >
                      {log.message}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            <div class="flex space-x-2 pt-4">
              <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                Stop Session
              </button>
              <button class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors">
                View Workspace
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
