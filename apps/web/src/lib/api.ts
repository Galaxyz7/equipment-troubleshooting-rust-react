import axios from 'axios';
import type { LoginRequest, LoginResponse, UserInfo } from '../types';
import type {
  StartSessionRequest,
  StartSessionResponse,
  SubmitAnswerRequest,
  SubmitAnswerResponse,
  SessionHistoryResponse,
  SessionsListResponse,
  DashboardStats,
  AuditLogsResponse,
  DeleteSessionsResponse,
  CategoryListResponse,
  RenameCategoryRequest,
  CategoryUpdateResponse,
} from '../types/troubleshoot';
import type {
  Issue,
  CreateIssueRequest,
  UpdateIssueRequest,
  IssueExportData,
  ImportResult,
} from '../types/issues';
import type {
  Node,
  CreateNode,
  UpdateNode,
  Connection,
  CreateConnection,
  UpdateConnection,
  IssueGraph,
  NodeWithConnections,
} from '../types';

// Auto-detect API URL based on current page
// In production, frontend is served by the backend, so we use the same origin
// In development, you can set VITE_API_URL in .env for separate dev servers
const getApiBaseUrl = () => {
  // If VITE_API_URL is set (development mode), use it
  if (import.meta.env.VITE_API_URL) {
    return import.meta.env.VITE_API_URL;
  }

  // Otherwise, auto-detect from current window location (production mode)
  // This allows the API URL to be configured purely from the backend .env
  const protocol = window.location.protocol; // http: or https:
  const hostname = window.location.hostname;
  const port = window.location.port;

  // Construct the API base URL from current page URL
  return port ? `${protocol}//${hostname}:${port}` : `${protocol}//${hostname}`;
};

const API_BASE_URL = getApiBaseUrl();

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor to add auth token
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Response interceptor to handle errors
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

export const authAPI = {
  login: async (credentials: LoginRequest): Promise<LoginResponse> => {
    const { data } = await api.post<LoginResponse>('/api/v1/auth/login', credentials);
    return data;
  },

  getMe: async (): Promise<UserInfo> => {
    const { data } = await api.get<UserInfo>('/api/v1/auth/me');
    return data;
  },
};

export const troubleshootAPI = {
  startSession: async (req: StartSessionRequest): Promise<StartSessionResponse> => {
    const { data } = await api.post<StartSessionResponse>('/api/v1/troubleshoot/start', req);
    return data;
  },

  submitAnswer: async (sessionId: string, req: SubmitAnswerRequest): Promise<SubmitAnswerResponse> => {
    const { data } = await api.post<SubmitAnswerResponse>(
      `/api/v1/troubleshoot/${sessionId}/answer`,
      req
    );
    return data;
  },

  getSession: async (sessionId: string): Promise<SubmitAnswerResponse> => {
    const { data} = await api.get<SubmitAnswerResponse>(`/api/v1/troubleshoot/${sessionId}`);
    return data;
  },

  getHistory: async (sessionId: string): Promise<SessionHistoryResponse> => {
    const { data } = await api.get<SessionHistoryResponse>(`/api/v1/troubleshoot/${sessionId}/history`);
    return data;
  },
};

export const adminAPI = {
  getSessions: async (): Promise<SessionsListResponse> => {
    const { data } = await api.get<SessionsListResponse>('/api/v1/admin/sessions');
    return data;
  },

  getStats: async (): Promise<DashboardStats> => {
    const { data } = await api.get<DashboardStats>('/api/v1/admin/stats');
    return data;
  },

  getAuditLogs: async (): Promise<AuditLogsResponse> => {
    const { data } = await api.get<AuditLogsResponse>('/api/v1/admin/audit-logs');
    return data;
  },

  getSessionsCount: async (params: URLSearchParams): Promise<{ count: number }> => {
    const { data } = await api.get<{ count: number }>(`/api/v1/admin/sessions/count?${params.toString()}`);
    return data;
  },

  deleteSessions: async (params: URLSearchParams): Promise<DeleteSessionsResponse> => {
    const { data } = await api.delete<DeleteSessionsResponse>(`/api/v1/admin/sessions?${params.toString()}`);
    return data;
  },

  getCategories: async (): Promise<CategoryListResponse> => {
    const { data } = await api.get<CategoryListResponse>('/api/v1/admin/categories');
    return data;
  },

  renameCategory: async (oldName: string, newName: string): Promise<CategoryUpdateResponse> => {
    const request: RenameCategoryRequest = { new_name: newName };
    const { data } = await api.put<CategoryUpdateResponse>(`/api/v1/admin/categories/${encodeURIComponent(oldName)}`, request);
    return data;
  },

  deleteCategory: async (name: string): Promise<CategoryUpdateResponse> => {
    const { data } = await api.delete<CategoryUpdateResponse>(`/api/v1/admin/categories/${encodeURIComponent(name)}`);
    return data;
  },
};

export const issuesAPI = {
  list: async (): Promise<Issue[]> => {
    const { data } = await api.get<Issue[]>('/api/v1/admin/issues');
    return data;
  },

  getGraph: async (category: string): Promise<IssueGraph> => {
    const { data } = await api.get<IssueGraph>(`/api/v1/admin/issues/${category}/graph`);
    return data;
  },

  create: async (request: CreateIssueRequest): Promise<Issue> => {
    const { data } = await api.post<Issue>('/api/v1/admin/issues', request);
    return data;
  },

  update: async (category: string, request: UpdateIssueRequest): Promise<Issue> => {
    const { data } = await api.put<Issue>(`/api/v1/admin/issues/${category}`, request);
    return data;
  },

  toggle: async (category: string, force?: boolean): Promise<Issue> => {
    const params = force ? '?force=true' : '';
    const { data } = await api.patch<Issue>(`/api/v1/admin/issues/${category}/toggle${params}`);
    return data;
  },

  delete: async (category: string, deleteSessions?: boolean): Promise<void> => {
    const params = deleteSessions ? '?delete_sessions=true' : '';
    await api.delete(`/api/v1/admin/issues/${category}${params}`);
  },

  exportIssue: async (category: string): Promise<IssueExportData> => {
    const { data } = await api.get<IssueExportData>(`/api/v1/admin/issues/${category}/export`);
    return data;
  },

  exportAll: async (): Promise<IssueExportData[]> => {
    const { data } = await api.get<IssueExportData[]>('/api/v1/admin/issues/export-all');
    return data;
  },

  importIssues: async (issues: IssueExportData[]): Promise<ImportResult> => {
    const { data } = await api.post<ImportResult>('/api/v1/admin/issues/import', issues);
    return data;
  },
};

export const nodesAPI = {
  list: async (category?: string, nodeType?: string): Promise<Node[]> => {
    const params = new URLSearchParams();
    if (category) params.append('category', category);
    if (nodeType) params.append('node_type', nodeType);

    const { data } = await api.get<Node[]>(`/api/v1/nodes?${params.toString()}`);
    return data;
  },

  get: async (id: string): Promise<Node> => {
    const { data } = await api.get<Node>(`/api/v1/nodes/${id}`);
    return data;
  },

  getWithConnections: async (id: string): Promise<NodeWithConnections> => {
    const { data } = await api.get<NodeWithConnections>(`/api/v1/nodes/${id}/with-connections`);
    return data;
  },

  create: async (node: CreateNode): Promise<Node> => {
    const { data } = await api.post<Node>('/api/v1/nodes', node);
    return data;
  },

  update: async (id: string, updates: UpdateNode): Promise<Node> => {
    const { data } = await api.put<Node>(`/api/v1/nodes/${id}`, updates);
    return data;
  },

  delete: async (id: string): Promise<void> => {
    await api.delete(`/api/v1/nodes/${id}`);
  },
};

export const connectionsAPI = {
  list: async (fromNodeId?: string, toNodeId?: string): Promise<Connection[]> => {
    const params = new URLSearchParams();
    if (fromNodeId) params.append('from_node_id', fromNodeId);
    if (toNodeId) params.append('to_node_id', toNodeId);

    const { data} = await api.get<Connection[]>(`/api/v1/connections?${params.toString()}`);
    return data;
  },

  create: async (connection: CreateConnection): Promise<Connection> => {
    const { data } = await api.post<Connection>('/api/v1/connections', connection);
    return data;
  },

  update: async (id: string, updates: UpdateConnection): Promise<Connection> => {
    const { data } = await api.put<Connection>(`/api/v1/connections/${id}`, updates);
    return data;
  },

  delete: async (id: string): Promise<void> => {
    await api.delete(`/api/v1/connections/${id}`);
  },
};

export default api;
