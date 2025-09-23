import { Component, createSignal } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";
import { TaskCreationForm } from "../components/tasks/TaskCreationForm.js";

export const CreateTask: Component = () => {
  const [taskCreated, setTaskCreated] = createSignal<string | null>(null);

  const handleTaskCreated = (taskId: string) => {
    setTaskCreated(taskId);
  };

  const handleCreateAnother = () => {
    setTaskCreated(null);
  };

  return (
    <MainLayout currentPath="/create">
      <div class="flex-1 p-6">
        <div class="max-w-4xl mx-auto">
          <h1 class="text-2xl font-bold text-gray-900 mb-6">Create New Task</h1>

          {taskCreated() ? (
            <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
              <div class="text-center">
                <div class="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-green-100 mb-4">
                  <svg
                    class="h-6 w-6 text-green-600"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                </div>
                <h3 class="text-lg font-medium text-gray-900 mb-2">
                  Task Created Successfully!
                </h3>
                <p class="text-sm text-gray-600 mb-4">
                  Task ID:{" "}
                  <code class="bg-gray-100 px-2 py-1 rounded text-xs">
                    {taskCreated()}
                  </code>
                </p>
                <div class="flex justify-center space-x-3">
                  <button
                    onClick={handleCreateAnother}
                    class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                  >
                    Create Another Task
                  </button>
                  <a
                    href={`/sessions?highlight=${taskCreated()}`}
                    class="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                  >
                    View Sessions
                  </a>
                </div>
              </div>
            </div>
          ) : (
            <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
              <TaskCreationForm onTaskCreated={handleTaskCreated} />
            </div>
          )}
        </div>
      </div>
    </MainLayout>
  );
};
