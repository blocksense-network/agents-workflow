import { Component, createSignal, onMount } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";
import { ThreePaneLayout } from "../components/layout/ThreePaneLayout.js";

export const Sessions: Component = () => {
  const [selectedSessionId, setSelectedSessionId] = createSignal<
    string | undefined
  >();

  // Handle URL hash for session selection
  onMount(() => {
    if (typeof window !== "undefined") {
      const hash = window.location.hash;
      if (hash.startsWith("#session-")) {
        setSelectedSessionId(hash.substring(9)); // Remove '#session-' prefix
      }
    }
  });

  const handleSessionSelect = (sessionId: string) => {
    setSelectedSessionId(sessionId);
    // Update URL hash for bookmarkable links
    if (typeof window !== "undefined") {
      window.location.hash = `session-${sessionId}`;
    }
  };

  return (
    <MainLayout currentPath="/sessions">
      <ThreePaneLayout
        selectedSessionId={selectedSessionId()}
        onSessionSelect={handleSessionSelect}
      />
    </MainLayout>
  );
};
