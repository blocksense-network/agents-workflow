import { Component } from 'solid-js';
import { RepositoriesPane } from '../repositories/RepositoriesPane';
import { SessionsPane } from '../sessions/SessionsPane';
import { TaskDetailsPane } from '../tasks/TaskDetailsPane';

interface ThreePaneLayoutProps {
  selectedSessionId?: string;
}

export const ThreePaneLayout: Component<ThreePaneLayoutProps> = (props) => {
  return (
    <>
      {/* Left Pane - Repositories */}
      <div class="w-80 bg-white border-r border-gray-200 flex flex-col">
        <RepositoriesPane />
      </div>

      {/* Center Pane - Sessions Feed */}
      <div class="flex-1 bg-white border-r border-gray-200 flex flex-col">
        <SessionsPane selectedSessionId={props.selectedSessionId} />
      </div>

      {/* Right Pane - Task Details */}
      <div class="w-96 bg-white flex flex-col">
        <TaskDetailsPane sessionId={props.selectedSessionId} />
      </div>
    </>
  );
};
