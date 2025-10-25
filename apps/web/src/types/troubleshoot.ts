// Re-export generated types from Rust
export type { StartSessionRequest } from './StartSessionRequest';
export type { StartSessionResponse } from './StartSessionResponse';
export type { SubmitAnswerRequest } from './SubmitAnswerRequest';
export type { SubmitAnswerResponse } from './SubmitAnswerResponse';
export type { HistoryStep } from './HistoryStep';
import type { HistoryStep } from './HistoryStep';

export interface SessionHistoryResponse {
  session_id: string;
  started_at: string;
  completed: boolean;
  steps: HistoryStep[];
  final_conclusion: string | null;
}

export interface SessionSummary {
  session_id: string;
  started_at: string;
  completed_at: string | null;
  abandoned: boolean;
  tech_identifier: string | null;
  client_site: string | null;
  final_conclusion: string | null;
  step_count: number;
}

export interface SessionsListResponse {
  sessions: SessionSummary[];
  total_count: number;
  page: number;
  page_size: number;
}

export interface ConclusionStats {
  conclusion: string;
  count: number;
}

export interface CategoryStats {
  category: string;
  count: number;
}

export interface DashboardStats {
  total_sessions: number;
  completed_sessions: number;
  abandoned_sessions: number;
  active_sessions: number;
  avg_steps_to_completion: number;
  most_common_conclusions: ConclusionStats[];
  sessions_by_category: CategoryStats[];
}

export interface AuditLogEntry {
  id: number;
  timestamp: string;
  user_id: number | null;
  action: string;
  entity_type: string;
  entity_id: string;
}

export interface AuditLogsResponse {
  logs: AuditLogEntry[];
  total_count: number;
  page: number;
  page_size: number;
}
