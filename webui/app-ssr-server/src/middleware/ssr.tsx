// Simple SSR using plain HTML strings to avoid SolidJS client-only API issues

export const ssrMiddleware = async (req: any, res: any, next: any) => {
  try {
    // Skip SSR for API routes and static assets
    if (req.url.startsWith("/api") || req.url.includes(".")) {
      return next();
    }

    // Only serve HTML for known application routes, otherwise let Express handle 404
    const knownRoutes = ["/", "/sessions", "/create", "/settings"];
    if (!knownRoutes.includes(req.url)) {
      // For API requests, return JSON error; for others, let Express handle
      if (req.url?.startsWith("/api")) {
        res.status(404).json({
          type: "https://docs.example.com/errors/not-found",
          title: "Not Found",
          status: 404,
          detail: `The requested resource '${req.url}' was not found`,
        });
        return;
      }
      return next(); // Let Express handle non-API 404s
    }

    // Simple SSR HTML that matches the client structure for hydration
    const appHtml = `<div class="h-screen flex flex-col bg-gray-50">
      <header class="bg-white border-b border-gray-200 px-4 py-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <h1 class="text-xl font-semibold text-gray-900">Agents-Workflow</h1>
          </div>
          <nav class="flex items-center space-x-6">
            <a href="/" class="text-sm font-medium text-blue-600">Dashboard</a>
            <a href="/sessions" class="text-sm font-medium text-gray-600 hover:text-gray-900">Sessions</a>
            <a href="/create" class="text-sm font-medium text-gray-600 hover:text-gray-900">Create Task</a>
            <a href="/settings" class="text-sm font-medium text-gray-600 hover:text-gray-900">Settings</a>
          </nav>
        </div>
      </header>
      <div class="flex-1 flex overflow-hidden">
        <div class="w-80 bg-white border-r border-gray-200 p-4">
          <h2 class="text-lg font-semibold text-gray-900">Loading...</h2>
          <p class="text-sm text-gray-600 mt-1">Client-side content loading</p>
        </div>
        <div class="flex-1 bg-white border-r border-gray-200 p-4">
          <h2 class="text-lg font-semibold text-gray-900">Dashboard</h2>
          <p class="text-sm text-gray-600 mt-1">Loading application...</p>
        </div>
        <div class="w-96 bg-white p-4">
          <h2 class="text-lg font-semibold text-gray-900">Task Details</h2>
          <p class="text-sm text-gray-600 mt-1">Select a session to view details</p>
        </div>
      </div>
    </div>`;

    // HTML template with server-rendered content for hydration
    const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Agents-Workflow WebUI</title>
  <link rel="icon" type="image/svg+xml" href="/favicon.ico">
  <meta name="description" content="Web-based dashboard for creating, monitoring, and managing agent coding sessions">
  <style>
    /* Critical CSS for initial render */
    body {
      margin: 0;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
      background-color: #f8fafc;
      color: #1e293b;
    }
    .h-screen {
      height: 100vh;
    }
    .flex {
      display: flex;
    }
    .flex-col {
      flex-direction: column;
    }
    .bg-gray-50 {
      background-color: #f9fafb;
    }
  </style>
</head>
<body>
  <div id="app">${appHtml}</div>

  <!-- Fallback for when JavaScript is disabled -->
  <noscript>
    <div style="padding: 2rem; text-align: center; background: #fef2f2; border: 1px solid #fca5a5; margin: 1rem; border-radius: 0.5rem;">
      <h2 style="color: #dc2626; margin: 0 0 1rem 0;">JavaScript Required</h2>
      <p style="margin: 0; color: #7f1d1d;">
        This application requires JavaScript to function properly.
        Please enable JavaScript in your browser settings and reload the page.
      </p>
      <p style="margin: 1rem 0 0 0; color: #7f1d1d;">
        <a href="/" style="color: #dc2626; text-decoration: underline;">Reload Page</a>
      </p>
    </div>
  </noscript>

  <!-- Load the client-side JavaScript bundle -->
  <script type="module" src="/client.js"></script>
</body>
</html>`;

    res.status(200).set("Content-Type", "text/html").send(html);
  } catch (error) {
    console.error("SSR Error:", error);
    next(error);
  }
};
