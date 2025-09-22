import { Component } from 'solid-js';

interface SessionsPaneProps {
  selectedSessionId?: string;
}

export const SessionsPane: Component<SessionsPaneProps> = (props) => {
  const sessions = [
    {
      id: '01HVZ6K9T1N8S6M3V3Q3F0X5B7',
      status: 'running',
      task: 'Fix flaky tests in checkout service',
      agent: 'claude-code',
      repository: 'storefront',
      createdAt: '2025-01-01T12:00:00Z'
    },
    {
      id: '02HVZ6K9T1N8S6M3V3Q3F0X5B8',
      status: 'completed',
      task: 'Add error handling to user auth',
      agent: 'openhands',
      repository: 'api-gateway',
      createdAt: '2025-01-01T11:30:00Z'
    },
    {
      id: '03HVZ6K9T1N8S6M3V3Q3F0X5B9',
      status: 'failed',
      task: 'Optimize database queries',
      agent: 'claude-code',
      repository: 'user-service',
      createdAt: '2025-01-01T11:00:00Z'
    },
  ];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return 'bg-green-100 text-green-800';
      case 'completed': return 'bg-blue-100 text-blue-800';
      case 'failed': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div class="flex flex-col h-full">
      <div class="p-4 border-b border-gray-200">
        <h2 class="text-lg font-medium text-gray-900">Sessions</h2>
      </div>

      <div class="flex-1 overflow-y-auto">
        <div class="p-2">
          {sessions.map((session) => (
            <div
              key={session.id}
              class={`p-4 mb-3 border rounded-lg cursor-pointer transition-colors ${
                props.selectedSessionId === session.id
                  ? 'border-blue-500 bg-blue-50'
                  : 'border-gray-200 bg-white hover:bg-gray-50'
              }`}
              onClick={() => {
                // In a real app, this would navigate to the session details
                window.location.hash = `#session-${session.id}`;
              }}
            >
              <div class="flex items-start justify-between mb-2">
                <h3 class="font-medium text-gray-900 text-sm">{session.task}</h3>
                <span class={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(session.status)}`}>
                  {session.status}
                </span>
              </div>

              <div class="text-sm text-gray-600 space-y-1">
                <p><strong>Agent:</strong> {session.agent}</p>
                <p><strong>Repository:</strong> {session.repository}</p>
                <p><strong>Created:</strong> {new Date(session.createdAt).toLocaleString()}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
