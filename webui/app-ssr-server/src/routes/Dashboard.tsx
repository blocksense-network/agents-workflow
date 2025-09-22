import { Component } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";
import { ThreePaneLayout } from "../components/layout/ThreePaneLayout.js";

export const Dashboard: Component = () => {
  return (
    <MainLayout currentPath="/">
      <ThreePaneLayout />
    </MainLayout>
  );
};
