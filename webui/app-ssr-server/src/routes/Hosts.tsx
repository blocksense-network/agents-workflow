import { Component } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";

export const Hosts: Component = () => {
  return (
    <MainLayout currentPath="/hosts">
      <div class="flex-1 p-6">
        <div class="max-w-6xl mx-auto">
          <h1 class="text-2xl font-bold text-gray-900 mb-6">Execution Hosts</h1>
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-8 text-center">
            <div class="text-gray-500">
              <svg class="w-12 h-12 mx-auto mb-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
              </svg>
              <h3 class="text-lg font-medium text-gray-900 mb-2">Host Management</h3>
              <p class="text-sm text-gray-600 mb-4">
                Monitor and manage execution hosts, snapshot capabilities, and system resources.
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
