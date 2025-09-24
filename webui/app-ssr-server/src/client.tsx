import { render } from "solid-js/web";
import { Router, Route } from "@solidjs/router";
import { Dashboard } from "./routes/Dashboard.js";
import { Sessions } from "./routes/Sessions.js";
import { CreateTask } from "./routes/CreateTask.js";
import { Settings } from "./routes/Settings.js";
import "./app.css";

function FullApp() {
  console.log("FullApp component rendering");
  console.log("Current location:", window.location.pathname);

  return (
    <Router>
      <Route path="/" component={Dashboard} />
      <Route path="/sessions" component={Sessions} />
      <Route path="/create" component={CreateTask} />
      <Route path="/settings" component={Settings} />
    </Router>
  );
}

// Render the application directly - no SSR to avoid hydration issues
console.log("Client script loaded, rendering application...");
console.log("Current URL:", window.location.href);

try {
  const appElement = document.getElementById("app");
  console.log("App element found:", appElement);

  if (appElement) {
    console.log("Starting render...");
    render(() => FullApp(), appElement);
    console.log("Render completed successfully");

    // Check if render worked
    setTimeout(() => {
      console.log("Post-render check - app element:", document.getElementById("app")?.innerHTML?.substring(0, 200));
    }, 100);
  } else {
    console.error("App element not found!");
  }
} catch (error) {
  console.error("Render failed:", error);
  console.error("Error stack:", error.stack);
}
