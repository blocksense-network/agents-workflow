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
      <div class="w-80 bg-white/70 backdrop-blur-sm border border-slate-200/50 rounded-xl shadow-sm flex flex-col overflow-hidden">
        <RepositoriesPane />
      </div>

      {/* Center Pane - Sessions Feed */}
      <div class="flex-1 bg-white/70 backdrop-blur-sm border border-slate-200/50 rounded-xl shadow-sm flex flex-col overflow-hidden">
        <SessionsPane selectedSessionId={props.selectedSessionId} />
      </div>

      {/* Right Pane - Task Details */}
      <div class="w-96 bg-white/70 backdrop-blur-sm border border-slate-200/50 rounded-xl shadow-sm flex flex-col overflow-hidden">
        <TaskDetailsPane sessionId={props.selectedSessionId} />
      </div>
    </>
  );
};
