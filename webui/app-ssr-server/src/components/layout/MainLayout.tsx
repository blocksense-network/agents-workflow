import { Component, createSignal } from "solid-js";
import { A } from "@solidjs/router";

interface MainLayoutProps {
  children: any;
  currentPath?: string;
}

export const MainLayout: Component<MainLayoutProps> = (props) => {
  const [searchQuery, setSearchQuery] = createSignal("");

  const isActive = (path: string) => {
    return props.currentPath === path;
  };

  const handleSearch = (e: SubmitEvent) => {
    e.preventDefault();
    const query = searchQuery().trim();
    if (query) {
      // TODO: Implement global search functionality
      console.log("Searching for:", query);
      // For now, we'll just log the search query
      // In the future, this will navigate to search results or filter sessions
    }
  };

  return (
    <div class="h-screen flex flex-col bg-gray-50">
      {/* Top Navigation */}
      <header class="bg-white border-b border-gray-200 px-4 py-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <h1 class="text-xl font-semibold text-gray-900">Agents-Workflow</h1>

            {/* Global Search */}
            <form onSubmit={handleSearch} class="hidden md:flex items-center">
              <div class="relative">
                <input
                  type="text"
                  placeholder="Search sessions, repositories..."
                  value={searchQuery()}
                  onInput={(e) => setSearchQuery(e.currentTarget.value)}
                  class="w-64 pl-9 pr-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
                />
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                  <svg
                    class="h-4 w-4 text-gray-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                    ></path>
                  </svg>
                </div>
              </div>
              <button
                type="submit"
                class="ml-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 text-sm font-medium"
              >
                Search
              </button>
            </form>
          </div>

          <nav class="flex items-center space-x-6">
            <A
              href="/"
              class={`text-sm font-medium transition-colors ${
                isActive("/")
                  ? "text-blue-600"
                  : "text-gray-600 hover:text-gray-900"
              }`}
            >
              Dashboard
            </A>
            <A
              href="/sessions"
              class={`text-sm font-medium transition-colors ${
                isActive("/sessions")
                  ? "text-blue-600"
                  : "text-gray-600 hover:text-gray-900"
              }`}
            >
              Sessions
            </A>
            <A
              href="/create"
              class={`text-sm font-medium transition-colors ${
                isActive("/create")
                  ? "text-blue-600"
                  : "text-gray-600 hover:text-gray-900"
              }`}
            >
              Create Task
            </A>
            <A
              href="/agents"
              class={`text-sm font-medium transition-colors ${
                isActive("/agents")
                  ? "text-blue-600"
                  : "text-gray-600 hover:text-gray-900"
              }`}
            >
              Agents
            </A>
            <A
              href="/runtimes"
              class={`text-sm font-medium transition-colors ${
                isActive("/runtimes")
                  ? "text-blue-600"
                  : "text-gray-600 hover:text-gray-900"
              }`}
            >
              Runtimes
            </A>
            <A
              href="/hosts"
              class={`text-sm font-medium transition-colors ${
                isActive("/hosts")
                  ? "text-blue-600"
                  : "text-gray-600 hover:text-gray-900"
              }`}
            >
              Hosts
            </A>
            <A
              href="/settings"
              class={`text-sm font-medium transition-colors ${
                isActive("/settings")
                  ? "text-blue-600"
                  : "text-gray-600 hover:text-gray-900"
              }`}
            >
              Settings
            </A>
          </nav>
        </div>

        {/* Mobile Search */}
        <div class="md:hidden mt-3">
          <form onSubmit={handleSearch} class="flex items-center">
            <div class="relative flex-1">
              <input
                type="text"
                placeholder="Search sessions, repositories..."
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
                class="w-full pl-9 pr-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
              />
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg
                  class="h-4 w-4 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                  ></path>
                </svg>
              </div>
            </div>
            <button
              type="submit"
              class="ml-2 px-3 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 text-sm font-medium"
            >
              Search
            </button>
          </form>
        </div>
      </header>

      {/* Main Content - Three Pane Layout */}
      <div class="flex-1 flex overflow-hidden">{props.children}</div>
    </div>
  );
};
