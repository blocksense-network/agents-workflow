export const ssrMiddleware = async (req: any, res: any, next: any) => {
  try {
    // Skip SSR for API routes and static assets
    if (req.url.startsWith("/api") || req.url.includes(".")) {
      return next();
    }

    // HTML template with basic structure for progressive enhancement
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
    .ssr-placeholder {
      display: flex;
      justify-content: center;
      align-items: center;
      height: 100vh;
      font-size: 1.2rem;
      color: #64748b;
    }
    .ssr-loading {
      animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
    }
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: .5; }
    }
  </style>
</head>
<body>
  <div id="app" class="ssr-placeholder ssr-loading">
    Loading Agents-Workflow WebUI...
  </div>

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
