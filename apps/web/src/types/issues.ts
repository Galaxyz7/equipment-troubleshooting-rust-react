// Auto-generated types for Issues API

export interface Issue {
  id: string;
  name: string;
  category: string;
  display_category?: string | null;
  root_question_id: string;
  is_active: boolean;
  question_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateIssueRequest {
  name: string;
  category: string;
  display_category?: string | null;
  root_question_text: string;
}

export interface UpdateIssueRequest {
  name?: string;
  display_category?: string | null;
  is_active?: boolean;
}

export interface TreeNode {
  question: Question;
  answers: TreeAnswer[];
}

export interface TreeAnswer {
  id: string;
  label: string;
  order_index: number;
  destination: AnswerDestination;
}

export interface AnswerDestination {
  type: string; // "question" or "conclusion"
  question_id: string | null;
  question_text: string | null;
  conclusion_text: string | null;
}

export interface IssueTree {
  issue: Issue;
  nodes: TreeNode[];
}

// Import Question type from existing types
import type { Question } from './Question';
