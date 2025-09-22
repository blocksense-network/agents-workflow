import { Component } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";

export const CreateTask: Component = () => {
  return (
    <MainLayout currentPath="/create">
      <div class="flex-1 p-6">
        <div class="max-w-4xl mx-auto">
          <h1 class="text-2xl font-bold text-gray-900 mb-6">Create New Task</h1>
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <p class="text-gray-600">
              Task creation form will be implemented here.
            </p>
            <p class="text-sm text-gray-500 mt-2">
              This will include repository selection, agent configuration, and
              runtime settings.
            </p>
          </div>
        </div>
      </div>
    </MainLayout>
  );
};
