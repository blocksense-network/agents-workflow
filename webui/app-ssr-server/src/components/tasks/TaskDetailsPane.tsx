import { Component } from 'solid-js';

interface TaskDetailsPaneProps {
  sessionId?: string;
}

export const TaskDetailsPane: Component<TaskDetailsPaneProps> = (props) => {
  return (
    <div class="flex flex-col h-full">
      <div class="p-4 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Task Details</h2>
        <p class="text-sm text-gray-600 mt-1">
          {props.sessionId ? `Session ${props.sessionId}` : 'Select a session to view details'}
        </p>
      </div>

      <div class="flex-1 overflow-y-auto p-4">
        {!props.sessionId ? (
          <div class="p-4 bg-gray-50 rounded-lg border border-gray-200 text-center">
            <div class="text-sm text-gray-600">No session selected</div>
            <div class="text-xs text-gray-500 mt-1">Click on a session to view its details</div>
          </div>
        ) : (
          <div class="space-y-4">
            <div class="p-4 bg-gray-50 rounded-lg border border-gray-200">
              <div class="text-sm font-medium text-gray-900">Loading task details...</div>
              <div class="text-xs text-gray-500 mt-1">Live logs and status will appear here</div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

