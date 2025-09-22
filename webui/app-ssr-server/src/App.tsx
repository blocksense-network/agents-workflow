import { MainLayout } from "./components/layout/MainLayout.js";
import { ThreePaneLayout } from "./components/layout/ThreePaneLayout.js";

export default function App() {
  return (
    <MainLayout>
      <ThreePaneLayout />
    </MainLayout>
  );
}
