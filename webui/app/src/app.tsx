import { Router } from "@solidjs/router";
import { FileRoutes } from "@solidjs/start/router";
import { Suspense } from "solid-js";
import { MainLayout } from "./components/layout/MainLayout";
import { ThreePaneLayout } from "./components/layout/ThreePaneLayout";
import "./app.css";

export default function App() {
  return (
    <Router
      root={props => (
        <MainLayout>
          <ThreePaneLayout />
        </MainLayout>
      )}
    >
      <FileRoutes />
    </Router>
  );
}
