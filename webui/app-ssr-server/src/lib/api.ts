// API client for Agents-Workflow REST service

export interface Repository {
  mode: "git" | "upload" | "none";
  url?: string;
  branch?: string;
  commit?: string;
}

export interface Runtime {
  type: "devcontainer" | "local" | "disabled";
  devcontainerPath?: string;
  resources?: {
    cpu?: number;
    memoryMiB?: number;
  };
}

export interface Agent {
  type: string;
  version: string;
  settings?: Record<string, any>;
}

export interface Delivery {
  mode: "pr" | "branch" | "patch";
  targetBranch?: string;
}

export interface CreateTaskRequest {
  tenantId?: string;
  projectId?: string;
  prompt: string;
  repo: Repository;
  runtime: Runtime;
  agent: Agent;
  delivery?: Delivery;
  labels?: Record<string, string>;
  webhooks?: Array<{
    event: string;
    url: string;
  }>;
}

export interface CreateTaskResponse {
  id: string;
  status: string;
  links: {
    self: string;
    events: string;
    logs: string;
  };
}

export interface Session {
  id: string;
  tenantId?: string;
  projectId?: string;
  status:
    | "queued"
    | "provisioning"
    | "running"
    | "pausing"
    | "paused"
    | "resuming"
    | "stopping"
    | "stopped"
    | "completed"
    | "failed"
    | "cancelled";
  createdAt: string;
  prompt: string;
  repo: Repository;
  runtime: Runtime;
  agent: Agent;
  delivery?: Delivery;
  labels?: Record<string, string>;
  links: {
    self: string;
    events: string;
    logs: string;
  };
}

export interface SessionsListResponse {
  items: Session[];
  pagination: {
    page: number;
    perPage: number;
    total: number;
    totalPages: number;
  };
}

export interface AgentType {
  type: string;
  versions: string[];
  settingsSchemaRef?: string;
}

export interface RuntimeType {
  type: string;
  images?: string[];
  paths?: string[];
  sandboxProfiles?: string[];
}

export interface LogEntry {
  level: string;
  message: string;
  ts: string;
}

export interface LogsResponse {
  logs: LogEntry[];
}

export interface ApiError {
  type: string;
  title: string;
  status: number;
  detail: string;
  errors?: Record<string, string[]>;
}

// API base URL - will be configured based on environment
const API_BASE = "/api/v1";

class ApiClient {
  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
  ): Promise<T> {
    const url = `${API_BASE}${endpoint}`;

    const response = await fetch(url, {
      headers: {
        "Content-Type": "application/json",
        ...options.headers,
      },
      ...options,
    });

    if (!response.ok) {
      const errorData: ApiError = await response.json().catch(() => ({
        type: "unknown",
        title: "Network Error",
        status: response.status,
        detail: response.statusText,
      }));
      throw errorData;
    }

    return response.json();
  }

  // Task operations
  async createTask(data: CreateTaskRequest): Promise<CreateTaskResponse> {
    return this.request("/tasks", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  // Session operations
  async listSessions(params?: {
    status?: string;
    projectId?: string;
    page?: number;
    perPage?: number;
  }): Promise<SessionsListResponse> {
    const searchParams = new URLSearchParams();
    if (params?.status) searchParams.set("status", params.status);
    if (params?.projectId) searchParams.set("projectId", params.projectId);
    if (params?.page) searchParams.set("page", params.page.toString());
    if (params?.perPage) searchParams.set("perPage", params.perPage.toString());

    const query = searchParams.toString();
    return this.request(`/sessions${query ? `?${query}` : ""}`);
  }

  async getSession(id: string): Promise<Session> {
    return this.request(`/sessions/${id}`);
  }

  async stopSession(id: string): Promise<{ status: string }> {
    return this.request(`/sessions/${id}/stop`, {
      method: "POST",
    });
  }

  async cancelSession(id: string): Promise<void> {
    await this.request(`/sessions/${id}`, {
      method: "DELETE",
    });
  }

  async pauseSession(id: string): Promise<{ status: string }> {
    return this.request(`/sessions/${id}/pause`, {
      method: "POST",
    });
  }

  async resumeSession(id: string): Promise<{ status: string }> {
    return this.request(`/sessions/${id}/resume`, {
      method: "POST",
    });
  }

  async getSessionLogs(id: string, tail?: number): Promise<LogsResponse> {
    const params = tail ? `?tail=${tail}` : "";
    return this.request(`/sessions/${id}/logs${params}`);
  }

  // Metadata operations
  async listAgents(): Promise<{ items: AgentType[] }> {
    return this.request("/agents");
  }

  async listRuntimes(): Promise<{ items: RuntimeType[] }> {
    return this.request("/runtimes");
  }
}

export const apiClient = new ApiClient();
