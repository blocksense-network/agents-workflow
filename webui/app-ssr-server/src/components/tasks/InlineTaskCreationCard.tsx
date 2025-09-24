import {
  Component,
  createSignal,
  createResource,
  Show,
  For,
  createEffect,
  onMount,
} from "solid-js";
import {
  apiClient,
  type CreateTaskRequest,
} from "../../lib/api.js";
import { BranchSelector } from "../common/BranchSelector.js";

interface Repository {
  id: string;
  name: string;
  branch: string;
  lastCommit: string;
}

interface InlineTaskCreationCardProps {
  repository: Repository;
  onTaskCreated?: (taskId: string) => void;
  onCancel?: () => void;
}

interface FormData {
  prompt: string;
  branch: string;
  agentType: string;
  agentVersion: string;
  runtimeType: string;
  deliveryMode: "pr" | "branch" | "patch";
  targetBranch: string;
}

interface FormErrors {
  prompt?: string;
  branch?: string;
  agentType?: string;
  runtimeType?: string;
  general?: string;
}

export const InlineTaskCreationCard: Component<InlineTaskCreationCardProps> = (props) => {
  // Form state
  const [formData, setFormData] = createSignal<FormData>({
    prompt: "",
    branch: props.repository.branch, // Pre-populate with repo's default branch
    agentType: "",
    agentVersion: "latest",
    runtimeType: "",
    deliveryMode: "pr",
    targetBranch: "main",
  });

  const [errors, setErrors] = createSignal<FormErrors>({});
  const [isSubmitting, setIsSubmitting] = createSignal(false);

  // Draft persistence key based on repository
  const draftKey = () => `task-draft-${props.repository.id}`;

  // Load draft from localStorage on mount
  onMount(() => {
    if (typeof window !== "undefined") {
      try {
        const saved = localStorage.getItem(draftKey());
        if (saved) {
          const draft = JSON.parse(saved);
          setFormData((prev) => ({ ...prev, ...draft }));
        }
      } catch (error) {
        console.warn("Failed to load task draft:", error);
      }
    }
  });

  // Save draft to localStorage whenever form data changes
  createEffect(() => {
    if (typeof window !== "undefined") {
      try {
        const data = formData();
        // Only save if there's actual content (not just defaults)
        if (data.prompt.trim() || data.agentType || data.runtimeType) {
          localStorage.setItem(draftKey(), JSON.stringify(data));
        }
      } catch (error) {
        console.warn("Failed to save task draft:", error);
      }
    }
  });

  // API data
  const [agents] = createResource(async () => {
    try {
      const result = await apiClient.listAgents();
      return result.items;
    } catch (error) {
      console.error("Failed to load agents:", error);
      return [];
    }
  });

  const [runtimes] = createResource(async () => {
    try {
      const result = await apiClient.listRuntimes();
      return result.items;
    } catch (error) {
      console.error("Failed to load runtimes:", error);
      return [];
    }
  });

  const selectedAgent = () =>
    agents()?.find((a) => a.type === formData().agentType);

  const validateForm = (): boolean => {
    const data = formData();
    const newErrors: FormErrors = {};

    if (!data.prompt.trim()) {
      newErrors.prompt = "Description is required";
    }

    if (!data.branch.trim()) {
      newErrors.branch = "Branch is required";
    }

    if (!data.agentType) {
      newErrors.agentType = "Please select an agent";
    }

    if (!data.runtimeType) {
      newErrors.runtimeType = "Please select a runtime";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    setIsSubmitting(true);
    setErrors({});

    try {
      const data = formData();
      const taskRequest: CreateTaskRequest = {
        prompt: data.prompt,
        repo: {
          mode: "git",
          url: `https://github.com/${props.repository.name}.git`, // Construct URL from repo name
          branch: data.branch,
        },
        agent: {
          type: data.agentType,
          version: data.agentVersion,
        },
        runtime: {
          type: data.runtimeType as any,
        },
        delivery: {
          mode: data.deliveryMode,
          targetBranch: data.targetBranch,
        },
      };

      const result = await apiClient.createTask(taskRequest);

      // Clear the draft on successful task creation
      if (typeof window !== "undefined") {
        localStorage.removeItem(draftKey());
      }

      props.onTaskCreated?.(result.id);
    } catch (error: any) {
      console.error("Failed to create task:", error);
      setErrors({
        general: error.detail || error.title || "Failed to create task",
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  const updateFormData = (field: keyof FormData, value: any) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    // Clear field-specific error when user starts typing
    if (errors()[field]) {
      setErrors((prev) => ({ ...prev, [field]: undefined }));
    }
  };

  return (
    <div class="bg-blue-50 border-2 border-blue-200 border-dashed rounded-lg p-4 mb-4">
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center space-x-2">
          <div class="w-5 h-5 bg-blue-500 rounded-md flex items-center justify-center">
            <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 6v6m0 0v6m0-6h6m-6 0H6"/>
            </svg>
          </div>
          <span class="text-sm font-medium text-blue-900">
            Create task for {props.repository.name}
          </span>
        </div>
        <button
          type="button"
          onClick={() => {
            // Clear the draft when cancelling
            if (typeof window !== "undefined") {
              localStorage.removeItem(draftKey());
            }
            props.onCancel();
          }}
          class="text-gray-400 hover:text-gray-600 p-1"
          title="Cancel task creation"
          aria-label="Cancel task creation"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <form onSubmit={handleSubmit} class="space-y-3">
        <Show when={errors().general}>
          <div class="bg-red-50 border border-red-200 rounded-md p-2">
            <div class="text-xs text-red-800">{errors().general}</div>
          </div>
        </Show>

        {/* Description */}
        <div>
          <textarea
            rows="2"
            class={`w-full px-2 py-1 text-sm border rounded focus:outline-none focus:ring-1 focus:ring-blue-500 ${
              errors().prompt ? "border-red-300" : "border-gray-300"
            }`}
            placeholder="Describe what you want the agent to accomplish..."
            value={formData().prompt}
            onInput={(e) => updateFormData("prompt", e.currentTarget.value)}
            disabled={isSubmitting()}
          />
          <Show when={errors().prompt}>
            <p class="mt-1 text-xs text-red-600">{errors().prompt}</p>
          </Show>
        </div>

        {/* Branch and Agent/Runtime row */}
        <div class="grid grid-cols-1 md:grid-cols-3 gap-2">
          <div>
            <BranchSelector
              repository={props.repository.name}
              value={formData().branch}
              onChange={(value) => updateFormData("branch", value)}
              disabled={isSubmitting()}
              class={`w-full px-2 py-1 text-sm border rounded focus:outline-none focus:ring-1 focus:ring-blue-500 ${
                errors().branch ? "border-red-300" : "border-gray-300"
              }`}
            />
            <Show when={errors().branch}>
              <p class="mt-1 text-xs text-red-600">{errors().branch}</p>
            </Show>
          </div>

          <div>
            <select
              class={`w-full px-2 py-1 text-sm border rounded focus:outline-none focus:ring-1 focus:ring-blue-500 ${
                errors().agentType ? "border-red-300" : "border-gray-300"
              }`}
              value={formData().agentType}
              onChange={(e) => updateFormData("agentType", e.currentTarget.value)}
              disabled={isSubmitting()}
            >
              <option value="">Agent</option>
              <For each={agents()}>
                {(agent) => <option value={agent.type}>{agent.type}</option>}
              </For>
            </select>
            <Show when={errors().agentType}>
              <p class="mt-1 text-xs text-red-600">{errors().agentType}</p>
            </Show>
          </div>

          <div>
            <select
              class={`w-full px-2 py-1 text-sm border rounded focus:outline-none focus:ring-1 focus:ring-blue-500 ${
                errors().runtimeType ? "border-red-300" : "border-gray-300"
              }`}
              value={formData().runtimeType}
              onChange={(e) => updateFormData("runtimeType", e.currentTarget.value)}
              disabled={isSubmitting()}
            >
              <option value="">Runtime</option>
              <For each={runtimes()}>
                {(runtime) => <option value={runtime.type}>{runtime.type}</option>}
              </For>
            </select>
            <Show when={errors().runtimeType}>
              <p class="mt-1 text-xs text-red-600">{errors().runtimeType}</p>
            </Show>
          </div>
        </div>

        {/* Actions */}
        <div class="flex justify-end space-x-2 pt-2">
          <button
            type="submit"
            class="px-3 py-1 text-xs font-medium text-white bg-blue-600 border border-transparent rounded hover:bg-blue-700 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={isSubmitting()}
          >
            {isSubmitting() ? "Creating..." : "Start Task"}
          </button>
        </div>
      </form>
    </div>
  );
};
