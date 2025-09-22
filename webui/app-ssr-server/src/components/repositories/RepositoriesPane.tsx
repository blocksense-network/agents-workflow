import { Component } from 'solid-js';

export const RepositoriesPane: Component = () => {
  return (
    <div class="flex flex-col h-full">
      <div class="p-4 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Repositories</h2>
        <p class="text-sm text-gray-600 mt-1">Select a repository to create tasks</p>
      </div>

      <div class="flex-1 overflow-y-auto p-4">
        <div class="space-y-2">
          <div class="p-3 bg-gray-50 rounded-lg border border-gray-200">
            <div class="text-sm font-medium text-gray-900">Loading repositories...</div>
            <div class="text-xs text-gray-500 mt-1">This content will load with JavaScript enabled</div>
          </div>
        </div>
      </div>
    </div>
  );
};

