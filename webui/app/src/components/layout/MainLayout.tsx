import { Component, JSX } from 'solid-js';

interface MainLayoutProps {
  children: JSX.Element;
}

export const MainLayout: Component<MainLayoutProps> = (props) => {
  return (
    <div class="h-screen flex flex-col bg-gray-50">
      {/* Top Navigation */}
      <header class="bg-white border-b border-gray-200 px-4 py-3">
        <div class="flex items-center justify-between">
          <h1 class="text-xl font-semibold text-gray-900">Agents-Workflow</h1>
          <nav class="flex space-x-4">
            <a href="/" class="text-gray-600 hover:text-gray-900">
              Dashboard
            </a>
            <a href="/sessions" class="text-gray-600 hover:text-gray-900">
              Sessions
            </a>
            <a href="/create" class="text-gray-600 hover:text-gray-900">
              Create Task
            </a>
          </nav>
        </div>
      </header>

      {/* Main Content - Three Pane Layout */}
      <div class="flex-1 flex overflow-hidden">{props.children}</div>
    </div>
  );
};
