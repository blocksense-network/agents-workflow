import { Component } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";

export const Agents: Component = () => {
  return (
    <MainLayout currentPath="/agents">
      <div class="flex-1 p-6">
        <div class="max-w-6xl mx-auto">
          <h1 class="text-2xl font-bold text-gray-900 mb-6">Agents</h1>
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-8 text-center">
            <div class="text-gray-500">
              <svg class="w-12 h-12 mx-auto mb-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
              <h3 class="text-lg font-medium text-gray-900 mb-2">Agents Management</h3>
              <p class="text-sm text-gray-600 mb-4">
                Configure and manage available agent types and versions for task execution.
              </p>
              <p class="text-xs text-gray-500">
                This feature is not yet implemented in the current demo.
              </p>
            </div>
          </div>
        </div>
      </div>
    </MainLayout>
  );
};
