import axios from 'axios';
import type { LoginRequest, LoginResponse } from '../types';
import type {
  StartSessionRequest,
  StartSessionResponse,
  SubmitAnswerRequest,
  SubmitAnswerResponse,
  SessionHistoryResponse,
  SessionsListResponse,
  DashboardStats,
  AuditLogsResponse,
} from '../types/troubleshoot';
import type {
  Issue,
  CreateIssueRequest,
  UpdateIssueRequest,
  IssueTree,
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
      window.location.href = '/admin/login';
    }
    return Promise.reject(error);
  }
);

export const authAPI = {
  login: async (credentials: LoginRequest): Promise<LoginResponse> => {
    const { data } = await api.post<LoginResponse>('/api/auth/login', credentials);
    return data;
  },

  getMe: async () => {
    const { data } = await api.get('/api/auth/me');
    return data;
  },
};

export const troubleshootAPI = {
  startSession: async (req: StartSessionRequest): Promise<StartSessionResponse> => {
    const { data } = await api.post<StartSessionResponse>('/api/troubleshoot/start', req);
    return data;
  },

  submitAnswer: async (sessionId: string, req: SubmitAnswerRequest): Promise<SubmitAnswerResponse> => {
    const { data } = await api.post<SubmitAnswerResponse>(
      `/api/troubleshoot/${sessionId}/answer`,
      req
    );
    return data;
  },

  getSession: async (sessionId: string): Promise<SubmitAnswerResponse> => {
    const { data} = await api.get<SubmitAnswerResponse>(`/api/troubleshoot/${sessionId}`);
    return data;
  },

  getHistory: async (sessionId: string): Promise<SessionHistoryResponse> => {
    const { data } = await api.get<SessionHistoryResponse>(`/api/troubleshoot/${sessionId}/history`);
    return data;
  },
};

export const questionsAPI = {
  list: async () => {
    const { data } = await api.get('/api/questions');
    return data;
  },

  get: async (id: string) => {
    const { data } = await api.get(`/api/questions/${id}`);
    return data;
  },

  create: async (question: any) => {
    const { data } = await api.post('/api/questions', question);
    return data;
  },

  update: async (id: string, question: any) => {
    const { data } = await api.put(`/api/questions/${id}`, question);
    return data;
  },

  delete: async (id: string) => {
    await api.delete(`/api/questions/${id}`);
  },
};

export const answersAPI = {
  list: async (questionId: string) => {
    const { data } = await api.get(`/api/questions/${questionId}/answers`);
    return data;
  },

  create: async (questionId: string, answer: any) => {
    const { data } = await api.post(`/api/questions/${questionId}/answers`, answer);
    return data;
  },

  update: async (id: string, answer: any) => {
    const { data } = await api.put(`/api/answers/${id}`, answer);
    return data;
  },

  delete: async (id: string) => {
    await api.delete(`/api/answers/${id}`);
  },
};

export const adminAPI = {
  getSessions: async (): Promise<SessionsListResponse> => {
    const { data } = await api.get<SessionsListResponse>('/api/admin/sessions');
    return data;
  },

  getStats: async (): Promise<DashboardStats> => {
    const { data } = await api.get<DashboardStats>('/api/admin/stats');
    return data;
  },

  getAuditLogs: async (): Promise<AuditLogsResponse> => {
    const { data } = await api.get<AuditLogsResponse>('/api/admin/audit-logs');
    return data;
  },
};

export const issuesAPI = {
  list: async (): Promise<Issue[]> => {
    const { data } = await api.get<Issue[]>('/api/admin/issues');
    return data;
  },

  getTree: async (category: string): Promise<IssueTree> => {
    const { data } = await api.get<IssueTree>(`/api/admin/issues/${category}/tree`);
    return data;
  },

  getGraph: async (category: string): Promise<IssueGraph> => {
    const { data } = await api.get<IssueGraph>(`/api/admin/issues/${category}/graph`);
    return data;
  },

  create: async (request: CreateIssueRequest): Promise<Issue> => {
    const { data } = await api.post<Issue>('/api/admin/issues', request);
    return data;
  },

  update: async (category: string, request: UpdateIssueRequest): Promise<Issue> => {
    const { data } = await api.put<Issue>(`/api/admin/issues/${category}`, request);
    return data;
  },

  toggle: async (category: string, force?: boolean): Promise<Issue> => {
    const params = force ? '?force=true' : '';
    const { data } = await api.patch<Issue>(`/api/admin/issues/${category}/toggle${params}`);
    return data;
  },

  delete: async (category: string): Promise<void> => {
    await api.delete(`/api/admin/issues/${category}`);
  },
};

export const nodesAPI = {
  list: async (category?: string, nodeType?: string): Promise<Node[]> => {
    const params = new URLSearchParams();
    if (category) params.append('category', category);
    if (nodeType) params.append('node_type', nodeType);

    const { data } = await api.get<Node[]>(`/api/nodes?${params.toString()}`);
    return data;
  },

  get: async (id: string): Promise<Node> => {
    const { data } = await api.get<Node>(`/api/nodes/${id}`);
    return data;
  },

  getWithConnections: async (id: string): Promise<NodeWithConnections> => {
    const { data } = await api.get<NodeWithConnections>(`/api/nodes/${id}/with-connections`);
    return data;
  },

  create: async (node: CreateNode): Promise<Node> => {
    const { data } = await api.post<Node>('/api/nodes', node);
    return data;
  },

  update: async (id: string, updates: UpdateNode): Promise<Node> => {
    const { data } = await api.put<Node>(`/api/nodes/${id}`, updates);
    return data;
  },

  delete: async (id: string): Promise<void> => {
    await api.delete(`/api/nodes/${id}`);
  },
};

export const connectionsAPI = {
  list: async (fromNodeId?: string, toNodeId?: string): Promise<Connection[]> => {
    const params = new URLSearchParams();
    if (fromNodeId) params.append('from_node_id', fromNodeId);
    if (toNodeId) params.append('to_node_id', toNodeId);

    const { data } = await api.get<Connection[]>(`/api/connections?${params.toString()}`);
    return data;
  },

  create: async (connection: CreateConnection): Promise<Connection> => {
    const { data } = await api.post<Connection>('/api/connections', connection);
    return data;
  },

  update: async (id: string, updates: UpdateConnection): Promise<Connection> => {
    const { data } = await api.put<Connection>(`/api/connections/${id}`, updates);
    return data;
  },

  delete: async (id: string): Promise<void> => {
    await api.delete(`/api/connections/${id}`);
  },
};

export default api;
