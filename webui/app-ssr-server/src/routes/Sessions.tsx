import { Component } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";
import { ThreePaneLayout } from "../components/layout/ThreePaneLayout.js";

export const Sessions: Component = () => {
  return (
    <MainLayout currentPath="/sessions">
      <ThreePaneLayout />
    </MainLayout>
  );
};
