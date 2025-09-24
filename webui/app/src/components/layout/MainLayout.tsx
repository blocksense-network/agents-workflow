import { Component, JSX } from 'solid-js';

interface MainLayoutProps {
  children: JSX.Element;
}

export const MainLayout: Component<MainLayoutProps> = (props) => {
  return (
    <div class="h-screen flex flex-col bg-gradient-to-br from-slate-50 to-blue-50">
      {/* Top Navigation */}
      <header class="bg-white/80 backdrop-blur-sm border-b border-slate-200/50 px-6 py-4 shadow-sm">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-3">
            <div class="w-8 h-8 bg-gradient-to-br from-blue-600 to-purple-600 rounded-lg flex items-center justify-center">
              <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
              </svg>
            </div>
            <div>
              <h1 class="text-xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                Agents-Workflow
              </h1>
              <p class="text-xs text-slate-500">AI Agent Management Platform</p>
            </div>
          </div>
          <nav class="flex space-x-1">
            <a
              href="/"
              class="px-4 py-2 rounded-lg text-sm font-medium text-slate-600 hover:text-slate-900 hover:bg-slate-100 transition-colors"
            >
              🏠 Dashboard
            </a>
            <a
              href="/sessions"
              class="px-4 py-2 rounded-lg text-sm font-medium text-slate-600 hover:text-slate-900 hover:bg-slate-100 transition-colors"
            >
              📊 Sessions
            </a>
            <a
              href="/create"
              class="px-4 py-2 rounded-lg text-sm font-medium bg-gradient-to-r from-blue-600 to-purple-600 text-white shadow-sm hover:shadow-md transition-all"
            >
              ➕ Create Task
            </a>
          </nav>
        </div>
      </header>

      {/* Main Content - Three Pane Layout */}
      <div class="flex-1 flex overflow-hidden p-6 gap-6">{props.children}</div>
    </div>
  );
};
