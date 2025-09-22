import { createProxyMiddleware } from "http-proxy-middleware";

const NODE_ENV = process.env.NODE_ENV || "development";
const RUST_API_URL = process.env.RUST_API_URL || "http://localhost:8080";
const MOCK_API_URL = process.env.MOCK_API_URL || "http://localhost:3001";

export const apiProxyMiddleware = createProxyMiddleware({
  target: NODE_ENV === "production" ? RUST_API_URL : MOCK_API_URL,
  changeOrigin: true,
  pathRewrite: {
    "^/api": "/api/v1", // Rewrite /api to /api/v1 to match REST service spec
  },
  onProxyReq: (proxyReq, _req, _res) => {
    // Add any necessary headers for the target service
    if (NODE_ENV === "production") {
      // Add authentication headers for production Rust service
      const apiKey = process.env.API_KEY;
      if (apiKey) {
        proxyReq.setHeader("Authorization", `ApiKey ${apiKey}`);
      }
    }
  },
  onProxyRes: (proxyRes, req, _res) => {
    // Log proxy requests in development
    if (NODE_ENV === "development") {
      console.log(
        `[API Proxy] ${req.method} ${req.url} -> ${proxyRes.statusCode}`,
      );
    }
  },
  onError: (err, req, res) => {
    console.error("[API Proxy Error]", err);
    res.status(502).json({
      type: "https://docs.example.com/errors/bad-gateway",
      title: "Bad Gateway",
      status: 502,
      detail: "Failed to proxy request to API service",
    });
  },
});
