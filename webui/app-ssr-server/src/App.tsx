import { Router, Route } from "@solidjs/router";
import { Dashboard } from "./routes/Dashboard.js";
import { Sessions } from "./routes/Sessions.js";
import { CreateTask } from "./routes/CreateTask.js";
import { Settings } from "./routes/Settings.js";

export default function App() {
  return (
    <Router>
      <Route path="/" component={Dashboard} />
      <Route path="/sessions" component={Sessions} />
      <Route path="/create" component={CreateTask} />
      <Route path="/settings" component={Settings} />
    </Router>
  );
}
