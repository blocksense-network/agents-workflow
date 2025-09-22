import { Component } from 'solid-js';

export const RepositoriesPane: Component = () => {
  const repositories = [
    { id: '1', name: 'storefront', branch: 'main', url: 'git@github.com:acme/storefront.git' },
    { id: '2', name: 'api-gateway', branch: 'develop', url: 'git@github.com:acme/api-gateway.git' },
    { id: '3', name: 'user-service', branch: 'main', url: 'git@github.com:acme/user-service.git' },
  ];

  return (
    <div class="flex flex-col h-full">
      <div class="p-4 border-b border-gray-200">
        <h2 class="text-lg font-medium text-gray-900">Repositories</h2>
      </div>

      <div class="flex-1 overflow-y-auto">
        <div class="p-2">
          {repositories.map((repo) => (
            <div
              key={repo.id}
              class="p-3 mb-2 bg-gray-50 rounded-lg hover:bg-gray-100 cursor-pointer transition-colors"
            >
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="font-medium text-gray-900">{repo.name}</h3>
                  <p class="text-sm text-gray-500">{repo.branch}</p>
                </div>
                <button class="text-blue-600 hover:text-blue-800 text-sm font-medium">
                  + New Task
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
