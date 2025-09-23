import { Router } from '@solidjs/router';
import { FileRoutes } from '@solidjs/start/router';
import { MainLayout } from './components/layout/MainLayout';
import { ThreePaneLayout } from './components/layout/ThreePaneLayout';
import './app.css';

export default function App() {
  return (
    <Router
      root={() => (
        <MainLayout>
          <ThreePaneLayout />
        </MainLayout>
      )}
    >
      <FileRoutes />
    </Router>
  );
}
