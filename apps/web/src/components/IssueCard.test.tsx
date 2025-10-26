import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import IssueCard from './IssueCard';
import type { Issue } from '../types/issues';

/**
 * IssueCard Component Tests
 * Tests rendering, interactions, and state management
 */

const mockIssue: Issue = {
  id: '123',
  name: 'Hardware Issues',
  category: 'hardware',
  display_category: 'Hardware',
  root_question_id: 'q1',
  is_active: true,
  question_count: 10n,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

describe('IssueCard', () => {
  it('should render issue information correctly', () => {
    const onToggle = vi.fn();
    const onTest = vi.fn();
    const onEdit = vi.fn();
    const onDelete = vi.fn();

    render(
      <IssueCard
        issue={mockIssue}
        onToggle={onToggle}
        onTest={onTest}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    );

    // Check if issue name is displayed
    expect(screen.getByText('Hardware Issues')).toBeInTheDocument();

    // Check if category badges are displayed
    expect(screen.getByText('Hardware')).toBeInTheDocument();
    expect(screen.getByText('hardware')).toBeInTheDocument();

    // Check if question count is displayed
    expect(screen.getByText('10 questions in this decision tree')).toBeInTheDocument();

    // Check if active status is displayed
    expect(screen.getByText('Active')).toBeInTheDocument();
  });

  it('should render inactive state correctly', () => {
    const inactiveIssue: Issue = { ...mockIssue, is_active: false };
    const onToggle = vi.fn();
    const onTest = vi.fn();
    const onEdit = vi.fn();
    const onDelete = vi.fn();

    render(
      <IssueCard
        issue={inactiveIssue}
        onToggle={onToggle}
        onTest={onTest}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    );

    expect(screen.getByText('Inactive')).toBeInTheDocument();
  });

  it('should handle single question correctly', () => {
    const singleQuestionIssue: Issue = { ...mockIssue, question_count: 1n };
    const onToggle = vi.fn();
    const onTest = vi.fn();
    const onEdit = vi.fn();
    const onDelete = vi.fn();

    render(
      <IssueCard
        issue={singleQuestionIssue}
        onToggle={onToggle}
        onTest={onTest}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    );

    expect(screen.getByText('1 question in this decision tree')).toBeInTheDocument();
  });

  it('should call onEdit when Edit Tree button is clicked', async () => {
    const onToggle = vi.fn();
    const onTest = vi.fn();
    const onEdit = vi.fn();
    const onDelete = vi.fn();
    const user = userEvent.setup();

    render(
      <IssueCard
        issue={mockIssue}
        onToggle={onToggle}
        onTest={onTest}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    );

    const editButton = screen.getByText(/Edit Tree/);
    await user.click(editButton);

    expect(onEdit).toHaveBeenCalledWith('hardware');
  });

  it('should call onTest when Test button is clicked', async () => {
    const onToggle = vi.fn();
    const onTest = vi.fn();
    const onEdit = vi.fn();
    const onDelete = vi.fn();
    const user = userEvent.setup();

    render(
      <IssueCard
        issue={mockIssue}
        onToggle={onToggle}
        onTest={onTest}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    );

    const testButton = screen.getByText(/Test/);
    await user.click(testButton);

    expect(onTest).toHaveBeenCalledWith('hardware');
  });

  it('should call onToggle when toggle button is clicked', async () => {
    const onToggle = vi.fn().mockResolvedValue(undefined);
    const onTest = vi.fn();
    const onEdit = vi.fn();
    const onDelete = vi.fn();
    const user = userEvent.setup();

    render(
      <IssueCard
        issue={mockIssue}
        onToggle={onToggle}
        onTest={onTest}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    );

    const toggleButton = screen.getByTitle(/Turn issue off/);
    await user.click(toggleButton);

    await waitFor(() => {
      expect(onToggle).toHaveBeenCalledWith('hardware');
    });
  });

  it('should render all action buttons', () => {
    const onToggle = vi.fn();
    const onTest = vi.fn();
    const onEdit = vi.fn();
    const onDelete = vi.fn();

    render(
      <IssueCard
        issue={mockIssue}
        onToggle={onToggle}
        onTest={onTest}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    );

    expect(screen.getByText(/Edit Tree/)).toBeInTheDocument();
    expect(screen.getByText(/Test/)).toBeInTheDocument();
    expect(screen.getByText(/Delete/)).toBeInTheDocument();
  });

  it('should display delete loading state', async () => {
    const onToggle = vi.fn();
    const onTest = vi.fn();
    const onEdit = vi.fn();
    const onDelete = vi.fn().mockImplementation(() => new Promise(() => {})); // Never resolves
    const user = userEvent.setup();

    // Mock window.confirm to return true
    vi.stubGlobal('confirm', vi.fn(() => true));

    render(
      <IssueCard
        issue={mockIssue}
        onToggle={onToggle}
        onTest={onTest}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    );

    const deleteButton = screen.getByText(/Delete/);
    await user.click(deleteButton);

    await waitFor(() => {
      expect(screen.getByText('...')).toBeInTheDocument();
    });

    vi.unstubAllGlobals();
  });
});
