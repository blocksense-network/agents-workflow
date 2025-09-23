import {
  Component,
  createSignal,
  createResource,
  Show,
  For,
  createEffect,
} from "solid-js";
import {
  apiClient,
  type CreateTaskRequest,
  type AgentType,
  type RuntimeType,
} from "../../lib/api.js";

interface TaskCreationFormProps {
  onTaskCreated?: (taskId: string) => void;
  onCancel?: () => void;
}

interface FormData extends CreateTaskRequest {
  // Additional form state
  repoUrl: string;
  repoBranch: string;
  agentType: string;
  agentVersion: string;
  runtimeType: string;
  deliveryMode: "pr" | "branch" | "patch";
  targetBranch: string;
}

interface FormErrors {
  prompt?: string;
  repoUrl?: string;
  repoBranch?: string;
  agentType?: string;
  runtimeType?: string;
  general?: string;
}

export const TaskCreationForm: Component<TaskCreationFormProps> = (props) => {
  // Form state
  const [formData, setFormData] = createSignal<FormData>({
    prompt: "",
    repoUrl: "",
    repoBranch: "main",
    agentType: "",
    agentVersion: "latest",
    runtimeType: "",
    deliveryMode: "pr",
    targetBranch: "main",
    repo: { mode: "git" },
    runtime: { type: "devcontainer" },
    agent: { type: "", version: "latest" },
    delivery: { mode: "pr", targetBranch: "main" },
  });

  const [errors, setErrors] = createSignal<FormErrors>({});
  const [isSubmitting, setIsSubmitting] = createSignal(false);

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

  // Update form data when selections change
  createEffect(() => {
    const data = formData();
    setFormData({
      ...data,
      repo: {
        mode: "git" as const,
        url: data.repoUrl,
        branch: data.repoBranch,
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
    });
  });

  const selectedAgent = () =>
    agents()?.find((a) => a.type === formData().agentType);
  const selectedRuntime = () =>
    runtimes()?.find((r) => r.type === formData().runtimeType);

  const validateForm = (): boolean => {
    const data = formData();
    const newErrors: FormErrors = {};

    if (!data.prompt.trim()) {
      newErrors.prompt = "Prompt is required";
    }

    if (!data.repoUrl.trim()) {
      newErrors.repoUrl = "Repository URL is required";
    } else if (!data.repoUrl.includes("://")) {
      newErrors.repoUrl = "Please enter a valid repository URL";
    }

    if (!data.repoBranch.trim()) {
      newErrors.repoBranch = "Branch is required";
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
          url: data.repoUrl,
          branch: data.repoBranch,
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
    <form onSubmit={handleSubmit} class="space-y-6">
      <Show when={errors().general}>
        <div class="bg-red-50 border border-red-200 rounded-md p-4">
          <div class="text-sm text-red-800">{errors().general}</div>
        </div>
      </Show>

      {/* Prompt */}
      <div>
        <label
          for="prompt"
          class="block text-sm font-medium text-gray-700 mb-2"
        >
          Task Prompt *
        </label>
        <textarea
          id="prompt"
          rows="4"
          class={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
            errors().prompt ? "border-red-300" : "border-gray-300"
          }`}
          placeholder="Describe what you want the agent to accomplish..."
          value={formData().prompt}
          onInput={(e) => updateFormData("prompt", e.currentTarget.value)}
          disabled={isSubmitting()}
        />
        <Show when={errors().prompt}>
          <p class="mt-1 text-sm text-red-600">{errors().prompt}</p>
        </Show>
      </div>

      {/* Repository */}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label
            for="repoUrl"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            Repository URL *
          </label>
          <input
            type="url"
            id="repoUrl"
            class={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
              errors().repoUrl ? "border-red-300" : "border-gray-300"
            }`}
            placeholder="https://github.com/user/repo.git"
            value={formData().repoUrl}
            onInput={(e) => updateFormData("repoUrl", e.currentTarget.value)}
            disabled={isSubmitting()}
          />
          <Show when={errors().repoUrl}>
            <p class="mt-1 text-sm text-red-600">{errors().repoUrl}</p>
          </Show>
        </div>

        <div>
          <label
            for="repoBranch"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            Branch *
          </label>
          <input
            type="text"
            id="repoBranch"
            class={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
              errors().repoBranch ? "border-red-300" : "border-gray-300"
            }`}
            placeholder="main"
            value={formData().repoBranch}
            onInput={(e) => updateFormData("repoBranch", e.currentTarget.value)}
            disabled={isSubmitting()}
          />
          <Show when={errors().repoBranch}>
            <p class="mt-1 text-sm text-red-600">{errors().repoBranch}</p>
          </Show>
        </div>
      </div>

      {/* Agent */}
      <div>
        <label
          for="agentType"
          class="block text-sm font-medium text-gray-700 mb-2"
        >
          Agent *
        </label>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <select
            id="agentType"
            class={`px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
              errors().agentType ? "border-red-300" : "border-gray-300"
            }`}
            value={formData().agentType}
            onChange={(e) => updateFormData("agentType", e.currentTarget.value)}
            disabled={isSubmitting()}
          >
            <option value="">Select an agent...</option>
            <For each={agents()}>
              {(agent) => <option value={agent.type}>{agent.type}</option>}
            </For>
          </select>

          <Show when={selectedAgent()}>
            <select
              class="px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              value={formData().agentVersion}
              onChange={(e) =>
                updateFormData("agentVersion", e.currentTarget.value)
              }
              disabled={isSubmitting()}
            >
              <For each={selectedAgent()!.versions}>
                {(version) => <option value={version}>{version}</option>}
              </For>
            </select>
          </Show>
        </div>
        <Show when={errors().agentType}>
          <p class="mt-1 text-sm text-red-600">{errors().agentType}</p>
        </Show>
      </div>

      {/* Runtime */}
      <div>
        <label
          for="runtimeType"
          class="block text-sm font-medium text-gray-700 mb-2"
        >
          Runtime *
        </label>
        <select
          id="runtimeType"
          class={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 ${
            errors().runtimeType ? "border-red-300" : "border-gray-300"
          }`}
          value={formData().runtimeType}
          onChange={(e) => updateFormData("runtimeType", e.currentTarget.value)}
          disabled={isSubmitting()}
        >
          <option value="">Select a runtime...</option>
          <For each={runtimes()}>
            {(runtime) => <option value={runtime.type}>{runtime.type}</option>}
          </For>
        </select>
        <Show when={errors().runtimeType}>
          <p class="mt-1 text-sm text-red-600">{errors().runtimeType}</p>
        </Show>
      </div>

      {/* Delivery */}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label
            for="deliveryMode"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            Delivery Mode
          </label>
          <select
            id="deliveryMode"
            class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            value={formData().deliveryMode}
            onChange={(e) =>
              updateFormData("deliveryMode", e.currentTarget.value as any)
            }
            disabled={isSubmitting()}
          >
            <option value="pr">Pull Request</option>
            <option value="branch">Branch Push</option>
            <option value="patch">Patch Artifact</option>
          </select>
        </div>

        <div>
          <label
            for="targetBranch"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            Target Branch
          </label>
          <input
            type="text"
            id="targetBranch"
            class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="main"
            value={formData().targetBranch}
            onInput={(e) =>
              updateFormData("targetBranch", e.currentTarget.value)
            }
            disabled={isSubmitting()}
          />
        </div>
      </div>

      {/* Actions */}
      <div class="flex justify-end space-x-3 pt-4">
        <Show when={props.onCancel}>
          <button
            type="button"
            onClick={props.onCancel}
            class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            disabled={isSubmitting()}
          >
            Cancel
          </button>
        </Show>
        <button
          type="submit"
          class="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          disabled={isSubmitting()}
        >
          {isSubmitting() ? "Creating Task..." : "Create Task"}
        </button>
      </div>
    </form>
  );
};
