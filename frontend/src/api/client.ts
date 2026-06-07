import type {
  Metrics,
  Recipe,
  RunDetail,
  RunSummary,
  StoredTelemetry,
} from "../types";

const defaultApiHost =
  typeof window === "undefined" ? "localhost" : window.location.hostname;

export const API_URL =
  import.meta.env.VITE_API_URL ?? `http://${defaultApiHost}:8080`;

export const WS_URL =
  import.meta.env.VITE_WS_URL ??
  API_URL.replace(/^http/, "ws") + "/ws/telemetry";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_URL}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
  });

  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as
      | { error?: string }
      | null;
    throw new Error(body?.error ?? `Request failed with ${response.status}`);
  }

  return response.json() as Promise<T>;
}

export const api = {
  getRecipes: async () => {
    const response = await request<{ recipes: Recipe[] }>("/api/recipes");
    return response.recipes;
  },
  getRuns: () => request<RunSummary[]>("/api/runs"),
  getRun: (runId: string) => request<RunDetail>(`/api/runs/${runId}`),
  getTelemetry: (runId: string) =>
    request<StoredTelemetry[]>(`/api/runs/${runId}/telemetry`),
  getMetrics: () => request<Metrics>("/api/metrics"),
  startRun: (recipeId: string) =>
    request<{ run_id: string; status: string }>("/api/runs/start", {
      method: "POST",
      body: JSON.stringify({ recipe_id: recipeId }),
    }),
  abortRun: (runId: string) =>
    request<{ message: string }>(`/api/runs/${runId}/abort`, {
      method: "POST",
    }),
};
