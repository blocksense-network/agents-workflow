import { Component } from 'solid-js';

interface SessionsPaneProps {
  selectedSessionId?: string;
}

export const SessionsPane: Component<SessionsPaneProps> = (props) => {
  return (
    <div class="flex flex-col h-full">
      <div class="p-4 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Sessions</h2>
        <p class="text-sm text-gray-600 mt-1">Active and recent agent sessions</p>
      </div>

      <div class="flex-1 overflow-y-auto p-4">
        <div class="space-y-3">
          <div class="p-4 bg-gray-50 rounded-lg border border-gray-200">
            <div class="text-sm font-medium text-gray-900">Loading sessions...</div>
            <div class="text-xs text-gray-500 mt-1">Real-time session data will appear here</div>
          </div>
        </div>
      </div>
    </div>
  );
};

