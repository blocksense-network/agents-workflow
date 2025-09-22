import { Component } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";

export const Settings: Component = () => {
  return (
    <MainLayout currentPath="/settings">
      <div class="flex-1 p-6">
        <div class="max-w-4xl mx-auto">
          <h1 class="text-2xl font-bold text-gray-900 mb-6">Settings</h1>
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <p class="text-gray-600">
              Settings panel will be implemented here.
            </p>
            <p class="text-sm text-gray-500 mt-2">
              This will include tenant configuration, RBAC, API keys, and IDE
              integration settings.
            </p>
          </div>
        </div>
      </div>
    </MainLayout>
  );
};
