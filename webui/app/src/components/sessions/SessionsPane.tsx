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
      createdAt: '2025-01-01T12:00:00Z',
    },
    {
      id: '02HVZ6K9T1N8S6M3V3Q3F0X5B8',
      status: 'completed',
      task: 'Add error handling to user auth',
      agent: 'openhands',
      repository: 'api-gateway',
      createdAt: '2025-01-01T11:30:00Z',
    },
    {
      id: '03HVZ6K9T1N8S6M3V3Q3F0X5B9',
      status: 'failed',
      task: 'Optimize database queries',
      agent: 'claude-code',
      repository: 'user-service',
      createdAt: '2025-01-01T11:00:00Z',
    },
  ];

  const getStatusConfig = (status: string) => {
    switch (status) {
      case 'running':
        return {
          bg: 'bg-emerald-50 border-emerald-200',
          text: 'text-emerald-700',
          icon: '🟢',
          label: 'Running',
        };
      case 'completed':
        return {
          bg: 'bg-blue-50 border-blue-200',
          text: 'text-blue-700',
          icon: '✅',
          label: 'Completed',
        };
      case 'failed':
        return {
          bg: 'bg-red-50 border-red-200',
          text: 'text-red-700',
          icon: '❌',
          label: 'Failed',
        };
      default:
        return {
          bg: 'bg-gray-50 border-gray-200',
          text: 'text-gray-700',
          icon: '⏳',
          label: status,
        };
    }
  };

  const getAgentIcon = (agent: string) => {
    switch (agent) {
      case 'claude-code':
        return '🤖';
      case 'openhands':
        return '👐';
      default:
        return '👤';
    }
  };

  return (
    <div class="flex flex-col h-full">
      {/* Header */}
      <div class="p-6 border-b border-slate-200/50">
        <div class="flex items-center space-x-3">
          <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-500 rounded-xl flex items-center justify-center">
            <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z" />
            </svg>
          </div>
          <div>
            <h2 class="text-lg font-semibold text-slate-900">Active Sessions</h2>
            <p class="text-sm text-slate-500">Monitor your AI agent workflows</p>
          </div>
        </div>
      </div>

      {/* Sessions List */}
      <div class="flex-1 overflow-y-auto p-4">
        <div class="space-y-3">
          {sessions.map((session) => {
            const statusConfig = getStatusConfig(session.status);
            const isSelected = props.selectedSessionId === session.id;

            return (
              <div
                key={session.id}
                class={`p-4 rounded-xl border-2 cursor-pointer transition-all duration-200 hover:shadow-md ${
                  isSelected
                    ? 'border-blue-300 bg-gradient-to-r from-blue-50 to-indigo-50 shadow-md'
                    : 'border-slate-200/50 bg-white/50 hover:border-slate-300 hover:bg-white/80'
                }`}
                onClick={() => {
                  window.location.hash = `#session-${session.id}`;
                }}
              >
                {/* Header */}
                <div class="flex items-start justify-between mb-3">
                  <div class="flex-1">
                    <h3 class="font-semibold text-slate-900 text-sm leading-tight mb-1">
                      {session.task}
                    </h3>
                    <div class="flex items-center space-x-2 text-xs text-slate-500">
                      <span>{getAgentIcon(session.agent)}</span>
                      <span>{session.agent}</span>
                      <span>•</span>
                      <span>📁 {session.repository}</span>
                    </div>
                  </div>
                  <div
                    class={`px-3 py-1 rounded-full text-xs font-medium flex items-center space-x-1 ${statusConfig.bg} ${statusConfig.text}`}
                  >
                    <span>{statusConfig.icon}</span>
                    <span>{statusConfig.label}</span>
                  </div>
                </div>

                {/* Footer */}
                <div class="flex items-center justify-between text-xs text-slate-400">
                  <span class="flex items-center space-x-1">
                    <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                      <path
                        fill-rule="evenodd"
                        d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z"
                        clip-rule="evenodd"
                      />
                    </svg>
                    <span>{new Date(session.createdAt).toLocaleString()}</span>
                  </span>
                  <span class="text-slate-300 font-mono">{session.id.slice(-6)}</span>
                </div>
              </div>
            );
          })}
        </div>

        {/* Empty state hint */}
        <div class="mt-6 text-center">
          <div class="text-sm text-slate-400">
            💡 Click on any session to view detailed logs and status
          </div>
        </div>
      </div>
    </div>
  );
};
