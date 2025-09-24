import { Component } from 'solid-js';

export const RepositoriesPane: Component = () => {
  const repositories = [
    { id: '1', name: 'storefront', branch: 'main', url: 'git@github.com:acme/storefront.git' },
    { id: '2', name: 'api-gateway', branch: 'develop', url: 'git@github.com:acme/api-gateway.git' },
    { id: '3', name: 'user-service', branch: 'main', url: 'git@github.com:acme/user-service.git' },
  ];

  return (
    <div class="flex flex-col h-full">
      {/* Header */}
      <div class="p-6 border-b border-slate-200/50">
        <div class="flex items-center space-x-3">
          <div class="w-10 h-10 bg-gradient-to-br from-emerald-500 to-teal-500 rounded-xl flex items-center justify-center">
            <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M3 3h18v18H3V3zm16 16V5H5v14h14zM9 7h6v2H9V7zm0 4h6v2H9v-2zm0 4h4v2H9v-2z" />
            </svg>
          </div>
          <div>
            <h2 class="text-lg font-semibold text-slate-900">Repositories</h2>
            <p class="text-sm text-slate-500">Available codebases</p>
          </div>
        </div>
      </div>

      {/* Repositories List */}
      <div class="flex-1 overflow-y-auto p-4">
        <div class="space-y-3">
          {repositories.map((repo) => (
            <div
              key={repo.id}
              class="p-4 bg-white/50 rounded-xl border border-slate-200/50 hover:border-slate-300 hover:bg-white/80 cursor-pointer transition-all duration-200 hover:shadow-sm"
            >
              <div class="flex items-center justify-between mb-2">
                <div class="flex items-center space-x-3">
                  <div class="w-8 h-8 bg-gradient-to-br from-slate-400 to-slate-500 rounded-lg flex items-center justify-center">
                    <svg class="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
                    </svg>
                  </div>
                  <div>
                    <h3 class="font-semibold text-slate-900 text-sm">{repo.name}</h3>
                    <p class="text-xs text-slate-500 flex items-center space-x-1">
                      <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                        <path
                          fill-rule="evenodd"
                          d="M12.395 2.553a1 1 0 00-1.45-.385c-.345.23-.614.558-.822.88-.214.33-.403.713-.57 1.116-.334.804-.614 1.768-.84 2.734a31.365 31.365 0 00-.613 3.58 2.64 2.64 0 01-.945-1.067c-.328-.68-.398-1.534-.398-2.654A1 1 0 005.05 6.05 6.981 6.981 0 003 11a7 7 0 1011.95-4.95c-.592-.591-.98-.985-1.348-1.467-.363-.476-.724-1.063-1.207-2.03zM12.12 15.12A3 3 0 017 13s.879.5 2.5.5c0-1 .5-4 1.25-4.5.5 1 .786 1.293 1.371 1.879A2.99 2.99 0 0113 13a2.99 2.99 0 01-.879 2.121z"
                          clip-rule="evenodd"
                        />
                      </svg>
                      <span>{repo.branch}</span>
                    </p>
                  </div>
                </div>
                <button class="px-3 py-1 bg-gradient-to-r from-blue-500 to-purple-500 text-white text-xs font-medium rounded-lg hover:shadow-md transition-all">
                  ➕ Task
                </button>
              </div>

              <div class="text-xs text-slate-400 font-mono truncate">{repo.url}</div>
            </div>
          ))}
        </div>

        {/* Add repository hint */}
        <div class="mt-6 text-center">
          <div class="text-sm text-slate-400">
            🔗 Connect more repositories to expand your workflow options
          </div>
        </div>
      </div>
    </div>
  );
};
