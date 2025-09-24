import { Component, createSignal, createEffect, createMemo } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";
import { ThreePaneLayout } from "../components/layout/ThreePaneLayout.js";

export const Dashboard: Component = () => {
  // State for inline task creation
  const [inlineTaskCreationRepo, setInlineTaskCreationRepo] = createSignal<{ id: string; name: string; branch: string; lastCommit: string } | null>(null);

  // Create a memoized value to ensure reactivity
  const currentInlineRepo = createMemo(() => {
    const repo = inlineTaskCreationRepo();
    console.log("Dashboard: currentInlineRepo memo updated, value:", repo);
    return repo;
  });

  console.log("Dashboard component function called, currentInlineRepo:", currentInlineRepo());

  // Monitor signal changes
  createEffect(() => {
    const repo = inlineTaskCreationRepo();
    console.log("Dashboard: inlineTaskCreationRepo signal changed to:", repo);
  });

  const handleCreateTaskForRepo = (repo: { id: string; name: string; branch: string; lastCommit: string }) => {
    console.log("Create task for repo:", repo);
    console.log("Setting inlineTaskCreationRepo to:", repo);
    setInlineTaskCreationRepo(repo);
    console.log("setInlineTaskCreationRepo called, signal should update");
  };

  const handleInlineTaskCreated = (taskId: string) => {
    console.log("Task created:", taskId);
    // Hide the inline form and refresh the task feed (will happen automatically via the API refresh)
    setInlineTaskCreationRepo(null);
  };

  const handleCancelInlineTaskCreation = () => {
    setInlineTaskCreationRepo(null);
  };

  return (
    <MainLayout currentPath="/">
      <ThreePaneLayout
        onCreateTaskForRepo={handleCreateTaskForRepo}
        inlineTaskCreationRepo={currentInlineRepo()}
        onInlineTaskCreated={handleInlineTaskCreated}
        onCancelInlineTaskCreation={handleCancelInlineTaskCreation}
      />
    </MainLayout>
  );
};
