import { Component } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";
import { ThreePaneLayout } from "../components/layout/ThreePaneLayout.js";

export const Dashboard: Component = () => {
  console.log("Dashboard component rendering");

  const handleCreateTaskForRepo = (repo: { id: string; name: string; branch: string; lastCommit: string }) => {
    console.log("Create task for repo:", repo);
    // TODO: Implement inline task creation in the task feed
    // For now, just log the action
  };

  return (
    <MainLayout currentPath="/">
      <ThreePaneLayout onCreateTaskForRepo={handleCreateTaskForRepo} />
    </MainLayout>
  );
};
