import { Title } from "@solidjs/meta";

export default function Home() {
  return (
    <>
      <Title>Agents-Workflow Dashboard</Title>
      <div class="p-6">
        <h1 class="text-2xl font-bold text-gray-900 mb-4">Welcome to Agents-Workflow</h1>
        <p class="text-gray-600 mb-6">
          Manage and monitor your agent coding sessions with real-time visibility and seamless IDE integration.
        </p>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
            <h3 class="text-lg font-semibold text-gray-900 mb-2">Create Tasks</h3>
            <p class="text-sm text-gray-600">Start new agent sessions with repository selection and configuration</p>
          </div>
          <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
            <h3 class="text-lg font-semibold text-gray-900 mb-2">Monitor Sessions</h3>
            <p class="text-sm text-gray-600">Track running sessions with live logs and status updates</p>
          </div>
          <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
            <h3 class="text-lg font-semibold text-gray-900 mb-2">Launch IDEs</h3>
            <p class="text-sm text-gray-600">One-click access to VS Code, Cursor, and Windsurf workspaces</p>
          </div>
        </div>
      </div>
    </>
  );
}

