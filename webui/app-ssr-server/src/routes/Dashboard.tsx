import { Component } from "solid-js";
import { MainLayout } from "../components/layout/MainLayout.js";
import { ThreePaneLayout } from "../components/layout/ThreePaneLayout.js";

export const Dashboard: Component = () => {
  console.log("Dashboard component rendering");

  return (
    <MainLayout currentPath="/">
      <ThreePaneLayout />
    </MainLayout>
  );
};
