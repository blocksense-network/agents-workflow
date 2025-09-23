import { hydrate } from "solid-js/web";
import { Router, Route } from "@solidjs/router";
import { Dashboard } from "./routes/Dashboard.js";
import { Sessions } from "./routes/Sessions.js";
import { CreateTask } from "./routes/CreateTask.js";
import { Settings } from "./routes/Settings.js";

function FullApp() {
  return (
    <Router>
      <Route path="/" component={Dashboard} />
      <Route path="/sessions" component={Sessions} />
      <Route path="/create" component={CreateTask} />
      <Route path="/settings" component={Settings} />
    </Router>
  );
}

// Hydrate the server-rendered content with the full application
// The server rendered a simple HTML structure, and the client will replace it with the full SolidJS app
hydrate(() => FullApp(), document.getElementById("app")!);
